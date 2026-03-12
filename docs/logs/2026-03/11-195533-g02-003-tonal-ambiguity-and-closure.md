# 2026-03-11 19:55:33 GMT - g02.003 tonal ambiguity and closure

Closed `g02.003` by making weak and mixed local tonality explicit in the
public tonal analysis surface, then aligning the feature-reference and roadmap
state to the completed milestone.

This batch matters because downstream consumers can now distinguish:

- a stable global or local key,
- a one-way modulation event,
- recurring mixed-tonality material, and
- weak tonal-centre material that should not be treated as a firm key label.

Implemented changes:

- extended `crates/signal-analysis-tonal/src/lib.rs` with:
  - `TonalAmbiguityKind`
  - `TonalSegmentAmbiguitySummary`
  - `LocalTonalAmbiguitySummary`
- updated the tonal local-tracking surface so it now reports:
  - segment-local ambiguity evidence
  - track-level local ambiguity summaries
  - explicit modulation and mixed-tonality classification on the existing
    windowed tonal path
- added fixture coverage for:
  - stable local key tracking without false ambiguity promotion
  - explicit `C -> G` modulation
  - weak-tonal-centre material
  - recurring `C -> G -> C` mixed-tonality material
- aligned `docs/architecture/dsp-analysis-feature-reference.md` to the deeper
  tuning, local-tracking, and ambiguity surface
- updated the `g02` roadmap state so `g02.003` is complete and `g02.004` is
  the next active milestone

Validation:

- `cargo fmt`
- `cargo test -p signal-analysis-tonal`

Remaining limits recorded at close:

- local tonal tracking is still fixed-window and offline-oriented
- no chord transcription or harmonic-function interpretation is exposed yet
- broader tonal calibration against real-world corpora remains future work

Next task:

Open `g02.004` with an explicit multichannel loudness aggregation contract and
broader true-peak/sample-rate behavior in `signal-analysis-loudness`, then pin
deterministic fallback behavior before deepening trace surfaces.
