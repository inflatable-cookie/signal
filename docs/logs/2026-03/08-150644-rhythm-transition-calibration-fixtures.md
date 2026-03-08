# Rhythm Transition Calibration Fixtures

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm fixture system with explicit transition metadata so section
dropout, fill bars, and per-bar harmonic changes can be modeled declaratively,
then added transition-oriented calibration tests around confidence, ambiguity,
and unknown-meter behavior.

## Work completed

- widened the test-only `GrooveSection` model to support:
  - per-bar beat-pattern overrides
  - per-bar chord plans
  - explicit dropout-bar lists
- updated the four-four groove builder so it can synthesize:
  - mild section dropouts
  - dropout-heavy bars that should collapse meter confidence
  - fill bars with different beat emphasis
  - section-local harmonic rhythm changes
- added transition-oriented comparison tests covering:
  - confidence drop from steady sections to mild-dropout sections
  - four-four preservation through a fill bar plus harmonic-rhythm changes
  - explicit unknown-meter fallback on dropout-heavy transition fixtures

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `git diff --check`

## Notes

- The rhythm crate now carries 20 targeted tests across edge cases, fixture
  presets, calibration comparisons, and transition-aware scenarios.
- This batch keeps the work inside Signal’s reusable analysis crate and makes
  future rhythm tuning more systematic because section-level behavior can now be
  described directly in fixture metadata.
- The fill-bar transition case still carries some tempo ambiguity, but the
  important result is that overall confidence remains higher than ambiguity and
  meter stays stable instead of collapsing outright.

## Next Task

Promote the synthetic fixture system into reusable semi-realistic preset helpers
or saved offline assets that package full scenario families, then compare rhythm
outputs across those preset families to tune confidence and ambiguity thresholds
before Finch depends on them.
