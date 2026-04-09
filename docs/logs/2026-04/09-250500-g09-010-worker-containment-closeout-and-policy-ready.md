# 2026-04-09 - g09.010 Worker Containment Closeout And Policy Ready

## Summary

Closed the first strict `g09.010` batch by removing production
`join().unwrap()` worker crashes from rhythm onset feature extraction and
promoted the next strict ready card for the first real tempo-policy
normalization seam.

## Implementation

- replaced crash-on-worker-loss joins in
  `crates/signal-analysis-rhythm/src/onset_features.rs` with explicit worker
  result containment
- added typed internal feature-availability reporting so degraded worker loss is
  testable without broadening the crate's public API
- preserved deterministic output shape by zero-filling failed worker outputs to
  their expected feature lengths before combining the onset envelope
- added focused worker-panic tests for reduced and full feature paths

## Validation

- `cargo check -p signal-analysis-rhythm`
- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Planning Reassessment

- worker-failure containment is complete for the bounded onset-feature seam
- tempo and meter policy normalization remains deferred beyond this tranche
- the next honest strict seam is the near-copy tempo-state pair in
  `tempo_state_snap_integer_arm.rs` and
  `tempo_state_use_refined_stable_arm.rs`
- promoted new ready card:
  `docs/specs/batch-cards/014-g09-010-tempo-state-arm-unification.md`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/014-g09-010-tempo-state-arm-unification.md`.
