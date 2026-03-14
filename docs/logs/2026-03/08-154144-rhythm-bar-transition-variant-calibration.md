# Rhythm Bar Transition Variant Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Added a dedicated bar-transition preset family to the rhythm test surface so
pickup bars, delayed downbeat accents, and mixed bar-length disruptions can be
compared systematically instead of living only as isolated tests.

## Work completed

- introduced a new `BarTransitionVariant` family covering:
  - pickup into stable four-four
  - delayed downbeat/accent shift within otherwise regular four-four
  - mixed bar-length disruption that should collapse meter
- added a shared bar-transition preset builder so these cases flow through the
  same named preset surface as the existing harmony, fill-density, and dropout
  families
- expanded the named preset expectation table to include the new transition
  family, keeping bar-structure behavior visible at the preset-surface level
- added a dedicated monotonicity-style transition calibration test that checks:
  - pickup and delayed-shift cases retain four-four meter
  - delayed-shift cases increase ambiguity relative to pickup
  - mixed-length disruption falls back to `meter: None`

## Validation

- `cargo test -p signal-analysis-rhythm`
- `effigy validate`
- `cargo test --workspace`:
  failed in unrelated crate `signal-plugin-clap` because
  `PluginMessagePayload` initializers are missing the new
  `processing_epoch` and `shared_memory_lease_id` fields
- `git diff --check`

## Notes

- The rhythm crate now has 26 targeted tests in the root package test path.
- This batch keeps the scope inside Signal’s reusable rhythm analysis surface;
  the workspace failure is outside the touched rhythm code.
- The transition family now makes a clearer distinction between:
  - four-four recovery after a pickup
  - accent displacement that should stress confidence/ambiguity without
    destroying meter
  - genuine bar-structure disruption that should suppress meter entirely

## Next Task

Deepen the bar-transition family with section-level bar changes such as
temporary meter modulation, downbeat re-entry after dropout, and cadence-like
bar elongation, then tune how meter confidence recovers or stays unknown across
those transitions before Finch depends on the result surface.
