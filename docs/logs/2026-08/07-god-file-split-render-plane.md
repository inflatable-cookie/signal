# God-File Split: signal-render-plane

Status: complete
Created: 2026-08-07
Scope: `crates/signal-render-plane` structure hygiene (mechanical module split)

## Baseline

`effigy scan god-files` (warn=250 / high=400 / critical=700):

- scanned-files: 456
- severity-counts: critical=20 high=21 warning=27
- worst prod: `src/lib.rs` (6073 code lines), `src/offline.rs` (2622)

## What Changed

Split by existing section seams. Public crate API unchanged (`lib.rs` re-exports).

`lib.rs` → sibling mods:

- `sample_buffer`, `notes`, `plugin_events`, `stream`, `live_input`
- `plugin_processor`, `plan_spec`, `plan`, `plan_render`
- `plane/` (`command`, `controller`, `executor`)
- unit tests → `tests.rs`

`offline.rs` → `offline/`:

- `mod.rs` (options/output + re-exports)
- `stretch_artifact`, `bounce`, `wav`
- unit tests → `offline/tests.rs`

No DSP / RT contract / IPC shape changes — move-only.

## After

`effigy scan god-files`:

- scanned-files: 474
- severity-counts: critical=20 high=25 warning=30
- render-plane **production** criticals: none
  - cleared: `lib.rs`, `offline.rs`, and the interim `plane.rs` critical
- remaining render-plane noise: unit-test modules (`tests.rs`, `offline/tests.rs`) still critical; several prod highs (`stretch_artifact`, `plan`, `executor`, …)

Headline count stayed ~flat because extracted test modules are now separate findings. Production critical surface in this crate is gone.

## Validation

- `cargo fmt -p signal-render-plane`
- `cargo clippy -p signal-render-plane --all-targets --all-features -- -D warnings`
- `cargo test -p signal-render-plane --lib` — 145 passed

## Next Task

Continue god-file campaign on next worst **production** criticals (skip test/demo/bin until prod criticals shrink): e.g. `signal-plugin-vst3/.../hosting.rs`, `signal-dsp-stretch/src/lib.rs`, `signal-plugin-bridge/src/in_process.rs` — or stop for operator review.
