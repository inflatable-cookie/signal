# Frozen Family Policies v1

Status: active
Updated: 2026-03-11
Scope: `g02.007` first frozen analyzer-family acceptance and drift policies

## Purpose

This manifest freezes the first practical acceptance posture for Signal's
shared analysis corpus. It is intentionally small and synthetic-first: the goal
is to protect stable analyzer behavior without pretending the current corpus is
large enough for research-grade precision.

## Families Covered

- `signal-analysis-rhythm`
- `signal-analysis-tonal`
- `signal-analysis-character`
- `signal-analysis-loudness`
- `signal-analysis-embed`

## Shared Policy Rules

- acceptance thresholds should stay broad enough to survive harmless numeric
  implementation changes
- drift limits should protect user-visible behavior, not every internal float
- `elapsed_ms` is recorded by the harness for every case, but this `v1` policy
  treats performance as report-only evidence rather than a hard fail gate
- confidence metrics should be thresholded only where the analyzer already
  exposes an explicit bounded confidence surface

## Rhythm Policy

Canonical synthetic cases:

- `rhythm:steady-click120`
- `rhythm:structured-harmony120`
- `rhythm:ambiguous-subdivision90`

Acceptance metrics:

- steady click:
  - `bpm` in `119.9..=120.1`
  - `confidence` in `0.2..=1.0`
  - `tempo_ambiguity` in `0.0..=0.4`
  - `has_meter = 0`
- structured harmony:
  - `bpm` in `118.0..=122.0`
  - `has_meter = 1`
  - `beats_per_bar = 4`
  - `meter_confidence` in `0.2..=1.0`
  - `structure_bar_count >= 4`
  - `recovered_bar_count = 0`
- ambiguous subdivision:
  - `bpm` in `88.0..=92.0`
  - `confidence` in `0.1..=1.0`
  - `tempo_ambiguity` in `0.2..=1.0`
  - `has_meter = 0`

Drift posture:

- protect `bpm` within `0.25`
- protect `tempo_ambiguity` within `0.08`
- protect `meter_confidence` within `0.10`
- protect `has_meter` and `beats_per_bar` as exact contract signals

## Tonal Policy

Canonical synthetic cases:

- `tonal:c-major-triad`
- `tonal:detuned-c-major-432`
- `tonal:modulation-c-to-g`

Acceptance metrics:

- clear C major:
  - `key_tonic = 0`
  - `key_mode = 0`
  - `confidence` in `0.01..=1.0`
  - `tuning_reference_hz` in `438.0..=442.0`
  - `local_ambiguity_count = 0`
- detuned C major:
  - `key_tonic = 0`
  - `key_mode = 0`
  - `tuning_reference_hz` in `429.5..=434.5`
  - `tuning_cents_offset` in `-40.0..=-20.0`
- modulation:
  - `local_segment_count >= 2`
  - `local_change_count >= 1`
  - `modulation_ambiguity_count >= 1`
  - `first_segment_tonic = 0`
  - `last_segment_tonic = 7`

Drift posture:

- protect `key_tonic` and `key_mode` as exact contract signals
- protect `tuning_reference_hz` within `2.5 Hz`
- protect `tuning_cents_offset` within `8 cents`
- protect `local_change_count` and `modulation_ambiguity_count` as exact-or-higher
  evidence signals

## Character Descriptor Policy

Canonical synthetic cases:

- `character:tone:sine440`
- `character:noise:deterministic`
- `character:pulse:adsr`

Acceptance metrics:

- tone:
  - `spectral_flatness` in `0.0..=0.05`
  - `rms_energy` in `0.65..=0.75`
  - `sustain_ratio` in `0.95..=1.0`
  - `descriptor_confidence` in `0.15..=1.0`
- noise:
  - `spectral_spread_hz >= 2000`
  - `rms_energy` in `0.45..=0.55`
  - `sustain_ratio` in `0.95..=1.0`
  - `descriptor_confidence` in `0.15..=1.0`
- pulse:
  - `peak_transient_strength` in `0.80..=1.0`
  - `descriptor_confidence` in `0.25..=1.0`

Drift posture:

- protect descriptor confidence within `0.08` absolute drift
- protect tone `spectral_flatness` within `0.03` absolute drift
- protect pulse `peak_transient_strength` within `0.10` absolute drift

## Loudness Policy

Canonical synthetic cases:

- `loudness:quiet-sine`
- `loudness:loud-sine`
- `loudness:level-step`

Acceptance metrics:

- quiet sine:
  - `true_peak_dbtp` in `-20.5..=-19.5`
  - `confidence` in `0.9..=1.0`
- loud sine:
  - `true_peak_dbtp` in `-6.5..=-5.5`
  - `confidence` in `0.9..=1.0`
- level step:
  - `loudness_range_lu >= 5.0`
  - `momentary_range_lu >= 5.0`
  - `confidence` in `0.9..=1.0`

Drift posture:

- protect `true_peak_dbtp` within `0.2 dB`
- protect `integrated_lufs` within `0.75 LU`
- protect `loudness_range_lu` within `1.0 LU`

## Semantic Policy

Canonical synthetic cases:

- `semantic:tone:sine440`
- `semantic:noise:deterministic`
- `semantic:pulse:adsr`

Acceptance metrics:

- tone:
  - `tonal_focus_score` in `0.60..=1.0`
  - `semantic_confidence` in `0.03..=1.0`
- noise:
  - `textural_noise_score` in `0.50..=1.0`
  - `semantic_confidence` in `0.03..=1.0`
- pulse:
  - `pulse_driven_score` in `0.40..=1.0`
  - `dynamic_punch_score` in `0.40..=1.0`
  - `semantic_confidence` in `0.03..=1.0`
  - `descriptor_confidence` in `0.25..=1.0`

Drift posture:

- protect primary family tag score within `0.08` absolute drift
- protect `semantic_confidence` within `0.03` absolute drift
- protect `descriptor_confidence` within `0.08` absolute drift

## Next Task

`v1` is now the frozen closeout manifest for `g02`. Revise it only when a new
generation or a concrete regression class requires broader corpus policy.
