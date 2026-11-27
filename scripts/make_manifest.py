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
    ('Base and wall', ['toe_kick', 'toe_kick_height', 'leg_fixing', 'wall_hanging']),
    ('Sizes', ['base_height', 'base_depth', 'wall_height', 'wall_depth', 'tall_height', 'tall_depth', 'detail']),
    ('Shop standard', ['save_shop']),
]


# The Cabinets panel, as `describe` in src/lib.rs declares it: SolidCraft refuses the plugin
# when the two differ, so `plugin test` catches a change made to one alone.
PANEL = {'ui_version': 1, 'workspaces': [], 'panels': [{
    'id': 'cabinets', 'title': 'Cabinets', 'zones': ['right', 'bottom'],
    'layout': {'root': 0, 'nodes': [
        {'widget': {'group': [1, 2, 3, 4]}},
        {'widget': {'toolbar': [{'label': 'Refresh', 'icon': None, 'entry': 'show'},
                                {'label': 'Rebuild all', 'icon': None, 'entry': 'rebuild'}]}},
        {'widget': {'table': {'bind': '/cabinets', 'row_id': 'id', 'columns': [
            {'key': 'id', 'label': 'Cabinet', 'unit': None}, {'key': 'kind', 'label': 'Type', 'unit': None},
            {'key': 'size', 'label': 'W × H × D', 'unit': None}, {'key': 'fronts', 'label': 'Fronts', 'unit': None},
            {'key': 'warnings', 'label': 'Warnings', 'unit': None}],
            'select': 'single', 'on_select': 'select', 'on_edit': None}}},
        {'widget': {'text': 'Warnings'}},
        {'widget': {'item-list': {'bind': '/warnings', 'on_select': None}}},
    ]}}]}


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
        'toe_kick_height': number('Kick height', defaults['toe_kick_height'], 50, 250),
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
    setup_props = {}
    for section, fields in SETUP:
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
        'height': dict(number('Height', 0, 0, 2700), description="0: the job's height for its type"),
        'depth': dict(number('Depth', 0, 0, 900), description="0: the job's depth for its type"),
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
    piece = {'type': 'string', 'title': 'Piece', 'default': 'benchtop', 'oneOf': [
        choice('benchtop', 'Benchtop'), choice('end-panel', 'End panel'), choice('filler', 'Filler'),
        choice('splashback', 'Splashback')]}
    make = {'type': 'string', 'title': 'Make', 'default': 'csv', 'oneOf': [
        choice('csv', 'Cut list (CSV)'), choice('sheets', 'Cut sheets'), choice('labels', 'Part labels')]}
    entries = [
        entry('setup', 'Job setup', 'job', setup_props, dialog_hook=True, job=True),
        entry('boards', 'Boards', 'job'),
        entry('show', 'Cabinets panel', 'panel'),
        entry('rebuild', 'Rebuild all', 'panel', job=True),
        entry('select', 'Select cabinet', 'panel', {'row': {'type': 'string', 'title': 'Cabinet', 'default': ''}}),
        entry('add', 'Add cabinet', 'add', add_props, ribbon='Add', dialog_hook=True, job=True),
        edit,
        entry('benchtop', 'Bench top and panels', 'benchtop', {'piece': piece}, ribbon='Bench top'),
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
            'ribbon_groups': [{'label': 'Job', 'icon': 'icons/job.svg', 'entries': ['setup', 'boards', 'show', 'rebuild', 'select']}],
            'permissions': {'files': [{'name': 'exports', 'write': True}]},
            'ui': PANEL,
            'limits': old['runtime']['limits'],
        },
    }
    text = json.dumps(manifest, separators=(',', ':'), ensure_ascii=False) + '\n'
    VERSION_FILE.write_text(text)
    print(f'plugin.json: {len(text.encode())} bytes, {len(list(pics.glob("*.png")))} pictures')


if __name__ == '__main__':
    main()
