# CabinetStudioPlugin: notes for Claude

Cabinet Studio as a SolidCraft plugin. Plan of record: [docs/PLAN.md](docs/PLAN.md).

- The cabinet rules live in brendan779/CabinetForSolidcraft (`cabinet-core`, `solidcraft-bridge`),
  pinned by commit in `Cargo.toml`. Never copy a rule here: fix it there with its test, push, and
  move the pin. That repo is usually checked out at ../CabinetForSolidcraft; to build against it
  before pushing, add a `[patch."https://github.com/brendan779/CabinetForSolidcraft"]` section with
  path dependencies in `.cargo/config.toml` (not committed).
- `scripts/build.sh --test` is the gate: build, `plugin check`, `plugin test --require-cases`.
  Every entry gets cases in `plugin/cabinet-studio/tests`.
- The interface follows docs/PLAN.md: change the plan first, then the manifest.
- `wit/plugin.wit` is copied from SolidCraft's `crates/craft-plugin/wit/plugin.wit` at the tag the
  plugin targets (v0.50.0 now).
- Millimetres everywhere; follow CabinetForSolidcraft's CLAUDE.md conventions for writing style.
