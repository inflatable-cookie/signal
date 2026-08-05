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
- Added six release gates: `fmt`, `lint`, `lint:no-features`, `test`,
  `validate`, `docs`. All six pass; the run takes ~312s, dominated by `test`.
- Cleared all 14 clippy warnings so both lint gates can deny warnings. Nine were
  mechanical (`is_multiple_of`, `map_identity`, `skip().next()`, items after a
  test module). Three needed judgement, below.
- Reconciled contract `080`, whose required-evidence list named
  `effigy demo:coverage-matrix` — a task absent from this repository's manifest.
  A gate baseline naming an unrunnable task is not a baseline. Replaced with
  `effigy release gates`.

## Judgement Calls in the Lint Clearance

`not_unsafe_ptr_arg_deref` on `gui_open_embedded` in the VST3 and AU host
adapters. The lint is right: both take a caller-supplied `*mut c_void` parent
window and hand it straight to FFI that attaches a view to it, so a bogus handle
is UB. They are now `unsafe fn` with a documented safety contract. CLAP's
equivalent was not flagged only because its raw pointer crosses into unsafe code
one call deeper — the same hazard, less visible — so it was marked `unsafe` too,
keeping the three adapters consistent.

This changes the public signature of three crates. It was done now precisely
because `0.1.0` is the *first* tag: there are no external consumers pinned to
the safe signature yet, so this is the cheapest moment it will ever be. After
tagging it becomes a breaking change.

The `signal-plugin-bridge` wrappers stay safe `pub fn` taking `usize`. That is a
deliberate design choice — the integer keeps the backend `Send` — but it means
the bridge is a trust boundary that the type system does not mark. Each call
site now carries a `SAFETY` comment naming what the caller must guarantee. The
unmarked boundary is pre-existing and not addressed here.

`too_many_arguments` (13/7) on the RealtimePreview projection builder. Two of
its three call sites passed `0, 0, 0, 0.0, 0.0, 1.0, 1.0, false, 0, 0, 0, 0.0, 0`
positionally, which is the readability failure the lint exists to catch. The
eight ratio-related parameters are now a `DynamicSourceProjectionRatios` struct
with an `idle()` constructor for the reset path.

`large_enum_variant` on `ParseOutcome` in the corpus report binary: `Run` is
~232 bytes larger than `Help`, so it is boxed.

## Cargo.lock Is Not A Sync-File

`config/release.toml` originally set `sync-files = ["Cargo.lock"]`, which is the
documented shape. On the `0.1.0` prepare it turned out effigy syncs that entry
with `cargo generate-lockfile`, which rebuilds the lockfile from scratch and
resolves every dependency to its newest compatible version. The prepare bumped
roughly 40 crates, including `syn` 2 to 3 and `rustix` `0.38` to `0.41`.

Sync-files also run *after* the gates, so those upgrades would have entered the
release commit having never been compiled or tested. Caught by reading the
diffstat: 357 changed lines in a lockfile whose only legitimate change was 28
workspace members going `0.0.0` to `0.1.0`.

The entry is removed. The lockfile is updated with `cargo update -w`, which
touches only workspace member versions. Worth reporting upstream: an effigy log
from 2026-03 records the sync as `cargo check --quiet`, which would have been
correct, so this changed somewhere between then and `v0.8.17`.

## Planning State

`effigy release status` reports the changelog valid and the lane ready to
prepare. The workspace version was reset from `0.1.0` to `0.0.0`: the `0.1.0`
already in `Cargo.toml` had never been released or tagged, so `prepare` refused
with "0.1.0 must be greater than current version 0.1.0". `0.0.0` states the true
position and lets the lane own the bump. `prepare` still needs an explicit
`--version 0.1.0`, since the suggested bump from `0.0.0` is `patch -> 0.0.1`.

Tagging is blocked on the `g10.039` rev2 listening pack. Contract `084` Rule 5
makes listening the promotion authority, and the pack is unjudged, so tagging now
would ship unadmitted DSP behaviour inside the release.

## Next Task

Judge `~/Downloads/signal-listening-pack-39-rev2`, close out g10.039, then
`effigy release prepare --version 0.1.0` and tag.
