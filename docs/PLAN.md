# The SolidCraft plugin: workflow and interface

Plan of record for PLUG-0 to PLUG-6, also listed in Cabinet Studio's backlog
([CabinetForSolidcraft/docs/BACKLOG.md](https://github.com/brendan779/CabinetForSolidcraft/blob/main/docs/BACKLOG.md)).
Written 2026-11-27 against SolidCraft 0.50.0. This repo is the plugin; the cabinet rules stay in
[CabinetForSolidcraft](https://github.com/brendan779/CabinetForSolidcraft) (owner 2026-11-27). The owner's brief:

- The interface comes first: a simple, logical workflow, with fewer options than Cabinet Studio,
  each in one obvious place. Cabinet Studio's inspector is "a bit disjointed in the amount of
  options and their locations relative to each other".
- Boards can be text or simple swatches. Benchtops can be simplified. Handles and leg plinths can be
  simple shapes if meshes can't be brought in. The splashback is parametric only.
- No Blender or Revit for now.

## What carries over and what doesn't

The plugin is a thin shell around the crates we already have. `cabinet-core`, `cabinet-catalog`,
`cabinet-sheets` and `solidcraft-bridge` all compile for `wasm32-unknown-unknown` unchanged
(checked 2026-11-27). The Blum, Häfele and ABI data is built into the core with `include_str!`.

- **Shared, never copied:** cabinet rules, joinery, hardware, the part list, cut list, nesting, cut
  sheets, and the commands that build a cabinet in SolidCraft. A fix lands once and both the app and
  the plugin get it. The plugin crate never holds a cabinet rule. If it needs one, the rule goes
  into the core with its test.
- **New, in this repo:** the WebAssembly component (`src/`: entries, dialog hooks, panel data,
  design storage) and the package in `plugin/cabinet-studio/` (the manifest, icons, option
  pictures, `tests/*.json`). `Cargo.toml` pins Cabinet Studio's crates to a commit; taking a core
  fix is moving that pin.
- **Stays in Cabinet Studio:** Blender, Revit, material packs, quotes, the elevation, lighting and
  lift systems (on hold anyway). The app keeps working throughout and is where interface ideas are
  tried first while the plugin catches up.

## What SolidCraft 0.50 gives an interface

| Piece | What it can do | What it can't |
|---|---|---|
| Ribbon | Buttons in the Plugins group, labelled with each slot's `name`, in our order. `ribbon_groups` puts several entries behind one drop-down button. Every entry gets a button | Our own ribbon tab, or a workspace, or entries with no button |
| Dialogs (an entry's form) | Sections that fold, declared field order, fields shown or enabled by other fields, choices narrowed by other fields, pictures on choices (`x-solidcraft-image`, PNG up to 512 px), swatch grids with search, viewport pick fields, live preview on a scratch copy, a hook that computes defaults and hints from the design | Choices that aren't written in the manifest (a hook can only narrow them), pictures outside a dropdown, more than 64 KiB of manifest |
| Re-editing | `forms.open` reopens an entry's form filled with stored values: Edit is the same dialog as Add | — |
| Docked panel | Up to 8 panels, right or bottom, with tabs, splits, forms, toolbars, buttons, lists and tables up to 100,000 rows. Buttons and row clicks run our entries | Images, trees, charts (phase 3), refreshing itself when the design changes: it needs a Refresh button or one of our entries to run |
| Storage | `storage@0.1` keeps the job in the design. `user-storage@0.1` keeps the shop's own standard on the Mac (64 KiB) | — |
| Long jobs | 10 minutes, a million engine calls, progress and Cancel | Preview |
| Files | Writes into folders the user grants at install | Network, other programs |

What this means for the design:

1. **The catalogue can't be a dropdown.** 769 decors and 3,049 boards don't fit in a 64 KiB
   manifest or in 256 swatches. Boards are picked in a searchable table in the panel, then given a
   job role. The dialogs only refer to roles ("Fronts board"), which are a short, fixed list.
2. **Shapes are pictures and data is text.** Cabinet types, front layouts, how fronts open, edge
   profiles and corner shapes are picture dropdowns. Boards, sinks and hardware articles are text.
3. **One dialog per thing, reopened to edit.** Add and Edit are the same form, so an option is
   always where it was last time.

## The workflow

Five steps, in the order a cabinetmaker works. The ribbon buttons and the panel's tabs follow
this order, left to right.

```text
 1 Set up the job ─▶ 2 Lay out cabinets ─▶ 3 Add the run's pieces ─▶ 4 Check ─▶ 5 Make it
   Job setup          Cabinet                Benchtop, Panel,         Panel:      Panel: Cut list
   Boards tab         From boxes             Splashback               Job tab     Export
```

1. **Set up the job, once.** Boards for carcass, fronts and backs (Boards tab), and the job
   standard (Job setup): construction, fronts and hardware, the base and wall, standard sizes. A
   new job starts from the shop standard, so most jobs skip this step.
2. **Lay out the cabinets.** Add each cabinet beside the one picked, or draw boxes and turn them
   into cabinets. A cabinet asks only for its type, size and fronts.
3. **Add the run's pieces:** benchtop, end panels and fillers, splashback. Kick and plinth come
   with each run from the job setup.
4. **Check** in the panel's Job tab: every cabinet in a table with its warnings. Click a row to
   edit that cabinet.
5. **Make it:** the cut list, sheets and hardware in the panel, exported to a folder.

Going back is always the same move: pick the thing in the viewport or in the panel's table, and
its dialog opens filled in.

## Where every decision lives

The rule: **each option is decided at one level, by whoever decides it in a real job.**

- **Job:** everything a cabinetmaker sets once per job, in Job setup and the Boards tab.
- **Run:** what belongs to a row of cabinets: benchtop, kick, end panels.
- **Cabinet:** only what differs from cabinet to cabinet: type, size, fronts, and a few type
  extras.

A cabinet can differ from the job in only four things, kept in one folded section: door style,
how its fronts open, fronts board and hinge side. Everything else is job-only in the plugin. In
Cabinet Studio, every construction option can also be set per cabinet with the scope switch. That
doubles every option's location and is the main source of the disjointed feel.

### Job setup (one dialog, five folding sections)

| Section | Fields (pictures marked ▣) | Replaces in Cabinet Studio |
|---|---|---|
| **Boards** | Carcass, Fronts, Backs, Benchtop: read-only text showing the job's choices, with a hint "Change in the panel's Boards tab". Edge tape: same as board / contrasting (text) | Build tab › Boards, Boards workspace |
| **Fronts** | Door style ▣ (slab, shaker, glass). Door mounting ▣ (full, half, inset). **How fronts open** ▣ (handle, knob, edge pull, finger pull, sharknose, TIP-ON, SERVO-DRIVE). Shown only when they apply: handle size and finish (handle, knob), handle position ▣ (corner, centre), bevel ▣ and door edge ▣ (sharknose). Front grain ▣ | Fronts tab: Opening, Handle range, Handle, Handle finish, Handle place, Sharknose bevel, Sharknose on doors, Door style, Door mounting, Front grain |
| **Drawers and hinges** | Drawer system ▣ (LEGRABOX, MERIVOBOX, TANDEMBOX, MOVENTO, TANDEM). Sides (when it has a choice). Drawer frame. Hinge fixing | Fittings tab: Drawers, Drawer sides, Insert design, Hinge fixing; Fronts tab: Drawer frame |
| **Carcass** | Top ▣, bottom ▣, back ▣. Joinery ▣ (biscuit + screw, biscuits, screws, confirmat, dowels, cam lock, DOMINO), with its size and spacing shown only for the method chosen. Carcass thickness | Build tab: Top, Bottom, Back, Fixings, Dowel size, Dowel spacing, DOMINO size, Biscuit size, Fixing spacing |
| **Base, wall and sizes** | Base ▣ (legs and plinth, notched sides, none) and kick height. Wall hanging ▣. Standard height and depth for base, wall and tall cabinets. Benchtop height. Cut list: saw kerf and sheet trim | Cabinet tab: Base, Leg fixing, Wall hanging; Preferences; Cut List settings |

**How fronts open** merges four of Cabinet Studio's pickers (Opening, Handle range, Handle place
and the sharknose pickers) into one picture choice. That's why the sharknose was hard to find: it
was the fifth tile of the third picker, shown only after the first one was set to Handles.

Two buttons at the foot of the dialog: **Save as shop standard** (to user storage, so new jobs
start from it) and **Use shop standard**. Applying the setup rebuilds every cabinet as a long job,
with progress and Cancel. A long job has no preview, so the dialog shows what will change as text
(the hook's hints: "12 cabinets rebuild; hinges change to EXPANDO").

### Add cabinet and Edit cabinet (the same fields)

```text
┌ Edit cabinet ────────────────────────────────────────┐
│ Cabinet  B03 · Base 600   "Changing B03"            │  Edit: the pick, required
│                                                      │
│ Type     ▣ Base  ▣ Drawers  ▣ Sink  ▣ Corner  ...    │  picture grid
│ Size     Width 600   Height job (720)   Depth job    │  job values unless typed
│ Fronts   ▣ 1 door ▣ 2 doors ▣ drawer+door ▣ 3 drw …  │  only layouts the type allows
│          Hinges on  ▣ left ▣ right                    │  1 door only
│          Shelves 1                                    │
│ ▸ Sink   pipe cut-out ✓   liner ✓                     │  shown for the type only
│ ▸ Different from the job (none)                       │  folded: 4 fields
│                                   ☑ Preview   [ Run ] │
└──────────────────────────────────────────────────────┘
```

- **Type:** base, drawers, sink, blind corner, L-corner, wall, wall corner, tall, wardrobe,
  rangehood, open shelf. Pictures drawn from the cabinet itself.
- **Two buttons, one set of fields** (owner 2026-11-27). **Edit cabinet** starts with the pick
  and fills in the rest from it. **Add cabinet** has no pick field to need: a cabinet picked when
  it opens is where the new one goes, with **Beside it** (on the side picked) or **Above it** (a
  wall cabinet over a base); with nothing picked, it goes at the end of the run.
- **Fronts:** presets drawn as pictures, narrowed by type (a drawer bank offers drawer stacks, a
  wall cabinet offers doors and lift-ups when LIFT-HOLD ends). Finer splits are a later option
  (SEC-*).
- **Type extras:** one folding section, only for its type: sink (pipe cut-out, liner), blind
  corner (blind side and width), L-corner (the two depths), wardrobe (hanging, shelves and drawers
  presets), rangehood (insert).
- **Different from the job:** door style, how fronts open, fronts board (a role or "as the job")
  and hinge side. Each starts at "As the job".

That is at most 9 visible fields. Height and depth are "job" until typed, so most cabinets are a
type, a width and a front layout: three clicks.

### From boxes

Draw boxes for the room layout with SolidCraft's own tools, pick them, then **From boxes**. The
dialog asks only for a type for all of them (or "guess from height": base, wall or tall) and
builds a cabinet in each box (SC-2's logic). Then edit cabinets one by one. This is our fastest
route for a whole kitchen, and Woodwork doesn't have it.

### The run's pieces

All simplified for the plugin's first version, as agreed:

- **Benchtop:** pick the cabinets (or one, and the run is found). Material: laminate, stone or
  timber (text). Thickness, front and side overhangs, edge profile ▣ (square, pencil round,
  bullnose, bevel). Cut-outs: a sink from the ABI list (text, the 16 sinks), or a size, and a
  cooktop size. No slabs, joins, veins, mitred aprons or waterfalls yet.
- **Panel:** pick a cabinet's side. End panel ▣, filler ▣ or seen end ▣, its width and board role.
- **Splashback:** pick the benchtop. Height and thickness, board role. Parametric only.
- **Kick and plinth:** automatic for each run, from Job setup.
- **Hardware: everything is in the plugin (owner 2026-11-27).** No supplier CAD folder: hinges,
  runners, legs, handles, lift and push-to-open hardware are placeholders built from SolidCraft
  commands at their catalog sizes and positions, and every hole they need is drilled exactly from
  the catalog data. The core already draws placeholders for hinges and plates, runners, legs,
  TIP-ON, SERVO-DRIVE, tracks and cams; a handle placeholder (a bar on two posts, or a knob, at the
  handle's real centres and projection) is added to the core in PLUG-1. Embedded STEP models were
  considered, but a plugin can only insert a STEP from a folder the user grants, so placeholders
  from commands need no grant and cost a few engine calls each.

### The panel: "Cabinets", docked right

One panel with four tabs, in workflow order. A table row click selects that thing in the viewport.
A double-click opens its dialog.

| Tab | Shows | Toolbar |
|---|---|---|
| **Job** | Table: id, type, width × height × depth, fronts, warnings count. Below it, the warnings list (empty state "No problems found") | Add, Edit, From boxes, Rebuild all, Refresh |
| **Boards** | The job's boards on top (role, brand, decor, finish, thickness). Below, search and brand fields and the catalogue table (brand, range, decor, finish, thickness, sheet) | Use for carcass, Use for fronts, Use for backs, Use for benchtop |
| **Cut list** | Table: part, board, length × width × thickness, quantity, edges, grain | Export folder… then Cut list CSV, Cut sheets (SVG/PDF), DXF per sheet, Labels |
| **Hardware** | Table: item, article, supplier, quantity, cabinet | Export CSV |

The panel can't refresh itself when the design changes yet (phase 3 of RFC 0015). Every entry
of ours updates it when it runs, and Refresh covers edits made with SolidCraft's own tools.

### The ribbon

Five buttons in workflow order, each with an icon drawn in SolidCraft's own style (isometric, grey
for what is there, blue for what the button makes; `scripts/make_icons.py`). The owner asked for a
few buttons covering the main functions, each with an icon that says what it does, and for adding
and editing a cabinet to be separate buttons (2026-11-27).

| Button | Icon | What it opens |
|---|---|---|
| **Job** ▾ | a grey cabinet and a blue settings dial | Job setup, Boards, Cabinets panel, Rebuild all; and, until SolidCraft lets panel-only entries leave the ribbon (tangency-labs/solidcraft#22300), Select cabinet (the panel's row click). The panel's Refresh runs Cabinets panel, so it needs no entry of its own |
| **Add** | a blue cabinet and a plus | Add cabinet: at the end of the run, or beside the picked one (PLUG-1); with boxes picked, cabinets in them (PLUG-5) |
| **Edit** | a grey cabinet and a blue pencil | Edit cabinet: changes the picked cabinet |
| **Bench top** | a grey cabinet under a blue top | One dialog, its first field a picture choice of piece: benchtop, end panel, filler, splashback |
| **Cut List** | a sheet with blue parts | One dialog, its first field what to make: cut list CSV, cut sheets, part labels |

SolidCraft 0.50's ribbon rules, found with the owner's screenshots (2026-11-27):

- A button's label is its ribbon slot's `name`. An entry's title in menus and its dialog is its
  `menu` slot's `name` (as Woodwork does); without one it is the bare entry name ("setup"). Every entry gets a button unless it is in a
  `ribbon_groups` drop-down.
- **Drop-downs come first**, before every single button, whatever the manifest's order. So the only
  drop-down is Job, which belongs first anyway.
- **Labels are narrow.** A single button holds about 8 letters ("Benchtop" is cut), a drop-down
  about 5 ("Bench…", "Cabin…") as its arrow takes the room. Two short words wrap onto two lines
  ("Cut / List", "Bench / top"). Keep labels to one word of up to 5 letters, or two short words.

### Editing a cabinet

The owner wants **Edit cabinet** active only while a cabinet is selected. SolidCraft 0.50 can't
do that: a plugin's ribbon button is always enabled, and nothing ties it to the selection. Until
it can (see Asks of SolidCraft):

- Edit cabinet's dialog needs a pick: its first field takes the viewport's pick, and Run stays off
  while nothing is picked.
- Its hook reads the picked body's cabinet (`C03.Door1` is C03) from the design's storage and fills
  in the dialog: "Changing C03", its type, size and fronts. A pick that isn't one of our cabinets
  greys the fields and says so, and the run refuses it.
- A filled-in form can't be opened from the panel (`forms.open` only works after a dialog's run),
  so clicking a row selects that cabinet in the view, ready for Edit cabinet.

### Asks of SolidCraft

All of them, with what happened and how to see it: [solidcraft-asks.md](solidcraft-asks.md).

Drafts, to post on tangency-labs/solidcraft only with the owner's go-ahead:

1. **A plugin entry enabled by the selection.** Posted 2026-11-27 as tangency-labs/solidcraft#23098. A manifest field on an entry, such as
   `"enabled_when": {"selection": {"accept": ["body"], "min": 1}}`, and better, a read-only
   check entry the host calls on selection change (with the same budget as a dialog hook) that
   answers whether the button is enabled and why not, shown as its tooltip. Our use: Edit cabinet
   is greyed until a cabinet is picked.
2. **Open a populated form from a panel event**, so a row's double-click opens its cabinet. Already
   fixed upstream: tangency-labs/solidcraft#22032, merged as #22044 on 2026-11-26, after 0.50.0. When
   the plugin targets 0.51, a row's double-click (`on_edit`) can open Edit cabinet filled in.
3. **Images in panels** (a picture column or tile list), for board swatches and cabinet thumbnails.
   Already asked: tangency-labs/solidcraft#20553 (open), pictures in form choice lists and panel rows.

## Data

- **In the design (`storage@0.1`):** the job: setup, board roles, each cabinet's spec (the same
  JSON as a `.cabinetjob` item), the run pieces. SolidCraft's bodies are rebuilt from it, so the
  design stays the source of truth, and it opens without the plugin as ordinary bodies.
- **On the Mac (`user-storage@0.1`):** the shop standard.
- **In the component:** the catalogue (boards, edge tape, hardware, sinks), built in.
- **Pictures:** PNGs in `plugin/pictures/`, made by a script from the drawings Cabinet Studio
  already uses for its tiles, so the two look alike. At most 256 in the package.
- **A job file from Cabinet Studio** opens in the plugin: an `import_job` entry reads a
  `.cabinetjob` from a granted folder into the design's storage, and builds it.

## Milestones

Each ends with `scripts/build.sh --test` green (`plugin test --require-cases`), and an
on-screen walk through the new steps for the owner.

| id | Size | What | Done when |
|---|---|---|---|
| PLUG-0 | S | **Spike.** The component builds from the core and makes one base cabinet. Measure: engine calls per cabinet, and whether `engine.batch` runs from a plugin; preview time; `forms.open` from the panel; `ui.set-data` from an entry run from a dialog; a picture dropdown; design and user storage; `file.insert_mesh` with a folder grant; writing an export. Write the answers into this file | We know each limit from a run, not from the docs |
| PLUG-1 | M | **Job and cabinets:** Job setup, Cabinet (add and edit, all types, front presets), Rebuild, design storage, the shop standard, the panel's Job tab | A kitchen run is laid out, edited and rebuilt in SolidCraft from the plugin alone |
| PLUG-2 | S | **Boards:** the Boards tab, roles, swatches from preview colours | A job's carcass and fronts are chosen from the catalogue |
| PLUG-3 | M | **Run pieces:** benchtop (simplified), panels and fillers, splashback, kick and plinth, handles and legs (meshes or simple shapes) | The kitchen run is complete to the benchtop |
| PLUG-4 | M | **Outputs:** Cut list and Hardware tabs, exports of CSV, cut sheets, DXF and labels | A job's cut list and sheets come out of SolidCraft |
| PLUG-5 | S | **From boxes** and **import a Cabinet Studio job** | A room drawn as boxes becomes cabinets; an existing job opens |
| PLUG-6 | S | **Polish and publish:** icons, help links, agent-tool descriptions, the Workshop listing (the owner publishes) | Installed from the Workshop on a clean Mac |

Rules for the work:

- Every cabinet rule stays in the core with its test. A bug found in the plugin that is a core bug
  is fixed in the core, and both get it.
- Every entry has `plugin test` cases: its inputs, and what `inspect.measure` should give.
- The plugin's option names and order follow this file. A change to the interface changes this
  file first.
- When SolidCraft adds image widgets, trees, workspaces or design-change listeners for panels
  (the PLG-3 weekly check), revisit the panel: a Cabinets workspace and pictures in the
  panel are the next step.

## Getting builds to the owner (and later, users)

Each build the owner should try is a GitHub release of this repo: `scripts/release.sh` builds,
tests, checks and publishes `cabinet-studio-<version>.zip`. `plugin.json` names the repo under
`updates`, so SolidCraft (0.49+) offers a newer release on the plugin's card in Preferences ›
Plugins as **Update to x.y.z**, checking at start and once a day (owner 2026-11-27). The card's
Update keeps the granted exports folder. Installing through the control port's `plugin.install`
(no dialog) hung on 0.50 and was given up. Checked 2026-11-27: with 0.0.8 installed, SolidCraft
offered Update to 0.0.9 after a restart.

## PLUG-5: from boxes (2026-11-28, 0.6.0)

**From boxes was taken out in 0.7.4** (owner: cabinets are made directly in SolidCraft now). Gap
items stay: a wall cabinet above a base uses them.

- **Add cabinet › Or from boxes:** pick boxes drawn in SolidCraft; each becomes a cabinet sized to
  it, its type by the core's `CabinetKind::for_box` (tall, wall when up off the floor, else base),
  in a run along X from the first box's corner (the job's `origin`) with the gaps between boxes
  kept as gap items; then the boxes are removed (`solid.remove`).
- For now one row of boxes, on a job with no cabinets yet; anything else is refused with why.
- 0.7.0: **Job ▾ › Open a Cabinet Studio job** reads a `.cabinetjob` from the folder granted as
  `jobs` at install (the update asks for it once): the file's standard becomes the job's, and its
  cabinets and pieces replace the run, kept as Cabinet Studio wrote them (`item: "core"`) so
  sections, corners and the rest build as in the app; the job's boards and sizes stay. Edit
  changes an imported cabinet's width, height and depth or removes it (0.7.1); its type, sections
  and fronts stay the app's, as the plugin's front layouts are simpler. The owner's six job files all build this way (`examples/check_import.rs`). A dialog's
  one-run folder field was tried first: SolidCraft 0.50 can't link it into a plugin with panels and
  hooks (solidcraft-asks.md, 16).

## Next (owner's notes, 2026-11-28)

- **PLUG-7: cut sheets as SolidCraft drawings.** The cut list, cut sheets and cutting layouts should
  be SolidCraft's own sheet drawings (its `drawing.*` commands: sheets, views, tables, dimensions),
  made in the design, not PDFs written by a script. The PDFs and CSV stay as exports.
  **0.11.0:** Cut List › **Cutting layouts as a SolidCraft drawing** (and Everything) makes the
  drawing "Cutting layouts" (`src/drawing.rs`): an A3 sheet for each board sheet the nesting opens,
  the board to scale in a sheet sketch with each part's rectangle, a note with its name and cut
  size, a heading (sheet n of m, parts, % used) and the board in the title block. A run replaces
  the drawing it made before and leaves it open. Next: the cut list as a table, and part drawings.
- **PLUG-8: a BOM for costings.** Boards (sheets, edge tape by the metre), hardware and benchtops with
  quantities and prices where the catalogue has them (ABI's handles carry `price_aud`; Cabinet
  Studio's Quote tab is the model), and a total.
  **0.10.0:** Cut List › **Costs**: the core's quote (`cabinet_core::quote`, as the Quote tab) into
  `costs.csv` and a Costs table in the Cabinets panel. The dialog's Prices section (shown for Costs
  and Everything) holds the shop's price list, kept in user storage (`prices`) from job to job:
  per-m² prices of this job's carcass, front, back and benchtop boards, edge tape, labour, times,
  markup and GST. 0 is no price; the quote lists what it lacks. ABI handles take their list price.
  0.10.1 prices hardware by kind (hinges, plates, runners, lifts, legs, handles, hangers, shelf
  supports, sinks, other), each, kept as `hardware_prices`: a dialog can't list a job's own
  articles. A handle price of 0 leaves ABI's list prices. Manifest: 60.8 KB of the 64 KB limit.
- **PLUG-9 (0.8.0, points 1 and 2 done):** double-clicking a row in the Cabinets panel opens its dialog filled
  in (Edit for a cabinet, Bench top for a piece), and the panel's row helpers have `model_group:
  false`, off the ribbon and the Job menu; the plugin needs SolidCraft 0.51. Point 3 (Add and Edit
  as ordinary runs with a preview) needs the user's Max plugin host calls raised first.
  0.8.1 drops the row's select action: `select.set` from a plugin run doesn't reach the app's
  selection. In 0.51.1 a row's double-click reaches nothing (solidcraft-asks 21) and a button gets
  no row (20; 0.8.3 tried an Edit button, 0.8.4 took it out), so point 1 waits on SolidCraft; the
  `on_edit` hook stays. Warnings is a table (0.8.2): an item list broke every button (19).
  **Point 3 (0.9.0):** Add and Edit are ordinary runs again, so their dialogs preview, and the
  manifest asks for 5000 host calls (`runtime.limits.host_calls`), which the user allows by raising
  Max plugin host calls in Preferences › Plugins. A preview (`plugin.preview`) builds plain panels
  at design detail, unpainted, and stores nothing; whether a cabinet changed is still judged at the
  job's detail. The build passes `scripts/plugin-limits.json` to the CLI for the same ceiling.
  Checking it: `plugin.run` with `preview: true` over the control port only makes `plugin.preview`
  answer true; it builds in the live design (undo it). Only the dialog's Preview runs on a scratch copy.
- PLUG-9 as planned: **what SolidCraft 0.51 allows** (the owner updated to 0.51.1): panel helpers opting out of
  the ribbon (#21604), a panel row's double-click opening Edit filled in (`forms.open`, #22044),
  previews that build less (`plugin.preview`, #22502), and a raised engine-call ceiling (#19515), so
  Add and Edit could be ordinary runs with a preview again.
- **Quick size buttons** like Cabinet Studio's chips (600, 900...): SolidCraft's dialogs have no such
  control (number fields, text dropdowns only). Asked of SolidCraft: tangency-labs/solidcraft#23309.
  Meanwhile a size could be a dropdown of the usual sizes plus "Other" and a number field.

## PLUG-4: outputs (2026-11-28, 0.5.1)

Checked in SolidCraft's window by Claude: Cut List › Everything below wrote the cut list (the job's
Laminex Avignon Walnut carcass), cut sheets, cutting layouts, labels, 12 DXFs and the hardware list
(Blum and Häfele articles) into the granted exports folder (the owner's Desktop).

**Cut List** opens a dialog of what to make, written into the exports folder granted at install:
everything at once, the cut list (CSV), cut sheets (a PDF page a part), cutting layouts (the parts
nested on each board's largest sheet, our panel-saw settings: 4.4 mm kerf, 10 mm trim; a board
with no catalogue sheet nests on 2400 × 1200), part labels (PDF), a DXF a part for a CNC (in
`dxf/`), and the hardware list (CSV, brand and article). All from Cabinet Studio's `cabinet-sheets`
and core, pinned like the rest. The panel's Cut list and Hardware tabs of the plan are left out:
the outputs are files, and panels proved fragile (see Boards).

## PLUG-3: the run's pieces (2026-11-28, 0.5.0)

Checked in SolidCraft's window by Claude (owner asleep, with leave to drive the app): updated
through Preferences › Plugins, the Bench top dialog draws every field, a benchtop and an end panel
went in and the Cabinets panel lists them. The owner's own check is to come.

- The stored run holds cabinets and pieces (`item`: cabinet, benchtop, end_panel, filler); each
  becomes the core's run item (`spec::item`), and the build treats them alike: fingerprints,
  rebuild only what changed.
- **Bench top** opens one dialog, its fields shown for the piece chosen:
  - **Benchtop, splashback and cut-outs:** thickness (0: the benchtop board's, or 33 mm), front and
    side overhang, edge (square, pencil round, bullnose, bevel), a splashback height (parametric,
    in the benchtop's board), a sink (one of the 16 ABI sinks, or a cut-out of my size) over a
    picked cabinet, a cooktop cut-out over a picked cabinet. Its board is the one chosen for
    Benchtop with the Boards button. The dialog opens with the benchtop's settings.
  - **End panel** and **Filler:** pick the cabinet it goes beside and the side; a filler's width;
    scribe. Picking the panel itself with Remove takes it out.
  - **Remove it** takes the benchtop out.
- Kick and plinth come with each run from Job setup, as before.
- 0.7.2: **Add** with a wall-type cabinet (wall, wall corner, rangehood) picked beside a base
  cabinet puts it above that base: wall cabinets have their own line in the core's run, so a gap
  in it brings the new one to the base's left side (the run's toe kick spans the run, so the edge
  is the base's left side panel, not its parts' least X). Checked headless: both left sides at
  the same X.
- Not yet: L-shaped runs and returns, stone slabs and joins, waterfalls; handles and legs are
  placeholders (PLUG-1).

## Boards as a dialog (2026-11-28, 0.4.0; 0.4.2 checked by the owner in SolidCraft)

0.3.0's Boards panel failed the owner on SolidCraft 0.50: a declared panel always opens (a plugin
can't start one closed), a ribbon button always opens a dialog (it can't toggle a panel), after
resizing the panel the window would no longer resize, and "Use for" snapped back to Carcass and
Brand didn't refresh the list (the panel's runs share one settings file on the Mac and collide).
So the **Boards** button opens a dialog, as SolidCraft's own tools do:

- **Use for** (its hint names the role's board now), **Brand**, **Range**, **Find decor** (part of
  a name), **Decor**, **Finish**, **Thickness**: each list narrowed by the choices above it by the
  dialog's hook, to at most 256 (SolidCraft's limit; the hint says when to type in Find decor).
  **Run** gives the role that board and rebuilds or repaints.
- The manifest lists every range, decor name, finish and thickness once (47 KB of its 64 KiB);
  the hook only narrows them, in the manifest's order.
- A hook may spend only 10 million instructions, too few to read the whole board index. The index
  starts with a table of contents, where each brand's lines are, so the hook reads one brand's.
- The Boards panel and its two helper entries are gone.
- 0.4.3: the Cabinets panel lists the job's board per role under the cabinets (owner): a hook's
  hint doesn't show, so the dialog can't.
- 0.4.1: 0.4.0's dialog said "This entry needs to be run from the agent or CLI": SolidCraft's
  dialog draws a dropdown only of text, and Thickness was a dropdown of numbers. It is text now
  ("18", shown "18 mm"), and `make_manifest.py` checks every field against what the dialog draws
  (`check_drawable`, after `form` in SolidCraft's plugin_menu.rs), as headless tests never draw one.
- 0.4.2: in 0.4.1 Find decor searched the current range only, and the dialog opens on the
  brand's first range by name (Laminex: Compact Laminate), so "delana" found nothing and Decor fell
  back to the whole catalogue's names (seen in the owner's window). Find now searches the whole
  brand, Range offers only ranges with a match and opens on the brand's biggest, and with no match
  the lists grey out. The narrowing is plain code (`boards::narrow`), checked on the real index by
  `examples/check_boards.rs` in `build.sh --test`. A hook's `hint` showed nowhere in the dialog, so
  nothing relies on one.

## Boards reworked (2026-11-27, 0.3.0)

The owner tried 0.2.x: the Cabinets panel's Boards tab had no room for its columns, Laminex was
hard to find (an empty search shows the first boards in catalogue order, Dekton and EGGER), and a
pick looked like it did nothing as it was slow. Now:

- A **Boards** ribbon button (a stack of sheets, the top one blue) opens a **Boards panel**
  docked along the bottom of the window: the job's board per role and the filters (Use for,
  **Brand**, Search) on the left, the boards on the right with the window's width (300 rows).
  The Cabinets panel is cabinets only again. The ribbon: Job ▾, Add, Edit, Boards, Bench top,
  Cut List.
- The plugin reads a compact index of the catalogue, `src/boards.tsv`, which the build writes from
  Cabinet Studio's catalogue (`examples/catalog.rs --boards`), not the whole catalogue: 365 KB
  against 3.3 MB, and the component no longer carries `cabinet-catalog`.
- **A pick rebuilds only what its thickness changes.** Each cabinet keeps a fingerprint of its
  shape and one of its colours: a board of the same thickness repaints the parts (48 ms against
  771 ms a cabinet, headless).

## Job setup's sizes (2026-11-27, 0.2.1)

The owner missed the sharknose finger gap and the drawer rail sizes: Job setup had only Cabinet
Studio's pickers, not the size rows under them. The rows are the core's now
(`options::measurements`, shown where `options::applies` says), so Job setup has all of them
(finger gaps, rails, door gaps, shaker frame, plinth, finger pull notch, edge pull clearance,
wardrobe drops), each right after its picker, its usual values in its description. A field shows
once, so the rails behind a sharknose stack are under Drawer frame, and the sharknose finger gap
says so. Cabinet Studio can take its chips from the same list.

## PLUG-2: boards (2026-11-27, 0.2.0)

- The board catalogue is built into the program: `cabinet-catalog`'s `embedded` feature (all of
  `catalog/boards` and `catalog/edge-tape`, kept equal to the folder by its tests).
- The Cabinets panel has two tabs. **Boards** shows the job's board for each role (carcass, fronts,
  backs, benchtop), a **Use for** choice and a **Search** field, and the matching boards: brand,
  range, decor, finish, thickness and largest sheet, at most 200 rows (a panel holds 1 MiB) with a
  last row saying how many more matched. Benchtop lists laminate tops and sintered stone; the other
  roles list sheet boards, linings left out.
- **Clicking a board gives the chosen role that board**: the core's board and its thickness, then
  the cabinets it changes are rebuilt, coloured by the boards' preview colours (`board_colours`).
  The benchtop's board waits for PLUG-3. Ctrl+Z undoes it.
- Job setup shows the job's boards in a hint under Door style.
- "Use for" and the search are kept on the Mac (`user-storage`), so the tab opens as it was left.
- Two more panel helpers sit in the Job menu, Filter boards and Choose board, until SolidCraft lets
  panel-only entries leave the ribbon (tangency-labs/solidcraft#22300).

## PLUG-1: job and cabinets (2026-11-27, 0.1.0; 0.1.1 checked by the owner in SolidCraft)

- **Job setup** (Job ▾): the five sections of the plan, the choices generated from the core's
  option groups (`examples/catalog.rs`), shown only where the core's `applies` says they apply.
  It opens with the job's setup, or the shop standard for a job without one; **Also save as my shop
  standard** keeps it on the Mac (`user-storage`). Applying it rebuilds what it changes.
- **Add** and **Edit**: all 11 cabinet types and their front layouts as pictures, drawn from each
  cabinet as the core builds it; height and depth start at the job's sizes for the type and
  follow the type until changed (0.1.1: 0.1.0 showed 0, as SolidCraft ignored the hook's reply); type extras
  (sink, blind corner) and **Different from the job** (door style, how fronts open). Add puts the
  new cabinet at the end of the run, or after the cabinet picked in **Beside**. Edit can remove the
  cabinet.
- **Only what changed is rebuilt.** Each cabinet keeps a fingerprint of its SolidCraft commands;
  after any change the whole run is worked out again, and only cabinets whose commands differ are
  taken out and built again (inserting one moves those after it, and the run's kick).
- **Rebuild all** (Job ▾, and the panel's toolbar) builds every cabinet again.
- 0.7.3: the view is re-framed only when the run gains or loses an item, not on every edit.
- 0.7.4: an L corner or wall corner has **Return side** (pictures: side wall on the left or right,
  the core's `Corner.side`) and **Return length** (its second leg), in Add and Edit (owner: it was
  only ever drawn one way; checked by the owner in SolidCraft). From boxes taken out.
- **A hook reply SolidCraft ignores only shows in the run's log** ("Dialog hook reply ignored: …"),
  and the dialog falls back to the manifest's defaults. `scripts/build.sh --test` fails on one. Its
  choices must keep the manifest's order, and its values must fit the field's limits.
- **Handles** are placeholders at their catalog sizes on their real holes, in the core
  (`standins::handle`), so Cabinet Studio's builds without supplier models show them too.
- **The Cabinets panel**: cabinet, type, width × height × depth, fronts and warnings per row, the
  warnings listed below; a row click selects the cabinet for Edit.

Different from the plan, and why:

- **No live preview.** Add, Edit and Job setup run as long jobs: one change can rebuild many
  cabinets, beyond an ordinary run's 256 engine calls, and SolidCraft previews no long job. When
  `plugin.preview` ships (after 0.50), a preview can build just the cabinet in the dialog.
- **"Above it"** (a wall cabinet over a base) waits for wall runs over base runs (PLUG-3).
- **The Boards section** of Job setup is PLUG-2's Boards tab.

## PLUG-0: what the spike found (2026-11-27, SolidCraft 0.50.0)

The spike is the code in this repo at its first commit: `cabinet`, `refresh`, `edit`, `export` and
`handle` entries, a Cabinets panel, and 12 `plugin test` cases, all passing headless.

| Question | Answer |
|---|---|
| Does the core build as a plugin? | Yes, once material packs were made a feature of `cabinet-catalog` (zip pulled in `wasm-bindgen` through `getrandom`, which SolidCraft can't link). The component is 1.6 MB |
| Engine calls per cabinet | One command, one call: plugins have no `engine.batch`. At Design detail: wall 17, sink 49, base 50, drawer bank 57. At Manufacturing detail, a base is 197. An ordinary run allows 256, plus our storage and view calls |
| What that means | Add and Edit build one cabinet at Design detail in an ordinary run, with room to spare. Rebuild all, Manufacturing detail and From boxes run as long jobs. Preview stays at Design detail |
| Time | 0.6–0.8 s a cabinet at Design detail, 2.5 s at Manufacturing, headless |
| `plugin.preview` (telling a preview from a run) | Not in 0.50: it's in SolidCraft's docs on main. Until it ships, a preview builds the same as a run |
| Design storage | Works: the run's cabinets kept with `storage.set`, read back in the next case from a saved design |
| Editing a cabinet | Works: deleting its component and building it again from the stored spec, one undo step |
| A cabinet added after the others | Works: the core's run places it beside the last one |
| `forms.open` | Refused headless as it should be (`interactive_required`). In the app, see below |
| Supplier meshes | `file.insert_mesh` works from a folder granted read-only, with the `file` command group, and lands at the origin. Not used: the owner wants nothing outside the plugin (see Hardware above), so the entry was removed in 0.0.2 |
| Exports | `files.write_file` into a granted folder works (the cut list CSV) |
| Panel data from a dialog run | `ui.set-data` and `show-panel` succeed in a run started from the dialog. Whether the open panel shows it: see below |
| Manifest size | 3.7 KB for the spike. `plugin-runtime.md` says 16 KiB, `plugins.md` says 64 KiB. Keep the real plugin's manifest under 16 KiB: pictures and long choice lists are the risk, so shared `$defs` and few, short descriptions |

In the app (the owner installed 0.0.1 on SolidCraft 0.50.0, 2026-11-27):

| Check | Result |
|---|---|
| Picture dropdown | Works: a large picture of the highlighted option above a grid of tiles |
| Panel updates after a dialog run | Works: the row appeared without Refresh |
| Double-clicking a row to edit | Didn't open the form: `on_edit` is for editing a cell, and a row click (`on_select`, 0.0.2) didn't either: a panel event can't open a form in 0.50. 0.0.4 edits through Edit cabinet's dialog hook instead (Editing a cabinet, above); to check |
| Supplier mesh | Went in, at the origin (expected in the spike) |
| Ribbon labels | Wrong: a button's label is its ribbon slot's `name` ("Model"), and every entry gets a button. 0.0.2 fixed the labels; the owner wants a few buttons with icons for the main functions (0.0.3), and Add and Edit cabinet apart (0.0.4) (The ribbon, above); to check |

Other things seen: the dialog's header is the entry's name in lower case ("cabinet"), and the
install screen lists every grant and the panel. Entry names should read as titles.
