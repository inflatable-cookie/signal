# Rhythm Multiwindow Segment Meter Recovery

Date: 2026-03-08
Owner: core-product

## Summary

Replaced the old single trailing-window meter promotion with a multiwindow
segment-aware recovery pass. Signal now compares several recent local windows,
clusters agreeing meter hypotheses, and only promotes meter when late stability
is sustained across adjacent windows rather than appearing in one isolated tail
slice.

## Work completed

- added local meter-window modeling to `signal-analysis-rhythm`:
  - `MeterWindowCandidate`
  - `meter_window_candidate(...)`
  - `select_segment_meter_candidate(...)`
- updated `infer_meter(...)` so local recovery now requires:
  - several recent recoverable windows
  - agreement on beats-per-bar and absolute phase
  - sustained coverage across adjacent end windows
  - improvement relative to the immediately preceding destabilized region
- preserved the existing whole-track meter path for stable fixtures, while
  making the local promotion path explicitly segment-oriented
- added stronger transition presets to the shared rhythm fixture surface:
  - `ReentryAcceleratingHarmonySustainedReset`
  - `ReentryDeceleratingHarmonySustainedReset`
  - `ModulationDenseFillExtended`
- updated the named preset expectations and transition calibration tests so the
  current Signal-owned contract is explicit:
  - sustained late reset/re-entry can recover `meter: Some(4)`
  - shorter reset and cadential re-anchor cases still remain `meter: None`
  - prolonged modulation-heavy dense transitions stay `meter: None`

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- This batch materially changes the rhythm surface: segment-aware recovery is no
  longer just an internal heuristic, because the fixture family now codifies
  both successful late recovery and prolonged no-meter outcomes.
- Effigy validation was run serially to avoid the known workspace lock conflict
  when multiple repo-owned tasks overlap.

## Next Task

Deepen the segment-aware meter pass with explicit recovery-duration metadata or
confidence decomposition, then calibrate how quickly `meter` should return and
how strongly it should be trusted across section re-entry, cadence, and
modulation-heavy transitions before Finch relies on fixed threshold behavior.
