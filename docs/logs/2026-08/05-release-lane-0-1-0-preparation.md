# Release Lane 0.1.0 Preparation

Status: complete
Created: 2026-08-05
Scope: release tooling, changelog format, gate baseline

## Change

Signal is consumed by GitHub ref rather than crates.io, so a tagged commit is
the only stable thing a consumer can point at. This wires the release lane that
produces one.

- Added `config/release.toml` and included it from `effigy.toml`. The workspace
  is virtual — no root `[package]` — so `effigy release status` could not find a
  version until `version-path = "workspace.package.version"` told it where to
  look.
- Converted `CHANGELOG.md` from the flat `(timestamp) [tag] text` log to Keep a
  Changelog. 136 historical entries were parsed and remapped; 9 new entries
  record the g10.036-039 work, which had never been written down. All 145 sit
  under `[Unreleased]`, which is what `prepare` promotes into a version section.
- Added `fmt` and `lint` tasks to `effigy.toml`. Neither `cargo fmt --check` nor
  `cargo clippy` had a repo-owned task before this, so formatting and lint drift
  were unguarded by any command anyone could run. `fmt` also joins `validate`.
- Added five release gates: `fmt`, `lint`, `test`, `validate`, `docs`. All five
  pass; the run takes ~295s, dominated by `test` at ~259s.
- Reconciled contract `080`, whose required-evidence list named
  `effigy demo:coverage-matrix` — a task absent from this repository's manifest.
  A gate baseline naming an unrunnable task is not a baseline. Replaced with
  `effigy release gates`.

## Known Limit

The `lint` gate does not deny warnings. The workspace carries 14 clippy warnings
tracked as g10.038 follow-up, so `-D warnings` would block every release on
pre-existing debt. As written the gate catches new clippy *errors* only.
Tightening it is open work.

## Planning State

`effigy release status` reports the changelog valid and the lane ready to
prepare. Current version is already `0.1.0`, so effigy suggests `patch -> 0.1.1`;
the intended tag is `v0.1.0` and `prepare` must be given `--version 0.1.0`
explicitly.

Tagging is blocked on the `g10.039` rev2 listening pack. Contract `084` Rule 5
makes listening the promotion authority, and the pack is unjudged, so tagging now
would ship unadmitted DSP behaviour inside the release.

## Next Task

Judge `~/Downloads/signal-listening-pack-39-rev2`, close out g10.039, then
`effigy release prepare --version 0.1.0` and tag.
