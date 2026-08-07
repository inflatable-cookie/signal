# God-File Split: realtime_preview + sandbox_broker_support

Status: complete
Created: 2026-08-07
Scope: `signal-dsp-stretch` realtime preview; `signal-runtime` sandbox broker support

## Baseline

After prior god-file batches: critical≈20. Next prod criticals included
`realtime_preview.rs` (1674) and `sandbox_broker_support.rs` (1389).

VST3 `hosting.rs` attempted first; COM/vtable private-field coupling made a
safe sibling split too expensive for this batch — restored monolith, deferred.

## What Changed

### `signal-dsp-stretch` `realtime_preview`

→ `realtime_preview/`:

- `contract` — plan types + projection helpers
- `callback` — `RealtimePreviewCallbackState` impl
- `tests`

### `signal-runtime` `sandbox_broker_support`

→ `sandbox_broker_support/`:

- `types` — receipts, inventory, session structs + wire helpers
- `client_session` — broker client impl
- `ops` — ensure/record/teardown entry points
- `tests`

Move-only. Public crate re-exports unchanged.

## After

`effigy scan god-files`: critical=19 high=31 warning=33.

Cleared production criticals:

- `realtime_preview.rs`
- `sandbox_broker_support.rs`

Residue: preview callback/tests and broker client/types as high/warning.

## Validation

- `cargo fmt` / `clippy -D warnings` on both crates
- `cargo test -p signal-dsp-stretch --lib realtime_preview` — 21 passed
- `cargo test -p signal-runtime --lib sandbox_broker_support` — 7 passed

## Next Task

Next prod criticals: `vst3_host_adapter/hosting.rs` (needs careful COM-local
split or larger owning-module carve), then clap/au hosting, sandbox `broker.rs`,
fixtures, `creative.rs` / `phase_vocoder.rs` — or stop for operator review.
