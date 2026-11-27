# Cabinet Studio for SolidCraft

Cabinets for [SolidCraft](https://github.com/tangency-labs/solidcraft), as a plugin: lay out a
kitchen or wardrobe, build it in SolidCraft, and get its cut list and sheets. The cabinet rules,
hardware data and board catalogue come from
[Cabinet Studio](https://github.com/brendan779/CabinetForSolidcraft), the macOS app this grew from.

**Status: early.** PLUG-0, the spike, is done; the plan is [docs/PLAN.md](docs/PLAN.md).

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
