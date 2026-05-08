# 2026-04-09 - g09.008 CLAP Sandbox Protocol Hardening Tranche

## Summary

Completed the targeted CLAP lifecycle hardening seam in
`signal-plugin-clap`.

## Code Reality

- replaced the remaining panic-oriented lifecycle `expect(...)` handling in the
  targeted create/prepare/activate/deactivate/reset path with explicit helper
  projection and validated-instance checks
- preserved lifecycle continuity semantics while converting drift into typed
  `protocolViolation` sandbox failure envelopes
- added focused lifecycle failure coverage for:
  - activate requests with an unprepared epoch
  - prepare-state projection loss
  - reset-state projection loss

## Validation Run

- `cargo test -p signal-plugin-clap`
- `cargo check -p signal-ipc`
- `effigy health`

## Validation Notes

- the CLAP batch did not add new warning noise
- `cargo test -p signal-plugin-clap` still reports the same pre-existing unused
  import warnings in:
  - `crates/signal-plugin-clap/src/tests/block_processing.rs`
  - `crates/signal-plugin-clap/src/tests/lifecycle.rs`

## Reassessment

The next honest `g09.008` seam is shared-memory lifecycle hardening. The
remaining CLAP warning noise is too narrow to justify another strict batch
card, while `signal-ipc` still carries the broader ownership and stale-region
cleanup posture gap already captured by the roadmap.

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/008-g09-008-shared-memory-lifecycle-hardening.md`.
