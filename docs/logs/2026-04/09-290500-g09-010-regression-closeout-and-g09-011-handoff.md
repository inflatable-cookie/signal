# 2026-04-09 - g09.010 Regression Closeout And g09.011 Handoff

## Summary

Closed strict card `018-g09-010-rhythm-regression-corpus-freeze` after adding
one explicit preset-driven regression bundle across the normalized tempo and
meter policy surfaces, then promoted `g09.011` as the next active strict
milestone.

## Implementation

- added `rhythm_tests/rhythm_regression_corpus.rs` as one grouped regression
  bundle for the stabilized post-normalization tempo and meter posture
- kept worker-containment proof in the existing `onset_features.rs` focused
  tests and used it as the failure-containment validation signal for this
  tranche
- closed `g09.010` and promoted the next strict ready card:
  `docs/roadmaps/g09/batch-cards/019-g09-011-demo-program-shape.md`

## Validation

- `cargo test -p signal-analysis-rhythm rhythm_tests::rhythm_regression_corpus::rhythm_regression_bundle_preserves_post_normalization_tempo_and_meter_surface -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm onset_features::tests::reduced_onset_containment_recovers_from_worker_panic -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm onset_features::tests::full_onset_containment_zero_fills_multiple_failed_workers_deterministically -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm rhythm_tests::tempo_state_policy_unification::tempo_state_stable_policy_preserves_integer_and_refined_divergence -- --exact --nocapture --test-threads=1`
- `cargo check -p signal-analysis-rhythm`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Notes

- the regression bundle records preserved posture across the normalized tempo
  and meter continuity surfaces; no intentional recommendation shift was
  introduced in this tranche
- interactive rhythm demo work remains deferred to the demo-substrate
  milestones under `g09.011+`

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/019-g09-011-demo-program-shape.md`.
