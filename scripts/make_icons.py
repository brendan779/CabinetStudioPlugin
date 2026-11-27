#!/usr/bin/env python3
"""Draws the plugin's ribbon and entry icons into plugin/cabinet-studio/icons, in SolidCraft's icon
style: isometric, grey faces for what exists, blue for what the tool makes, an ink outline. Only
the palette's source colours, so the app recolours them for its themes (SolidCraft's
crates/look/icons/design/palette.toml; the icon rules are in its docs/plugins.md, Icons and pictures).

    python3 scripts/make_icons.py
"""
from pathlib import Path

OUT = Path(__file__).resolve().parents[1] / 'plugin' / 'cabinet-studio' / 'icons'
INK, P1, P2, P3 = '#34465b', '#e2e8ef', '#b8c3cf', '#8e9faf'
A1, A2, A3 = '#b9ddf8', '#328bd3', '#236ead'
STROKE = 'stroke="#34465b" stroke-width="2.55" stroke-linejoin="round" stroke-linecap="round"'
C, S = 0.8660254, 0.5


def iso(o, k):
    """A projection: width runs down-right, depth up-right, height up; `o` the front-left-bottom
    corner on screen, `k` pixels per unit."""
    def p(x, y, z):
        return (o[0] + k * C * (x + y), o[1] + k * (S * x - S * y - z))
    return p


def poly(pts, fill):
    d = 'M' + 'L'.join(f'{x:.2f} {y:.2f}' for x, y in pts) + 'Z'
    return f'  <path d="{d}" fill="{fill}"/>'


def line(pts, closed=False):
    d = 'M' + 'L'.join(f'{x:.2f} {y:.2f}' for x, y in pts) + ('Z' if closed else '')
    return f'  <path d="{d}" fill="none" {STROKE}/>'


def block(p, x0, y0, z0, w, d, h, front, side, top):
    """A box's three visible faces and its outline."""
    f = [p(x0, y0, z0), p(x0 + w, y0, z0), p(x0 + w, y0, z0 + h), p(x0, y0, z0 + h)]
    r = [p(x0 + w, y0, z0), p(x0 + w, y0 + d, z0), p(x0 + w, y0 + d, z0 + h), p(x0 + w, y0, z0 + h)]
    t = [p(x0, y0, z0 + h), p(x0 + w, y0, z0 + h), p(x0 + w, y0 + d, z0 + h), p(x0, y0 + d, z0 + h)]
    out = [poly(f, front), poly(r, side), poly(t, top)]
    hull = [p(x0, y0, z0), p(x0 + w, y0, z0), p(x0 + w, y0 + d, z0), p(x0 + w, y0 + d, z0 + h),
            p(x0, y0 + d, z0 + h), p(x0, y0, z0 + h)]
    out += [line(hull, True), line([p(x0, y0, z0 + h), p(x0 + w, y0, z0 + h), p(x0 + w, y0 + d, z0 + h)]),
            line([p(x0 + w, y0, z0), p(x0 + w, y0, z0 + h)])]
    return out


def svg(title, parts):
    return ('<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 64 64">\n'
            f'  <title>{title}</title>\n' + '\n'.join(parts) + '\n</svg>\n')


def cabinet(p, front=A2, side=A3, top=A1, doors=True):
    """A base cabinet 1 wide, 0.75 deep and 1 high: its doors and handles drawn on the front."""
    out = block(p, 0, 0, 0, 1, 0.75, 1, front, side, top)
    if doors:
        out.append(line([p(0.5, 0, 0.04), p(0.5, 0, 0.96)]))
        out.append(line([p(0.42, 0, 0.62), p(0.42, 0, 0.84)]))
        out.append(line([p(0.58, 0, 0.62), p(0.58, 0, 0.84)]))
    return out


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    icons = {}
    # Cabinet: the thing the button makes, all blue.
    icons['cabinet'] = svg('Cabinet', cabinet(iso((12, 41), 27)))
    # Benchtop and panels: a grey cabinet with a blue top over it.
    p = iso((11, 47), 23)
    icons['benchtop'] = svg('Benchtop and panels', cabinet(p, P2, P3, P1)
                            + block(p, -0.1, -0.1, 1, 1.2, 0.95, 0.28, A2, A3, A1))
    # Job: a grey cabinet and a blue settings dial.
    p = iso((4, 37), 23)
    dial = ['  <circle cx="46" cy="45" r="10" fill="#328bd3"/>',
            '  <circle cx="46" cy="45" r="10" fill="none" ' + STROKE + '/>',
            '  <circle cx="46" cy="45" r="4" fill="#e2e8ef"/>',
            '  <circle cx="46" cy="45" r="4" fill="none" ' + STROKE + '/>']
    for x0, y0, x1, y1 in [(46, 31, 46, 34), (46, 56, 46, 59), (32, 45, 35, 45), (57, 45, 60, 45)]:
        dial.append(line([(x0, y0), (x1, y1)]))
    icons['job'] = svg('Job setup', cabinet(p, P2, P3, P1) + dial)
    # Cabinets panel: a list of cabinets, the first one picked.
    rows = []
    for i, y in enumerate((10, 26, 42)):
        rows.append(poly([(6, y), (58, y), (58, y + 12), (6, y + 12)], A1 if i == 0 else P1))
        rows.append(poly([(10, y + 3), (16, y + 3), (16, y + 9), (10, y + 9)], A2 if i == 0 else P3))
        rows.append(line([(6, y), (58, y), (58, y + 12), (6, y + 12)], True))
        rows.append(line([(22, y + 6), (50, y + 6)]))
    icons['panel'] = svg('Cabinets', rows)
    # Cut list: a sheet with blue parts nested on it.
    sheet = [poly([(5, 12), (59, 12), (59, 52), (5, 52)], P1),
             poly([(9, 16), (33, 16), (33, 48), (9, 48)], A2),
             poly([(37, 16), (55, 16), (55, 30), (37, 30)], A1),
             poly([(37, 34), (55, 34), (55, 48), (37, 48)], A1),
             line([(5, 12), (59, 12), (59, 52), (5, 52)], True),
             line([(9, 16), (33, 16), (33, 48), (9, 48)], True),
             line([(37, 16), (55, 16), (55, 30), (37, 30)], True),
             line([(37, 34), (55, 34), (55, 48), (37, 48)], True)]
    icons['cutlist'] = svg('Cut list', sheet)
    for name, text in icons.items():
        (OUT / f'{name}.svg').write_text(text)
    print(', '.join(sorted(icons)))


if __name__ == '__main__':
    main()
