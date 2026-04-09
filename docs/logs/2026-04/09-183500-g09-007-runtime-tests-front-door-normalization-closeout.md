# 2026-04-09 - g09.007 Runtime Tests Front Door Normalization Closeout

## Summary

Completed the active strict `g09.007` runtime test front-door batch. The
direct shared import slab moved out of `crates/signal-runtime/src/tests.rs`
into `crates/signal-runtime/src/tests/support.rs`, leaving `tests.rs` as a
small root entrypoint with the local sink helper and mounted test families.

## Code Reality

- added `crates/signal-runtime/src/tests/support.rs`
- reduced `crates/signal-runtime/src/tests.rs` to the local sink plus mounted
  support and test-family modules
- preserved the existing test tree by keeping `super::*` / `super::super::*`
  lookup chains working through the root support import rather than re-splitting
  test domains

## Validation Run

- `cargo test -p signal-runtime --lib --no-run`
- `effigy health`

## Validation Notes

- `cargo test -p signal-runtime --lib --no-run` passed
- `effigy health` passed
- the pre-existing five-item unused-import warning cluster remains, now on
  `crates/signal-runtime/src/tests/support.rs`:
  - `BrokerInvalidationStage`
  - `LingeringCleanupMode`
  - `LingeringCleanupTrigger`
  - `RuntimeClipProcessingReadiness`
  - `SandboxOperationFailureStage`

## Reassessment

I do not see another honest broad `g09.007` runtime-decomposition seam after
the offline-preview carveout and the runtime test front-door normalization.
The strict lane now needs a planning decision rather than another improvised
batch card.

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
`g09.007` closes here or hands off into `g09.008` before creating another
ready batch card.
