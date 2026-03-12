# g02 Milestones

Status: complete
Updated: 2026-03-11

## Why this generation matters now

`g01` made Signal a credible Rust-owned DSP/runtime workspace. `g02` should
turn that substrate into a deeper reusable analysis stack so Loophole, Finch,
and future consumers are not forced to rebuild core algorithmic functionality
in app-local repos.

The dependency order for this generation is:

1. shared streaming spectral and resampling substrate first
2. domain analyzers second
3. embedding/semantic inference only after descriptor surfaces are stable
4. benchmarking and acceptance hardening after the major algorithm crates have
   real outputs worth protecting

## Milestone map

- `g02.001` `complete`
  - streaming spectral, resampling, and analysis-rate substrate
- `g02.002` `complete`
  - rhythm structure, downbeat, and tempo-continuity depth
- `g02.003` `complete`
  - tonal analysis, tuning estimation, and harmonic tracking
- `g02.004` `complete`
  - loudness, true-peak, and multichannel dynamics depth
- `g02.005` `complete`
  - transient, timbral, and descriptor feature packs
- `g02.006` `complete`
  - embedding and semantic-analysis inference baseline
- `g02.007` `complete`
  - analysis corpus, benchmarking, and acceptance harnesses

## Current sequencing rule

`g02` is complete for this thread. It remains the closure spine for Signal's
first reusable deep-analysis generation until a later generation is explicitly
opened.

Working rules for that thread:

- keep generic DSP and analysis in reusable crates rather than host binaries
- prefer shared offline/streaming surfaces over analyzer-local one-off helpers
- keep real-time-safe pieces separate from offline-heavy convenience layers
- make confidence, ambiguity, and diagnostic outputs first-class instead of
  collapsing them into a single top-line label
- validate against concrete fixtures, corpus baselines, or external reference
  expectations before broadening scope again

## Next Task

`g02` is complete. Continue with `g03.001` now that the next Signal runway is
engine-oriented runtime depth rather than another DSP/analysis breadth pass.
