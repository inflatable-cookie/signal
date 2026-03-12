# 2026-03-11 22:28:09 GMT - g02.007 acceptance evidence and residual-risk tranche

Continued `g02.007` by recording explicit shared-harness acceptance evidence
for the frozen analyzer families and by writing down the remaining residual
risks for rhythm and tonal policy coverage.

This tranche matters because the acceptance spine now has reproducible evidence
with case-level metrics and timings, not just threshold definitions.

Acceptance evidence commands:

- `cargo test -p signal-analysis-character frozen_character_acceptance_report_remains_interpretable_for_closeout -- --nocapture`
- `cargo test -p signal-analysis-loudness frozen_loudness_acceptance_report_remains_interpretable_for_closeout -- --nocapture`
- `cargo test -p signal-analysis-embed frozen_semantic_acceptance_report_remains_interpretable_for_closeout -- --nocapture`
- `effigy acceptance:analysis --repo .`

Frozen-family acceptance evidence:

- character descriptor family:
  - report status: `Pass`
  - `character:tone:sine440`
    - `spectral_flatness = 1.06366e-9`
    - `rms_energy = 0.7071067`
    - `sustain_ratio = 0.98833334`
    - `descriptor_confidence = 0.2`
    - elapsed: about `200.6 ms`
  - `character:noise:deterministic`
    - `spectral_spread_hz = 2589.5632`
    - `rms_energy = 0.49804345`
    - `sustain_ratio = 1.0`
    - `descriptor_confidence = 0.2`
    - elapsed: about `201.1 ms`
  - `character:pulse:adsr`
    - `peak_transient_strength = 1.0`
    - `descriptor_confidence = 0.3`
    - elapsed: about `256.8 ms`
- loudness family:
  - report status: `Pass`
  - `loudness:quiet-sine`
    - `true_peak_dbtp = -20.0`
    - `confidence = 1.0`
    - elapsed: about `63.3 ms`
  - `loudness:loud-sine`
    - `true_peak_dbtp = -6.0206003`
    - `confidence = 1.0`
    - elapsed: about `39.3 ms`
  - `loudness:level-step`
    - `loudness_range_lu = 18.989187`
    - `momentary_range_lu = 22.711452`
    - `confidence = 1.0`
    - elapsed: about `103.8 ms`
- semantic family:
  - report status: `Pass`
  - `semantic:tone:sine440`
    - `tonal_focus_score = 0.674288`
    - `semantic_confidence = 0.053420015`
    - elapsed: about `276.2 ms`
  - `semantic:noise:deterministic`
    - `textural_noise_score = 0.5797433`
    - `semantic_confidence = 0.041505113`
    - elapsed: about `197.9 ms`
  - `semantic:pulse:adsr`
    - `pulse_driven_score = 0.72227913`
    - `dynamic_punch_score = 0.7304088`
    - `semantic_confidence = 0.061119247`
    - `descriptor_confidence = 0.3`
    - elapsed: about `254.8 ms`

Residual-risk notes:

- `signal-analysis-rhythm` is still outside the frozen policy set.
  - why: the family already exposes ambiguity, continuity, and recovery
    surfaces that need case design richer than single-label acceptance bands.
  - concrete risk: tempo and meter outputs could regress in ambiguous,
    modulation, or dropout-heavy material without a shared corpus gate.
- `signal-analysis-tonal` is still outside the frozen policy set.
  - why: key, tuning, and local-tracking evidence need ambiguity-aware policy
    rules so the acceptance surface does not collapse nuanced tonal outputs into
    brittle pass/fail labels.
  - concrete risk: closely related major/minor or detuned material could drift
    without a shared acceptance report catching the change.

Backlog candidates for the next tranche:

- add rhythm corpus cases for:
  - steady click tracks
  - sparse/dropout sections
  - accelerando/decelerando passages
  - competing-meter ambiguity
- add tonal corpus cases for:
  - clear major/minor triads
  - relative-major/minor ambiguity
  - detuned references
  - sectional harmonic change

Validation:

- `cargo fmt --all`
- `cargo test -p signal-analysis-character frozen_character_acceptance_report_remains_interpretable_for_closeout -- --nocapture`
- `cargo test -p signal-analysis-loudness frozen_loudness_acceptance_report_remains_interpretable_for_closeout -- --nocapture`
- `cargo test -p signal-analysis-embed frozen_semantic_acceptance_report_remains_interpretable_for_closeout -- --nocapture`
- `git diff --check`
- `effigy acceptance:analysis --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`

Next task:

Broaden `g02.007` to rhythm and tonal by defining ambiguity-aware acceptance
policies and first corpus cases for those families before attempting final
closeout.
