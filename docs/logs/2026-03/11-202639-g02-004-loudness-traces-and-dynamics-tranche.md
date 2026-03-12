# 2026-03-11 20:26:39 GMT - g02.004 loudness traces and dynamics tranche

Extended `g02.004` by making loudness movement and summary dynamics explicit
on top of the new multichannel aggregation contract.

This tranche matters because downstream delivery and monitoring consumers no
longer have to reconstruct loudness motion from one integrated LUFS figure and
one top-line range value.

Implemented changes:

- extended `crates/signal-analysis-loudness/src/lib.rs` with:
  - `LoudnessTracePoint`
  - `LoudnessTrace`
  - `LoudnessDynamicsSummary`
- updated `LoudnessAnalysisResult` so loudness analysis now reports:
  - momentary trace output
  - short-term trace output
  - dynamics summary fields such as target offset, peak-to-loudness spread,
    and trace-derived maxima/ranges
- kept those outputs on the same aggregated loudness path used for:
  - multichannel channel-weighting behavior
  - sample-rate support reporting
  - true-peak estimation
- added fixture coverage for:
  - level-step material that produces later louder momentary windows
  - dynamics summaries reacting to louder sections instead of static outputs
- aligned the loudness feature reference and roadmap state to the new trace and
  dynamics surface

Validation:

- `cargo fmt`
- `cargo test -p signal-analysis-loudness`

Next task:

Continue `g02.004` by deciding which of the new trace and dynamics outputs
should become stable runtime-diagnostics boundaries, then compare the current
multichannel/sample-rate behavior against stronger external reference
expectations before closing the milestone.
