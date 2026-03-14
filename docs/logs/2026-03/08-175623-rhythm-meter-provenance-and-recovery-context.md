# Rhythm Meter Provenance And Recovery Context

Date: 2026-03-08
Owner: core-product

## Summary

Extended the public rhythm result surface so meter claims now carry explicit
provenance and recovery metadata. Signal can now say whether a meter estimate
came from whole-track evidence or from segment recovery, expose a lightweight
confidence breakdown, and report the recovered span when a late stable region is
what actually won.

## Work completed

- added public meter metadata types in `signal-analysis-rhythm`:
  - `MeterDetectionKind`
  - `MeterConfidenceBreakdown`
  - `MeterRecoveryContext`
- extended `MeterEstimate` so every meter claim now includes:
  - `detection_kind`
  - `confidence_breakdown`
  - `recovery`
- updated the internal meter-confidence path so local window candidates keep
  confidence breakdown data instead of collapsing immediately to one scalar
- updated `infer_meter(...)` to compare qualified whole-track and segment
  candidates and prefer segment recovery when the local late-stability
  explanation is clearly stronger on destabilized material
- updated the offline rhythm example to print meter provenance, confidence
  breakdown, and recovery span metadata
- tightened the test surface so Signal now explicitly checks:
  - stable structured-harmony meter is `WholeTrack`
  - sustained late reset/re-entry meter can be `SegmentRecovery`
  - recovered meter exposes a non-empty recovery span with multiple supporting
    windows

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- Running `effigy test` and `effigy validate` in parallel
  still reproduces the known workspace lock conflict, so validation was rerun
  serially and passed cleanly.
- One older relative meter-confidence ordering assertion became brittle once the
  provenance-aware chooser was added, so it was removed in favor of stronger
  explicit provenance and recovery-contract checks.

## Next Task

Deepen the public meter surface with recovery-duration confidence calibration,
such as explicit confidence components for whole-track support versus
segment-recovery strength, then tune how Finch should interpret weak segment
recoveries versus strong whole-track claims without product-specific heuristics.
