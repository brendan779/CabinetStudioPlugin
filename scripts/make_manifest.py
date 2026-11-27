#!/usr/bin/env python3
"""Writes plugin/cabinet-studio/plugin.json and the dialogs' pictures from target/catalog.json,
which `cargo run --example catalog` writes from Cabinet Studio's core (docs/PLAN.md). The Job
setup's choices are the core's option groups, so the plugin offers what Cabinet Studio does; the
pictures are each cabinet type's and front layout's front view as the core builds it.

    python3 -I scripts/make_manifest.py
"""
import json
from pathlib import Path

from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[1]
PKG = ROOT / 'plugin' / 'cabinet-studio'
VERSION_FILE = PKG / 'plugin.json'
CARCASS, FRONT, LINE, METAL = (214, 196, 168), (196, 160, 112), (52, 70, 91), (70, 72, 76)

# The Job setup's sections and the core option groups in each, in the order the dialog shows them
# (docs/PLAN.md, Job setup). "opens" stands for opening, handle range and handle place.
SETUP = [
    ('Fronts', ['door_style', 'overlay', 'opens', 'handle_size', 'handle_finish', 'handle_position',
                'sharknose_angle', 'sharknose_door_edge', 'front_grain']),
    ('Drawers and hinges', ['drawer_system', 'drawer_sides', 'drawer_frame', 'hinge_boss']),
    ('Carcass', ['top_joint', 'bottom_joint', 'back', 'fixings', 'biscuit', 'dowel', 'dowel_spacing',
                 'domino', 'fixing_density', 'carcass_thickness']),
    ('Base and wall', ['toe_kick', 'leg_fixing', 'wall_hanging']),
    ('Sizes', ['base_height', 'base_depth', 'wall_height', 'wall_depth', 'tall_height', 'tall_depth', 'detail']),
    ('Shop standard', ['save_shop']),
]


# The Cabinets panel, as `describe` in src/lib.rs declares it: SolidCraft refuses the plugin
# when the two differ, so `plugin test` catches a change made to one alone.
def panels():
    """The Cabinets and Boards panels, as `describe` in src/lib.rs declares them: SolidCraft refuses
    the plugin when the two differ, so `plugin test` catches a change made to one alone."""
    col = lambda key, label, unit=None: {'key': key, 'label': label, 'unit': unit}
    cabinets = {'id': 'cabinets', 'title': 'Cabinets', 'zones': ['right', 'bottom'], 'layout': {'root': 0, 'nodes': [
        {'widget': {'group': [1, 2, 3, 4, 5, 6]}},
        {'widget': {'toolbar': [{'label': 'Refresh', 'icon': None, 'entry': 'show'},
                                {'label': 'Rebuild all', 'icon': None, 'entry': 'rebuild'}]}},
        {'widget': {'table': {'bind': '/cabinets', 'row_id': 'id', 'columns': [
            col('id', 'Cabinet'), col('kind', 'Type'), col('size', 'W × H × D'), col('fronts', 'Fronts'), col('warnings', 'Warnings')],
            'select': 'single', 'on_select': 'select', 'on_edit': None}}},
        {'widget': {'text': 'Boards'}},
        {'widget': {'table': {'bind': '/job_boards', 'row_id': 'role', 'columns': [col('role', 'Role'), col('board', 'Board')],
                              'select': 'none', 'on_select': None, 'on_edit': None}}},
        {'widget': {'text': 'Warnings'}},
        {'widget': {'item-list': {'bind': '/warnings', 'on_select': None}}},
    ]}}
    return {'ui_version': 1, 'workspaces': [], 'panels': [cabinets]}


def board_choices():
    """The Boards dialog's choices from src/boards.tsv, each list sorted as the dialog shows it;
    the hook narrows them (src/lib.rs keeps the same order: SolidCraft takes only an ordered subset)."""
    # The first line is the brands' table of contents (see src/boards.rs).
    rows = [l.split('\t') for l in (ROOT / 'src' / 'boards.tsv').read_text().splitlines()[1:]]
    brands = []
    for r in rows:
        if r[1] not in brands:
            brands.append(r[1])
    ranges = sorted({r[2] for r in rows})
    decors = sorted({r[3] for r in rows})
    finishes = sorted({r[4] for r in rows if r[4]})
    thicknesses = sorted({float(r[5]) for r in rows})
    return brands, ranges, decors, finishes, thicknesses


