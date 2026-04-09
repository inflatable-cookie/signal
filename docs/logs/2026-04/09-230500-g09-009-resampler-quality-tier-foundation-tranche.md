# 2026-04-09 - g09.009 Resampler Quality-Tier Foundation Tranche

## Summary

Completed the first strict `g09.009` batch in `signal-dsp-resample`.

## Implementation

- added `ResampleQuality::BandLimited` in
  `crates/signal-dsp-resample/src/lib.rs`
- kept `Nearest` and `Linear` explicit as lower-quality deterministic modes
- implemented a bounded windowed-sinc low-pass path for higher-quality
  downsampling behavior
- updated streaming drain logic so chunked and offline `BandLimited` output
  remains deterministic and consistent
- added focused tests for chunked/offline equivalence and material attenuation
  of alias-prone content

## Validation

- `cargo test -p signal-dsp-resample`
- `cargo check -p signal-analysis-embed`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Reassessment

The next honest `g09.009` seam is still inside resampler proof, not semantics
yet. The crate now has a real quality-tier foundation, but it still wants one
explicit comparative evidence surface before the lane should switch to semantic
calibration.

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/010-g09-009-resampler-proof-and-benchmark-surface.md`.
