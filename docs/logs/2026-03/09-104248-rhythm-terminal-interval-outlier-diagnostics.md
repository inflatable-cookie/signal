# Rhythm Terminal Interval Outlier Diagnostics

Date: 2026-03-09
Owner: core-product

## Summary

Moved part of the BPM hardening one layer earlier in the rhythm path by adding
beat-interval outlier diagnostics and using a trimmed interval set during BPM
refinement. This keeps terminal beat misses from biasing the refined BPM before
tempo interpretation decides whether to snap or defer.

## Work completed

- added `BeatIntervalOutlierDiagnostics` to
  `crates/signal-analysis-rhythm/src/lib.rs` and published it through
  `TempoDiagnostics`
- added `filter_interval_outliers(...)` and reused it in:
  - `refine_bpm_from_beats(...)`
  - `analyze_local_tempo(...)`
- changed outlier edge counting to report rejected intervals in the leading and
  trailing edge windows, which matches the real-file tail behavior better than a
  purely contiguous check
- updated `file_rhythm_probe` to print the new interval-outlier diagnostics
- added regression coverage for:
  - localized terminal interval outliers
  - BPM refinement ignoring terminal outlier intervals
  - snapping a stable near-integer result when terminal outliers are localized

## Real-file result

Test file:
`~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav`

Current result after this batch:

- `bpm=128.00000`
- `tempo_interpretation=SnapInteger/NearIntegerPulse`
- `tempo_state=Lock/StableIntegerTempo`
- `refined_bpm=127.96191`
- `snap_error=0.03809`
- `interval_outliers=total:738/retained:670/rejected:68/leading:0/trailing:3`

The important new diagnostic is the trailing interval count: Signal now makes
it explicit that the Garamond master has localized tail instability without
having to pretend the whole track is tempo-unstable. The last few raw intervals
still show the same noisy tail, but BPM refinement and interpretation now treat
that as a localized recovery problem instead of a reason to miss the integer BPM.

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo check -p signal-analysis-rhythm --example file_rhythm_probe`
- `cargo run -p signal-analysis-rhythm --example file_rhythm_probe -- '~/Library/CloudStorage/Dropbox/Music/Projects/Garamond/1. 086/Output/1. 086 - v1.5 - Master Stream.wav'`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- The outlier filter is still intentionally conservative about protecting the
  integer result rather than rewriting the beat grid itself.
- The current Garamond probe still rejects more intervals overall than would be
  ideal (`68`), even though the terminal behavior is now explicit and the final
  BPM/state result is correct.

## Next Task

Use the new interval-outlier diagnostics to tune or segment the beat grid
itself, especially deciding whether localized tail outliers should be excluded
from published beat-grid summaries or whether the next pass should infer a
stable core span and report that explicitly alongside the full-track beat grid.
