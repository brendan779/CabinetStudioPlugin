# What to ask SolidCraft for

The limits and bugs this plugin met in SolidCraft 0.50.0 (2026-11-27 and 28), each with what
happened, how to see it, and what would fix it. To post on tangency-labs/solidcraft one issue each,
only with the owner's go-ahead, after searching for an existing issue. The plugin's workaround is
in [PLAN.md](PLAN.md).

## Already on SolidCraft's tracker

- **A ribbon button enabled by the selection:** posted as tangency-labs/solidcraft#23098.
- **Pictures in panel rows:** tangency-labs/solidcraft#20553 (open), our use added as a comment.
- **A filled-in form from a panel event:** tangency-labs/solidcraft#22032, fixed by #22044 after
  0.50.0.
- **Panel-only entries on the ribbon:** tangency-labs/solidcraft#22060 and #22300 (open).

## Bugs

### 1. The window stops resizing after a plugin panel is resized

- **Seen:** the owner widened the plugin's Boards panel (docked at the bottom, a six-column table)
  and changed the window's size; the panel shrank, though set to its expanded view, and the window
  would no longer resize.
- **To see it:** a plugin panel docked at the bottom with a wide table (Cabinet Studio 0.3.0's
  Boards panel); resize the panel, then the window.
- **Ask:** a panel's content shouldn't decide the window's minimum size.

### 2. `plugin.install` through the control port doesn't finish on the desktop

- **Seen:** `engine.execute` `plugin.install` with `dry_run: true` (and then the real install, with
  `replace: true`, an `approve` object copying the installed grants, and `allow_paths` for the
  zip's folder) answers "still running" after 60 s; `control.pending` lists a `plugin_package`
  background job that runs for its full 300 s and is abandoned. Nothing appears in the window.
- **Ask:** finish, or say what it is waiting for. It's the way to update a plugin under
  development without the dialog.

### 3. A dialog with a field it can't draw gives a message that doesn't name the field

- **Seen:** Cabinet Studio 0.4.0's Boards dialog had a number field with `oneOf` choices; the
  whole dialog was off with "This entry needs to be run from the agent or CLI". `plugin check`
  reported no problem.
- **Ask:** `plugin check` warning that an entry's field can't be drawn, and the dialog naming the
  field and why. Better still, the next item.

### 4. A rejected hook reply is only in the log

- **Seen:** a `dialog@0.1` hook reply whose `choices` weren't in the manifest's order was dropped
  whole; the dialog said only "Plugin field update unavailable. Using the declared fields.", and
  the reason ("hook choices must be an ordered subset of schema consts") was only in the run's log.
  `plugin test` passed.
- **Ask:** the reason in the dialog's note, and `plugin test` failing a case whose hook reply was
  ignored.

## Limits that shaped the plugin

### 5. Dropdowns of numbers

- A `number` or `integer` field with `enum` or `oneOf` can't be drawn (see 3). A board's thickness
  (16, 18, 33 mm) is a choice of numbers; the plugin sends text and converts it.
- **Ask:** numeric choices, with the field's `unit`.

### 6. A panel can't start closed

- Every declared panel opens when the plugin loads (`sync_visibility`: a panel with no instance
  opens). The owner wanted the Boards panel minimised until asked for.
- **Ask:** an initial state in `PanelSpec` (`"initially": "closed"` or `"collapsed"`).

### 7. A ribbon button can't show a panel, and a field-less entry still opens a dialog

- Clicking a ribbon entry always opens its dialog, even with no fields (an empty dialog and Run).
  The owner expected Boards to open its panel, and Cabinets panel and Rebuild all to just run.
- **Ask:** an entry that runs on click when it has no fields (or says so in the manifest), and a
  ribbon slot that shows or toggles one of the plugin's panels.

### 8. A panel's form can't keep its own values

- A form field shows what the plugin last wrote at its path, and a change event carries only that
  field's value. To filter by role, brand and search together, the plugin had to keep the other
  values elsewhere; it used `user-storage`, and quick changes (a keystroke each) collided, so the
  dropdown snapped back and the list didn't refresh.
- **Ask:** a change event carrying the whole form's values, or a host call to read the panel's
  data.

### 9. Catalogue-sized choices

- Choices must be written in the manifest (64 KiB), and a hook can only narrow them, to at most
  256, in the manifest's order. A board catalogue (3,049 boards, 741 decor names) only just fits as
  names, and a long range still has more than 256 decors.
- **Ask:** choices a hook (or a read entry) supplies, not only narrows, or a searchable picker the
  plugin feeds; and a higher limit.

### 10. A dialog hook's budget is fixed

- 10 million fuel, 16 MiB, 250 ms, whatever the manifest asks. Reading a 365 KB board index once
  was too much; the plugin had to split it by brand.
- **Ask:** a hook budget the manifest can ask for and the user approve, like an entry's.

### 11. Ribbon order and labels

- `ribbon_groups` drop-downs always come before single buttons, whatever the manifest's order, so
  a workflow's order (Job ▾, Add, Edit, Boards...) can't be kept with more than one drop-down.
- A drop-down's label shows about 5 letters ("Bench…"), a single button's about 8, then they're
  cut; two short words wrap.
- **Ask:** buttons and drop-downs in the manifest's order; labels that wrap or show in full as a
  tooltip.

### 12. A whole kitchen doesn't fit an ordinary run

- Plugins have no `engine.batch`, so each command is one host call, and an ordinary run allows 256.
  One base cabinet at design detail is about 50 commands, at manufacturing detail about 200. Adding
  or editing a cabinet runs as a long job, which has no preview.
- **Ask:** `engine.batch` (or `engine.script`) for plugins as one host call, or previews for long
  jobs with a lower budget.

### 13. No way to tell a preview from a run

- `plugin.preview` is in docs/plugins.md on main but not in 0.50.0, so `plugin check` refuses a
  manifest listing it.
- **Ask:** ship it (it lets a preview build less).

### 14. A hook's hints don't show

- A `dialog@0.1` reply's `hint` on a field (Boards' "Carcass now: …", Decor's "133 decors") showed
  nowhere in the 0.50.0 dialog that took the same reply's choices and values.
- **Ask:** show a field's hint under it, or say in docs/plugins.md where it goes.

### 16. One-run folder access can't be had with panels or dialog hooks

- A plugin that declares `folders@0.1` and builds against its own world (`include
  panel-dialog-plugin; import folders;`, as docs/plugins.md suggests composing) passes `plugin check`,
  but every entry then fails at run time with "function implementation is missing": the host links
  `folders` only into `command-with-folders` and `importer-with-folders`.
- **Ask:** link `folders` into any command world that asks for it (or ship worlds with panels and
  hooks that include it), and have `plugin check` refuse a world the host can't link.

### 17. The docs disagree on the manifest's limit

- plugins.md says 64 KiB; plugin-runtime.md says 16 KiB, and the writing guide's Limits table says
  16 KiB. 0.50.0 accepted 47 KB.
- **Ask:** one figure in the docs.