def draw(view, path, size=256):
    """A front view as a picture: carcass, fronts and hardware, fitted with a margin."""
    rects = view['carcass'] + view['fronts'] + view['hardware']
    x0, z0 = min(r[0] for r in rects), min(r[1] for r in rects)
    x1, z1 = max(r[2] for r in rects), max(r[3] for r in rects)
    k = (size - 40) / max(x1 - x0, z1 - z0)
    ox, oz = (size - (x1 - x0) * k) / 2, (size - (z1 - z0) * k) / 2
    im = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)

    def box(r):
        return (ox + (r[0] - x0) * k, size - oz - (r[3] - z0) * k, ox + (r[2] - x0) * k, size - oz - (r[1] - z0) * k)

    for r in view['carcass']:
        d.rectangle(box(r), fill=CARCASS, outline=LINE, width=2)
    for r in view['fronts']:
        d.rectangle(box(r), fill=FRONT, outline=LINE, width=3)
    for r in view['hardware']:
        d.rectangle(box(r), fill=METAL)
    im.save(path, optimize=True)


def choice(const, title, **extra):
    return dict({'const': const, 'title': title}, **extra)


# What SolidCraft 0.50's dialog draws (crates/ui-egui/src/plugin_menu.rs, `form`): any other
# field turns the whole dialog off with "This entry needs to be run from the agent or CLI", which
# headless tests never see (0.4.0's Thickness, a dropdown of numbers, owner 2026-11-28).
DRAWN = {
    ('number', False): {'minimum', 'maximum', 'exclusiveMinimum', 'exclusiveMaximum', 'multipleOf'},
    ('integer', False): {'minimum', 'maximum', 'exclusiveMinimum', 'exclusiveMaximum'},
    ('boolean', False): set(),
    ('string', True): {'enum', 'oneOf', 'minLength', 'maxLength', 'format'},
    ('string', False): {'minLength', 'maxLength', 'format'},
}
PICK = {'type', 'title', 'description', 'minItems', 'maxItems', 'items', '$comment', 'deprecated', 'x-solidcraft-selection',
        'x-solidcraft-visible-when', 'x-solidcraft-enabled-when', 'x-solidcraft-section', 'x-solidcraft-help'}
NOTES = {'type', 'title', 'description', 'default', 'unit', 'x-unit', 'units', 'examples', '$comment', 'deprecated',
         'x-solidcraft-visible-when', 'x-solidcraft-enabled-when', 'x-solidcraft-defaults', 'x-solidcraft-grant',
         'x-solidcraft-section', 'x-solidcraft-unit-from', 'x-solidcraft-help'}


def check_drawable(entry, schema):
    for key, p in schema['properties'].items():
        if p.get('type') == 'array':
            extra = set(p) - PICK
            if extra:
                raise SystemExit(f"{entry}.{key}: SolidCraft's dialog can't draw this pick field ({sorted(extra)})")
            continue
        allowed = DRAWN.get((p.get('type'), 'enum' in p or 'oneOf' in p))
        extra = set(p) - NOTES - (allowed or set())
        if allowed is None or extra:
            raise SystemExit(f"{entry}.{key}: SolidCraft's dialog can't draw it ({p.get('type')}, {sorted(extra)})")


