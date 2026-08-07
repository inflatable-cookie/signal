# God-File Split: plugin-bridge + dsp-stretch

Status: complete
Created: 2026-08-07
Scope: `signal-plugin-bridge` in-process backends; `signal-dsp-stretch` lib core

## Baseline (entry to this batch)

After the render-plane split: critical=20 high=25 warning=30 (scan drifts).

Worst remaining **production** criticals included:

- `signal-plugin-bridge/src/in_process.rs` (2543)
- `signal-dsp-stretch/src/lib.rs` (2881)

## What Changed

### `signal-plugin-bridge`

`in_process.rs` → `in_process/`:

- `common` — event conversion + `PluginGuiEvent`
- `clap` / `vst3` / `au` / `lv2` — one processor module each
- `tests.rs` — unit tests

Crate root re-exports unchanged. `convert_block_event` stays `pub(crate)` for `shm`.

### `signal-dsp-stretch`

`lib.rs` core body →:

- `stretch_backend` — tiers, stretcher types, public entry points
- `stretch_engine` — ratio/segment/pitch helpers + `StretchRenderError`
- `tests.rs` + `dynamic_segment_seam_evidence.rs`

Existing sibling modules untouched. Crate-private helpers re-exported at root for `crate::sanitize_ratio` etc.

Move-only. No DSP algorithm / RT contract / plugin ABI changes.

## After

`effigy scan god-files`: critical=20 high=28 warning=31 (scanned 484).

Cleared production criticals:

- `in_process.rs`
- `dsp-stretch/src/lib.rs`

Touched-crate residue: test modules critical; `stretch_engine` / `stretch_backend` / `in_process/vst3` high. Headline critical count flat because extracted tests are separate findings.

## Validation

- `cargo fmt` / `clippy -D warnings` on both crates (`--all-targets --all-features`)
- `cargo test -p signal-dsp-stretch --lib` — 193 passed
- `cargo test -p signal-plugin-bridge --lib` — 21 passed

## Next Task

Next prod criticals: `vst3_host_adapter/hosting.rs`, `dsp-stretch/realtime_preview.rs`, `runtime/sandbox_broker_support.rs`, plugin hosting/fixture files — or stop for operator review.
