# Rhythm Tempo Consumption Resolution

Date: 2026-03-09
Owner: core-product

## Summary

Shifted the tempo continuity line from metadata deepening into consumer fit by
adding a compact tempo consumption resolver on top of the existing analysis
surface. Signal can now tell downstream code which tempo to use now and which
tempo to fall back to next, without forcing wrappers to reconstruct that
decision from `tempo_interpretation`, `tempo_state`, and the continuity arc by
hand.

## Work completed

- added `TempoConsumptionSource`, `TempoConsumptionSelection`, and
  `TempoConsumptionDecision` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- added `BeatAnalysisResult::tempo_consumption(prior_bpm)` so callers can
  resolve:
  - current tempo source and BPM
  - fallback tempo source and BPM
  - fallback beat horizon
  - top-level action and continuity action
- kept the resolver Signal-owned and continuity-aware:
  - snapped lock states resolve to snapped current tempo
  - core-window monitor states resolve to core-window tempo with optional prior
    tempo fallback
  - guarded refined monitor states resolve to current refined tempo with clear
    fallback
  - cleared tempo states resolve to no current and no fallback tempo
- updated `offline_rhythm_demo` to print a compact tempo consumption line ahead
  of the deeper continuity diagnostics
- added end-to-end tests against real analyzed material, not just synthetic
  interpretation stubs:
  - neutral integer click lock
  - slower click core-window carry with and without prior tempo
  - weak-backbeat guarded refined reacquisition
  - silence clear/defer path

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- This batch is intended to make the current tempo continuity work directly
  usable by Finch without inventing another interpretation layer.
- The resolver is intentionally compact and consumer-facing. The deeper
  continuity arc, rationale, inflection, and support fields remain available for
  debugging, calibration, and future runtime policy.

## Next Task

Run the tempo consumption resolver through realistic offline fixtures or a
Finch-facing integration seam and tune any thresholds that feel too eager or too
conservative once the resolved current/fallback tempo choices are consumed
outside the synthetic calibration path.
