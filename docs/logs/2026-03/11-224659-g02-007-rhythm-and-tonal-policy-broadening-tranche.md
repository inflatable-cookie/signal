# 2026-03-11 22:46:59 GMT - g02.007 rhythm and tonal policy broadening tranche

Broadened `g02.007` by bringing rhythm and tonal under the shared acceptance
spine with ambiguity-aware corpus cases, repo-owned thresholds, and explicit
closeout-evidence tests.

This tranche matters because the acceptance harness no longer skips the
ambiguity-heavy analyzer families. Signal now has one shared corpus policy
surface across rhythm, tonal, character, loudness, and semantic analysis.

Implemented changes:

- added shared-harness acceptance coverage to `signal-analysis-rhythm` for:
  - `rhythm:steady-click120`
  - `rhythm:structured-harmony120`
  - `rhythm:ambiguous-subdivision90`
- added shared-harness acceptance coverage to `signal-analysis-tonal` for:
  - `tonal:c-major-triad`
  - `tonal:detuned-c-major-432`
  - `tonal:modulation-c-to-g`
- kept the policy ambiguity-aware instead of top-line-label-only by freezing:
  - rhythm `tempo_ambiguity`, meter presence, beats-per-bar, and recovery-free
    structure expectations
  - tonal tuning-reference drift, local change count, and modulation ambiguity
    evidence
- added closeout-evidence report tests so the accepted harness outputs are
  printed directly from the analyzer crates rather than reconstructed later
- expanded `effigy acceptance:analysis --repo .` to include:
  - `signal-analysis-rhythm`
  - `signal-analysis-tonal`
  - the previously frozen character, loudness, and semantic families
- updated `fixtures/analysis-corpus/manifests/frozen-family-policies-v1.md`
  so the shared manifest now covers all current analyzer families in `g02`

Acceptance evidence highlights:

- rhythm family report status: `Pass`
  - `rhythm:steady-click120`
    - `bpm = 120.0`
    - `confidence = 0.9339566`
    - `tempo_ambiguity = 0.3531099`
    - `has_meter = 0`
  - `rhythm:structured-harmony120`
    - `bpm = 119.8407`
    - `has_meter = 1`
    - `beats_per_bar = 4`
    - `meter_confidence = 0.48859477`
    - `structure_bar_count = 6`
    - `recovered_bar_count = 0`
  - `rhythm:ambiguous-subdivision90`
    - `bpm = 90.00035`
    - `confidence = 0.70896035`
    - `tempo_ambiguity = 1.0`
    - `has_meter = 0`
- tonal family report status: `Pass`
  - `tonal:c-major-triad`
    - `key_tonic = 0`
    - `key_mode = 0`
    - `confidence = 0.15580657`
    - `tuning_reference_hz = 441.27258`
    - `local_ambiguity_count = 0`
  - `tonal:detuned-c-major-432`
    - `key_tonic = 0`
    - `key_mode = 0`
    - `tuning_reference_hz = 433.6918`
    - `tuning_cents_offset = -25.0`
  - `tonal:modulation-c-to-g`
    - `local_segment_count = 3`
    - `local_change_count = 1`
    - `modulation_ambiguity_count = 1`
    - `first_segment_tonic = 0`
    - `last_segment_tonic = 7`

Residual limits after broadening:

- the corpus is still synthetic-first and intentionally small
- performance is still recorded through harness elapsed time but not yet gated
- drift posture is documented in the manifest, but this tranche does not yet
  add full baseline-versus-candidate regression fixtures for every family

Validation:

- `cargo fmt --all`
- `cargo test -p signal-analysis-rhythm`
- `cargo test -p signal-analysis-tonal`
- `cargo test -p signal-analysis-rhythm frozen_rhythm_acceptance_report_remains_interpretable_for_closeout -- --nocapture`
- `cargo test -p signal-analysis-tonal frozen_tonal_acceptance_report_remains_interpretable_for_closeout -- --nocapture`

Next task:

Close `g02.007` by consolidating the full analyzer-family acceptance evidence,
recording remaining corpus limits, and deciding whether `g02` is ready to mark
complete.
