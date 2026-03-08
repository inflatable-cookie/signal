# Rhythm Fixture System And Calibration Comparisons

Date: 2026-03-08
Owner: core-product

## Summary

Turned the newer arrangement-style rhythm tests into a reusable fixture system
and added comparison-oriented calibration checks so tempo ambiguity and meter
confidence can now be evaluated across scenarios instead of only in isolated
one-off tests.

## Work completed

- added reusable test-only fixture structures in `signal-analysis-rhythm`:
  - `GrooveSection`
  - `FixtureBuilder`
- refactored the richer four-four arrangement tests to build from those helpers
  instead of hand-assembling beat/tone vectors each time
- added calibration-oriented comparison tests that verify:
  - neutral click fixtures stay meter-unknown while structured four-four
    fixtures produce confident meter
  - subdivided/double-time-prone fixtures report higher tempo ambiguity than
    stable pulse fixtures
- kept the earlier adversarial rhythm tests intact so the new calibration layer
  complements, rather than replaces, direct edge-case assertions

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `git diff --check`

## Notes

- The rhythm crate now carries 17 targeted tests with both absolute assertions
  and scenario-to-scenario calibration checks.
- This should make the next rhythm tuning rounds cheaper because new fixtures
  can be composed from the same builder instead of copying bespoke beat/tone
  setup code.
- The calibration comparisons are still synthetic, but they are a better guard
  against accidental confidence/ambiguity regressions than threshold-only tests.

## Next Task

Add one more layer of realism by extending the fixture builder to support
explicit section metadata such as groove dropout, fill bars, and harmonic-rhythm
changes, then compare rhythm outputs across those transitions to tune confidence,
ambiguity, and unknown-meter behavior before Finch integration.
