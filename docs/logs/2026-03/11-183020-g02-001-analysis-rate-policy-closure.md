# 2026-03-11 18:30:20 GMT - g02.001 analysis-rate policy and closure

## Summary

Closed `g02.001` by freezing explicit analysis-rate defaults for the remaining
rhythm, tonal, and descriptor-oriented analyzers now that the shared
preparation/resampling substrate was already in place.

With this tranche, Signal's current analyzer families all share one input
preparation boundary and no longer inherit source sample rate implicitly. That
moves the remaining work out of substrate construction and into the
domain-specific milestones that build on it.

## What changed

- updated `crates/signal-analysis-rhythm/src/lib.rs` so `BeatTrackerConfig`
  now carries an explicit `analysis_sample_rate`, defaulting to 48 kHz across
  profiles and flowing through the shared preparation contract
- updated `crates/signal-analysis-tonal/src/lib.rs` so `KeyDetectorConfig`
  now carries an explicit `analysis_sample_rate`, defaulting to 48 kHz across
  profiles and flowing through the shared preparation contract
- updated `crates/signal-analysis-character/src/lib.rs` so
  `CharacterAnalyzerConfig` now carries an explicit `analysis_sample_rate`,
  defaulting to 48 kHz across profiles and flowing through the shared
  preparation contract
- added targeted coverage that pins frozen-rate behavior against non-native
  source inputs for:
  - click-track tempo stability
  - key stability
  - character descriptor stability
- marked `g02.001` complete and rolled `g02.002` active

## Validation

- `cargo fmt`
- `cargo test -p signal-analysis-rhythm -p signal-analysis-tonal -p signal-analysis-character`
- `git diff --check`

## Completion

`g02.001` is complete.
