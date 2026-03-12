# 2026-03-11 20:08:56 GMT - g02.004 multichannel loudness opening tranche

Opened `g02.004` by replacing the old mono-mixdown loudness path with an
explicit channel-aware aggregation contract in `signal-analysis-loudness`.

This tranche matters because loudness consumers can now see how integrated
loudness was aggregated across channels and whether the result came from native
`48 kHz` weighting, resampled `48 kHz` weighting, or a deterministic fallback
path.

Implemented changes:

- extended `crates/signal-analysis-loudness/src/lib.rs` with:
  - `LoudnessChannelWeightSource`
  - `LoudnessSampleRateSupport`
  - `LoudnessChannelSummary`
  - `LoudnessAggregationSummary`
- updated `LoudnessAnalysisResult` so loudness analysis now reports:
  - per-channel loudness and true-peak summaries
  - aggregation metadata for channel weighting and sample-rate support
- rewired `LoudnessMeter` to:
  - preserve channel layout instead of forcing analyzer-wide mono prep
  - aggregate per-channel block energies under explicit mono/stereo/fallback
    weighting
  - vary true-peak oversample factor by analysis sample rate
  - keep deterministic fallback behavior when exact weighting parity is not yet
    implemented
- added fixture coverage for:
  - stereo equal-weight aggregation
  - counted multichannel fallback behavior
  - resampled-to-`48 kHz` support reporting
  - non-`48 kHz` unweighted fallback reporting
- aligned the loudness feature reference and roadmap state to the new opening
  `004.1` evidence

Validation:

- `cargo fmt`
- `cargo test -p signal-analysis-loudness`

Next task:

Continue `g02.004` by adding short-term and momentary loudness trace surfaces
plus richer dynamics summaries on top of the new aggregation contract, then
decide which of those outputs should be treated as runtime-diagnostics
boundaries instead of analyzer-local detail.
