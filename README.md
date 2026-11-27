# Cabinet Studio for SolidCraft

Cabinets for [SolidCraft](https://github.com/tangency-labs/solidcraft), as a plugin: lay out a
kitchen or wardrobe, build it in SolidCraft, and get its cut list and sheets. The cabinet rules,
hardware data and board catalogue come from
[Cabinet Studio](https://github.com/brendan779/CabinetForSolidcraft), the macOS app this grew from.

**Status: early test builds.** The plan and what's done: [docs/PLAN.md](docs/PLAN.md).

## Using it

The ribbon's Plugins group, left to right, is the way through a job:

1. **Job ▾ › Job setup.** How this job is built: door style and mounting, how fronts open
   (handles, knobs, edge or finger pulls, sharknose, TIP-ON, SERVO-DRIVE) with their sizes, drawers
   and hinges, carcass joints and fixings, the base and wall hanging, and the standard heights and
   depths. Only what applies is shown. Tick **Also save as my shop standard** and new jobs start
   from it.
2. **Add.** A cabinet: its type (11, as pictures), width, height and depth (the job's sizes for its
   type until changed), fronts (as pictures), and what it does differently from the job. Pick a
   cabinet in **Beside** to put the new one on its right. **Or from boxes:** draw boxes in
   SolidCraft for the layout, pick them, and each becomes a cabinet of its size.
3. **Edit.** Pick any part of a cabinet (or click its row in the Cabinets panel), then Edit: the
   dialog fills in with it. Change anything, or remove the cabinet.
4. **Boards.** Choose what to use a board for (carcass, fronts, backs, benchtop), its brand, range,
   decor (type part of a name in Find decor), finish and thickness. Cabinets rebuild in it; one of
   the same thickness only repaints.
5. **Bench top.** The benchtop over the run (thickness, overhangs, edge, splashback, a sink over a
   picked cabinet, a cooktop cut-out), or an end panel or filler beside a picked cabinet.
6. **Cut List.** The cut list, cut sheets, cutting layouts, part labels, a DXF a part and the
   hardware list, into the folder you chose when installing.

The **Cabinets** panel lists the run, the job's boards and any warnings. **Rebuild all** builds
every cabinet again. Each change rebuilds only the cabinets it changes.

Hardware (hinges, runners, legs, handles) is drawn as simple shapes at each part's catalogue size,
on its exact drilling.

## Build

Needs Rust with `wasm32-unknown-unknown`, `wasm-tools` (`cargo install wasm-tools`) and SolidCraft
0.50 or later.

```bash
scripts/build.sh --test    # build, check and run the test cases
scripts/build.sh --zip     # then pack target/cabinet-studio-<version>.zip
```

Install the zip in SolidCraft from **Preferences › Plugins › Install plugin…**, or the latest
release's `cabinet-studio-<version>.zip`. After that, SolidCraft offers each new release on the
plugin's card in Preferences › Plugins (it checks at start and once a day).

`scripts/release.sh "notes"` publishes the version in `plugin.json` as a release.

## Licence

MIT or Apache-2.0, at your choice. `wit/plugin.wit` is SolidCraft's plugin interface, from
tangency-labs/solidcraft (MIT OR Apache-2.0).
