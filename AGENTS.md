# CabinetStudioPlugin: notes for Codex and other agents

Cabinet Studio as a SolidCraft plugin. Start with [docs/HANDOVER.md](docs/HANDOVER.md) (where
things stand, what's next, how to build, test and release), then [docs/PLAN.md](docs/PLAN.md),
the plan of record. [CLAUDE.md](CLAUDE.md) holds the same working rules for Claude; they apply to
any agent.

- The cabinet rules live in brendan779/CabinetForSolidcraft, pinned by commit in `Cargo.toml`.
  Never copy a rule here: fix it there with its test, push, and move the pin.
- `scripts/build.sh --test` is the gate. `plugin.json` is generated: change
  `scripts/make_manifest.py`, not the manifest (only its `"version"` by hand, for a release).
- Releases (`scripts/release.sh "notes"`) are how the owner gets builds.
- Ask the owner before posting issues, merging to main or publishing anything.
