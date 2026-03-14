# Rhythm Tempo Interpretation Recommendations

Date: 2026-03-08
Owner: core-product

## Summary

Added a Signal-owned tempo interpretation layer on top of the new local-tempo,
 trend, and beat-grid diagnostics so downstream consumers can use explicit tempo
 recommendations instead of inventing their own BPM-selection heuristics.

## Work completed

- added `TempoInterpretation` to `BeatAnalysisResult` in
  `crates/signal-analysis-rhythm/src/lib.rs`
- added public tempo recommendation semantics:
  - `TempoTrustLevel`
  - `TempoRecommendation`
  - `TempoInterpretationReason`
  - `TempoInterpretationSupport`
- added interpretation support signals derived from the existing diagnostics:
  - core-window consensus
  - drift stability
  - beat-grid stability
  - integer closeness
  - boundary pressure
- added a new interpretation pass that recommends one of:
  - `UseRefined`
  - `UseCoreWindow`
  - `SnapInteger`
  - `Defer`
- calibrated the interpretation path so it can distinguish:
  - clean near-integer pulse cases that are appropriate for integer snapping
  - edge-skewed but stable-core cases that should prefer the core-window tempo
  - unstable or ambiguous cases that should defer tempo locking
- updated `offline_rhythm_demo` to print tempo interpretation and support
  details

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- The intended public contract after this batch is:
  - stable clean pulse -> `SnapInteger` or `UseRefined`
  - stable core with edge skew -> `UseCoreWindow`
  - unstable or ambiguous pulse -> `Defer`
- One environment/runtime issue remains unresolved: after this batch the Rust
  test binary and `offline_rhythm_demo` example no longer start cleanly in this
  environment, even for direct invocations like `--list`, despite successful
  compile-level Rust validation. I did not count executable Rust test runs as a
  passing validation signal for this batch.
- Follow-up isolation suggests this is broader than `signal-analysis-rhythm`:
  trivial direct Rust and C hello-world binaries also stalled under the same
  command-execution path here, while the repo-owned Effigy/CTest validation path
  still executed binaries normally.

## Next Task

Resolve the current Rust binary startup regression so the new tempo
 interpretation path can be exercised at runtime again, then run calibration
 fixtures to tune whether stable near-integer material should prefer
 `SnapInteger` or `UseRefined` and when edge-skewed material should downgrade
 from `UseCoreWindow` to `Defer`.
