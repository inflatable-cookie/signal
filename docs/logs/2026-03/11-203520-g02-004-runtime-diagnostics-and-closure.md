# 2026-03-11 20:35:20 GMT - g02.004 runtime diagnostics and closure

Closed `g02.004` by freezing the runtime-facing loudness boundary and pinning
the multichannel/sample-rate behavior against stronger decibel-scale reference
expectations.

This closing batch matters because Signal now distinguishes:

- the full offline loudness result surface for catalog or delivery analysis,
- a compact bounded runtime-diagnostics loudness summary, and
- the expected loudness deltas for duplicated channels and gain changes rather
  than only monotonic ordering.

Implemented changes:

- extended `crates/signal-analysis-loudness/src/lib.rs` with:
  - `LoudnessRuntimeDiagnosticsSummary`
  - bounded recent-tail rules for momentary and short-term trace reuse
- updated `LoudnessAnalysisResult` so loudness analysis now exposes a stable
  `runtime_diagnostics_summary()` helper that keeps:
  - integrated loudness
  - true peak
  - target offset and peak-to-loudness spread
  - current momentary and short-term loudness
  - bounded recent momentary and short-term trace tails
- tightened loudness fixture expectations to pin:
  - stereo duplicate energy near `+3.01 LU`
  - four-channel duplicate energy near `+6.02 LU`
  - amplitude scaling near `20 * log10(gain ratio)`
- aligned the loudness feature reference and roadmap state so `g02.004` is
  complete and `g02.005` is the next active milestone

Validation:

- `cargo fmt`
- `cargo test -p signal-analysis-loudness`
- `git diff --check`
- `effigy validate --repo .`
- `effigy test --repo .`

Remaining limits recorded at close:

- named surround/speaker-role weighting is still not implemented
- non-`48 kHz` weighting remains an approximation or fallback surface
- wider external-tool parity work is still future scope

Next task:

Open `g02.005` by turning `signal-analysis-character` into a real descriptor-pack
surface: add practical spectral descriptors, freeze their reduction policy, and
group them into reusable packs before deepening transient-shape work.