def main():
    cat = json.loads((ROOT / 'target' / 'catalog.json').read_text())
    pics = PKG / 'pictures'
    pics.mkdir(exist_ok=True)
    for old in pics.glob('*.png'):
        old.unlink()
    for k in cat['kinds']:
        draw(k['view'], pics / f"kind-{k['id']}.png")
    for p in cat['presets']:
        draw(p['view'], pics / f"fronts-{p['id']}.png")

    groups = {g['field']: g for g in cat['groups']}
    defaults = cat['defaults']

    def group_field(field, title=None, drop=()):
        g = groups[field]
        return {'type': 'string', 'title': title or g['title'], 'default': defaults[field],
                'oneOf': [choice(c['id'], c['label']) for c in g['choices'] if c['id'] not in drop]}

    def number(title, default, lo, hi, unit='mm'):
        return {'type': 'number', 'title': title, 'default': default, 'minimum': lo, 'maximum': hi, 'unit': unit}

    sizes = cat['sizes']
    setup_fields = {
        'opens': {'type': 'string', 'title': 'How fronts open', 'default': 'handle',
                  'oneOf': [choice(o['id'], o['label']) for o in cat['opens']]},
        'handle_size': group_field('handle_size', 'Handle', drop=('knob',)),
        'handle_position': {'type': 'string', 'title': 'Handle position', 'default': 'corner',
                            'oneOf': [choice('corner', 'By the corner'), choice('centre', 'Centred')]},
        'carcass_thickness': number('Carcass board', defaults['carcass_thickness'], 12, 32),
        'detail': {'type': 'string', 'title': 'Build in SolidCraft', 'default': 'design',
                   'oneOf': [choice('design', 'Design: panels and hardware'),
                             choice('manufacturing', 'Manufacturing: every hole and slot (slower)')]},
        'save_shop': {'type': 'boolean', 'title': 'Also save as my shop standard', 'default': False,
                      'description': 'New jobs start from it'},
    }
    for cls in ('base', 'wall', 'tall'):
        h, d = sizes[cls]
        setup_fields[f'{cls}_height'] = number(f'{cls.title()} height', h, 200, 2700)
        setup_fields[f'{cls}_depth'] = number(f'{cls.title()} depth', d, 100, 900)
    # Each size row of the core's (Cabinet Studio's chips) right after the picker it belongs under:
    # "How fronts open" stands for handle place, and the wardrobe's go with the sizes.
    under = {'handle_place': 'opens', 'sharknose_angle': 'sharknose_door_edge', 'wardrobe': 'tall_depth'}
    for meas in cat['measurements']:
        presets = ', '.join(f"{p:g}" for p in meas['presets'])
        setup_fields[meas['field']] = dict(number(meas['title'], defaults[meas['field']], 0, 1000),
                                           description=f'Usual: {presets} mm')
    # A field shows once: the rails a sharknose stack sits on are under Drawer frame.
    setup_fields['sharknose_gap']['description'] += '. The rails behind the fronts: under Drawers and hinges'
    order = [(s, list(f)) for s, f in SETUP]
    for meas in reversed(cat['measurements']):
        anchor = under.get(meas['after'], meas['after'])
        for _, fields in order:
            if anchor in fields:
                fields.insert(fields.index(anchor) + 1, meas['field'])
                break
    setup_props = {}
    for section, fields in order:
        for i, f in enumerate(fields):
            prop = dict(setup_fields.get(f) or group_field(f))
            if i == 0:
                prop['x-solidcraft-section'] = section
            setup_props[f] = prop

    kinds = [choice(k['id'], k['label'], **{'x-solidcraft-image': f"pictures/kind-{k['id']}.png"}) for k in cat['kinds']]
    fronts = [choice(p['id'], p['label'], **{'x-solidcraft-image': f"pictures/fronts-{p['id']}.png"}) for p in cat['presets']]
    side = [choice('left', 'Left'), choice('right', 'Right')]
    job = choice('job', 'As the job')
    cabinet_props = {
        'kind': {'type': 'string', 'title': 'Type', 'default': 'base', 'oneOf': kinds, 'x-solidcraft-section': 'Cabinet'},
        'width': number('Width', 600, 150, 2400),
        # The job's base sizes until the hook gives the type's; a cabinet keeps the sizes it was given.
        'height': number('Height', sizes['base'][0], 200, 2700),
        'depth': number('Depth', sizes['base'][1], 100, 900),
        'fronts': {'type': 'string', 'title': 'Fronts', 'default': 'doors', 'oneOf': fronts, 'x-solidcraft-section': 'Fronts'},
        'hinge': {'type': 'string', 'title': 'Hinged on the', 'default': 'left', 'oneOf': side,
                  'x-solidcraft-visible-when': {'field': 'fronts', 'eq': 'door'}},
        'shelves': {'type': 'integer', 'title': 'Shelves', 'default': 1, 'minimum': 0, 'maximum': 8},
        'pipe_cutout': {'type': 'boolean', 'title': 'Pipe cut-out in the back', 'default': True,
                        'x-solidcraft-section': 'Sink', 'x-solidcraft-visible-when': {'field': 'kind', 'eq': 'sink'}},
        'liner': {'type': 'boolean', 'title': 'Waterproof liner', 'default': True,
                  'x-solidcraft-visible-when': {'field': 'kind', 'eq': 'sink'}},
        'blind_side': {'type': 'string', 'title': 'Blind side', 'default': 'left', 'oneOf': side,
                       'x-solidcraft-section': 'Blind corner',
                       'x-solidcraft-visible-when': {'field': 'kind', 'eq': 'blind_corner'}},
        'door_style': {'type': 'string', 'title': 'Door style', 'default': 'job',
                       'oneOf': [job] + group_field('door_style')['oneOf'],
                       'x-solidcraft-section': 'Different from the job'},
        'opens': {'type': 'string', 'title': 'How fronts open', 'default': 'job',
                  'oneOf': [job] + setup_fields['opens']['oneOf']},
    }

    def schema(title, icon, props, required=()):
        s = {'type': 'object', 'title': title, 'x-solidcraft-icon': f'icons/{icon}.svg',
             'x-solidcraft-field-order': 'declared', 'x-solidcraft-sections': 'collapsible', 'properties': props}
        if required:
            s['required'] = list(required)
        return s

    def entry(name, menu, icon, props=None, effect='edit', ribbon=None, **kw):
        e = {'name': name, 'effect': effect, 'input_schema': schema(menu, icon, props or {}),
             'slots': [{'kind': 'menu', 'name': menu}] + ([{'kind': 'ribbon', 'name': ribbon}] if ribbon else [])}
        e.update(kw)
        return e

    brands, ranges, decors, finishes, thicknesses = board_choices()
    opts = lambda values: [{'const': v} for v in values]
    board_props = {
        'role': {'type': 'string', 'title': 'Use for', 'default': 'carcass', 'oneOf': [
            choice('carcass', 'Carcass'), choice('fronts', 'Fronts'), choice('backs', 'Backs'), choice('benchtop', 'Benchtop')]},
        'brand': {'type': 'string', 'title': 'Brand', 'default': 'Laminex', 'oneOf': opts(brands)},
        'range': {'type': 'string', 'title': 'Range', 'default': ranges[0], 'oneOf': opts(ranges)},
        'find': {'type': 'string', 'title': 'Find decor', 'default': '', 'description': 'Part of a decor name, to narrow the list'},
        'decor': {'type': 'string', 'title': 'Decor', 'default': decors[0], 'oneOf': opts(decors)},
        'finish': {'type': 'string', 'title': 'Finish', 'default': finishes[0],
                   'oneOf': [choice('', 'None')] + opts(finishes)},
        # Text, as SolidCraft's dialog draws a dropdown only of text ("18", shown "18 mm").
        'thickness': {'type': 'string', 'title': 'Thickness', 'default': '18',
                      'oneOf': [choice(f'{x:g}', f'{x:g} mm') for x in thicknesses]},
    }
    pick = {'type': 'array', 'title': 'Cabinet', 'description': 'Any part of the cabinet to change', 'minItems': 1,
            'x-solidcraft-selection': {'version': 1, 'accept': ['body']}}
    beside = {'type': 'array', 'title': 'Beside', 'maxItems': 1,
              'description': 'A cabinet to put the new one beside, on its right; none: at the end of the run',
              'x-solidcraft-selection': {'version': 1, 'accept': ['body']}}
    edit_props = {'pick': pick, **cabinet_props,
                  'remove': {'type': 'boolean', 'title': 'Remove this cabinet', 'default': False,
                             'x-solidcraft-section': ''}}
    add_props = {'beside': beside, **cabinet_props}
    edit = entry('edit', 'Edit cabinet', 'edit', edit_props, ribbon='Edit', dialog_hook=True, job=True)
    edit['input_schema']['required'] = ['pick']
    # Bench top: the run's pieces (PLUG-3). Each field shows for its piece.
    when = lambda *ps: {'x-solidcraft-visible-when': {'field': 'piece', 'in': list(ps)}}
    top = lambda **kw: dict(kw, **when('benchtop'))
    pick_field = lambda title, desc: {'type': 'array', 'title': title, 'description': desc, 'maxItems': 1,
                                      'x-solidcraft-selection': {'version': 1, 'accept': ['body']}}
    bench_props = {
        'piece': {'type': 'string', 'title': 'Piece', 'default': 'benchtop', 'oneOf': [
            choice('benchtop', 'Benchtop, splashback and cut-outs'), choice('end-panel', 'End panel'), choice('filler', 'Filler')]},
        'beside': dict(pick_field('Beside', 'The cabinet it goes beside; to remove one, pick the panel itself'),
                       **when('end-panel', 'filler')),
        'side': {'type': 'string', 'title': 'On its', 'default': 'right', 'oneOf': [choice('left', 'Left'), choice('right', 'Right')],
                 **when('end-panel', 'filler')},
        'width': dict(number('Filler width', 50, 5, 300), **when('filler')),
        'scribe': dict(number('Scribe', 0, 0, 50), description='Extra to cut to the wall on site', **when('end-panel', 'filler')),
        'thickness': top(**number('Thickness', 0, 0, 100), description="0: the benchtop board's, or 33 mm"),
        'front_overhang': top(**number('Front overhang', 20, 0, 400)),
        'side_overhang': top(**number('Side overhang', 0, 0, 400)),
        'edge': top(type='string', title='Edge', default='square', oneOf=[choice(e['id'], e['label']) for e in cat['edges']]),
        'splashback': top(**number('Splashback height', 0, 0, 1200), description='0: none. In the benchtop board'),
        'splashback_thickness': top(**number('Splashback thickness', 20, 0, 50)),
        'sink': top(type='string', title='Sink', default='none',
                    oneOf=[choice('none', 'None'), choice('custom', 'A cut-out of my size')]
                          + [choice(s['id'], s['name']) for s in cat['sinks']]),
        'sink_over': dict(pick_field('Sink over', 'The cabinet the sink sits over'),
                          **{'x-solidcraft-visible-when': {'all': [{'field': 'piece', 'eq': 'benchtop'}, {'not': {'field': 'sink', 'eq': 'none'}}]}}),
        'sink_width': dict(number('Sink cut-out width', 760, 100, 1500),
                           **{'x-solidcraft-visible-when': {'all': [{'field': 'piece', 'eq': 'benchtop'}, {'field': 'sink', 'eq': 'custom'}]}}),
        'sink_depth': dict(number('Sink cut-out depth', 450, 100, 700),
                           **{'x-solidcraft-visible-when': {'all': [{'field': 'piece', 'eq': 'benchtop'}, {'field': 'sink', 'eq': 'custom'}]}}),
        'cooktop': top(type='string', title='Cooktop', default='none', oneOf=[choice('none', 'None'), choice('cutout', 'A cut-out of my size')]),
        'cooktop_over': dict(pick_field('Cooktop over', 'The cabinet the cooktop sits over'),
                             **{'x-solidcraft-visible-when': {'all': [{'field': 'piece', 'eq': 'benchtop'}, {'field': 'cooktop', 'eq': 'cutout'}]}}),
        'cooktop_width': dict(number('Cooktop cut-out width', 560, 100, 1200),
                              **{'x-solidcraft-visible-when': {'all': [{'field': 'piece', 'eq': 'benchtop'}, {'field': 'cooktop', 'eq': 'cutout'}]}}),
        'cooktop_depth': dict(number('Cooktop cut-out depth', 490, 100, 700),
                              **{'x-solidcraft-visible-when': {'all': [{'field': 'piece', 'eq': 'benchtop'}, {'field': 'cooktop', 'eq': 'cutout'}]}}),
        'remove': {'type': 'boolean', 'title': 'Remove it', 'default': False, 'x-solidcraft-section': ''},
    }
    make = {'type': 'string', 'title': 'Make', 'default': 'csv', 'oneOf': [
        choice('csv', 'Cut list (CSV)'), choice('sheets', 'Cut sheets'), choice('labels', 'Part labels')]}
    entries = [
        entry('setup', 'Job setup', 'job', setup_props, dialog_hook=True, job=True),
        entry('show', 'Cabinets panel', 'panel'),
        entry('rebuild', 'Rebuild all', 'panel', job=True),
        entry('select', 'Select cabinet', 'panel', {'row': {'type': 'string', 'title': 'Cabinet', 'default': ''}}),
        entry('add', 'Add cabinet', 'add', add_props, ribbon='Add', dialog_hook=True, job=True),
        edit,
        entry('boards', 'Boards', 'boards', board_props, ribbon='Boards', dialog_hook=True, job=True),
        entry('benchtop', 'Bench top and panels', 'benchtop', bench_props, ribbon='Bench top', dialog_hook=True, job=True),
        entry('export', 'Cut list', 'cutlist', {'make': make}, ribbon='Cut List'),
    ]
    old = json.loads(VERSION_FILE.read_text())
    commands = sorted(set(old['runtime']['commands']) | {'select.set', 'component.list'})
    manifest = {
        'id': 'cabinet-studio', 'name': 'Cabinet Studio', 'version': old['version'], 'updates': old['updates'],
        'description': old['description'],
        'capabilities': ['commands@0.2', 'ui@0.1', 'storage@0.1', 'dialog@0.1', 'user-storage@0.1'],
        'runtime': {
            'module': 'cabinet-studio.wasm', 'data_format': 1, 'commands': commands, 'entries': entries,
            'ribbon_groups': [{'label': 'Job', 'icon': 'icons/job.svg', 'entries': ['setup', 'show', 'rebuild', 'select']}],
            'permissions': {'files': [{'name': 'exports', 'write': True}]},
            'ui': panels(),
            'limits': old['runtime']['limits'],
        },
    }
    for e in entries:
        check_drawable(e['name'], e['input_schema'])
    text = json.dumps(manifest, separators=(',', ':'), ensure_ascii=False) + '\n'
    VERSION_FILE.write_text(text)
    print(f'plugin.json: {len(text.encode())} bytes, {len(list(pics.glob("*.png")))} pictures')


if __name__ == '__main__':
    main()
