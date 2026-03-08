# Rhythm Tail Meter Promotion Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Retuned `signal-analysis-rhythm` meter promotion so stable whole-track bar
patterns still surface meter, while sparse-harmony and modulation-heavy
fixtures no longer get promoted through the trailing recovery window. The
scorer now carries explicit meter-cue support metrics in each hypothesis, but
the main behavioral change is a stricter tail-promotion gate built around
regularity and recent stability instead of broad meter promotion.

## Work completed

- extended `MeterHypothesis` with explicit meter-cue support fields so meter
  scoring can distinguish general beat support from actual bar-cue support
- kept the global meter path available for stable accent-driven and sectioned
  patterns by continuing to gate on whole-track score, confidence, support, and
  regularity
- tightened the trailing segment-aware promotion path in `infer_meter(...)` so
  late recovery only promotes meter when the tail is genuinely stable:
  - higher minimum tail score and confidence
  - stronger support requirement
  - higher regularity requirement
  - stronger recent-strength requirement
- preserved the current Signal-owned contract from earlier batches:
  - sparse harmonic-rhythm fixtures stay `meter: None`
  - modulation-plus-dense-fill transition fixtures stay `meter: None`
  - stable four-four, pickup, late-shift, weak-backbeat, and section-transition
    fixtures still surface meter correctly

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- An initial parallel attempt to run `effigy test --repo .` and
  `effigy validate --repo .` reproduced the repo's workspace lock conflict; the
  validation run passed once rerun serially.
- This batch keeps the segment-aware recovery path in place, but makes it much
  less eager to claim meter from partially recovered tails.

## Next Task

Deepen the segment-aware meter pass so it can compare multiple adjacent windows
instead of only a single trailing recovery slice, then tune when meter should
reappear after sustained late stability versus staying `None` through prolonged
mixed or modulation-heavy transitions.
