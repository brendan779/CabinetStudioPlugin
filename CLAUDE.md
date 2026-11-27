# CabinetStudioPlugin: notes for Claude

Cabinet Studio as a SolidCraft plugin. Plan of record: [docs/PLAN.md](docs/PLAN.md).

- The cabinet rules live in brendan779/CabinetForSolidcraft (`cabinet-core`, `solidcraft-bridge`),
  pinned by commit in `Cargo.toml`. Never copy a rule here: fix it there with its test, push, and
  move the pin. That repo is usually checked out at ../CabinetForSolidcraft; to build against it
  before pushing, add a `[patch."https://github.com/brendan779/CabinetForSolidcraft"]` section with
  path dependencies in `.cargo/config.toml` (not committed).
- `plugin/cabinet-studio/plugin.json` and `pictures/` are generated: `scripts/build.sh` runs
  `examples/catalog.rs` (the core's option groups, cabinet types and front views) and
  `scripts/make_manifest.py`. Change the generator, never the manifest by hand. The panel's layout
  is in both `make_manifest.py` (PANEL) and `describe` in src/lib.rs; SolidCraft refuses the plugin
  when they differ.
- `scripts/build.sh --test` is the gate: build, `plugin check`, `plugin test --require-cases`.
  Every entry gets cases in `plugin/cabinet-studio/tests`.
- Releases are how the owner gets builds: SolidCraft offers a newer GitHub release on the plugin's
  card (Preferences › Plugins › Update) at its next start. Bump `version` in plugin.json, commit,
  then `scripts/release.sh "notes"`. Installing through the control port's `plugin.install` hung on
  SolidCraft 0.50 (2026-11-27); don't use it on the owner's running app.
- The interface follows docs/PLAN.md: change the plan first, then the manifest.
- `wit/plugin.wit` is copied from SolidCraft's `crates/craft-plugin/wit/plugin.wit` at the tag the
  plugin targets (v0.50.0 now).
- Millimetres everywhere; follow CabinetForSolidcraft's CLAUDE.md conventions for writing style.
