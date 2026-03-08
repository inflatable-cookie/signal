# Offline Loudness Slice Bootstrap

Date: 2026-03-08
Owner: core-product

## Summary

Expanded the Rust workspace with the first usable loudness-analysis baseline.

## Work completed

- implemented a first `signal-analysis-loudness` path with:
  - 48 kHz K-weighting baseline
  - gated integrated loudness
  - simple loudness-range estimate
  - 4x linear-interpolated true peak
  - confidence that drops on unsupported sample rates
- added a tiny offline loudness demo harness:
  - `cargo run -p signal-analysis-loudness --example offline_loudness_demo -- --amplitude 0.2 --sample-rate 48000 --seconds 4`
- added focused tests for:
  - silence handling
  - louder-vs-quieter ordering
  - reduced confidence on unsupported sample rates

## Validation

- `cargo test -p signal-analysis-loudness`
- `cargo test --workspace`
- `cargo run -p signal-analysis-loudness --example offline_loudness_demo -- --amplitude 0.2 --sample-rate 48000 --seconds 4`
- `git diff --check`

## Notes

This is a baseline, not the full finished EBU/ITU meter:

- K-weighting is only exact for the 48 kHz coefficient set used here
- multi-channel weighting is not implemented yet
- loudness range is a simplified percentile-based estimate

The 48 kHz demo sine at amplitude `0.2` measured approximately:

- integrated loudness: `-16.938 LUFS`
- loudness range: `0.009 LU`
- true peak: `-13.979 dBTP`

## Next Task

Pick the next DSP-deepening batch:

1. upgrade tonal analysis toward HPCP, tuning correction, and stronger profile
   handling, or
2. upgrade rhythm analysis toward stronger multifeature onset and tempo
   tracking, or
3. harden loudness with multi-rate filter coefficients and multi-channel
   weighting.
