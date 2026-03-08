# Rhythm Section Metadata Fixtures

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm fixture builder with explicit section metadata for dropout
bars, fill-bar beat overrides, and per-bar harmonic plans, then added
transition-oriented comparison tests so confidence, ambiguity, and unknown-meter
behavior can be checked across structured section changes instead of only within
steady loops.

## Work completed

- widened the test-only `GrooveSection` fixture model with:
  - per-bar beat-pattern overrides
  - per-bar chord plans
  - explicit dropout-bar metadata
- updated the four-four groove builder so the same fixture surface can model:
  - steady sections
  - milder groove dropout sections
  - heavier dropout bars that should suppress meter
  - fill bars with different beat emphasis
  - section-local harmonic rhythm changes
- added transition-oriented rhythm tests covering:
  - confidence drop from steady sections to mild-dropout sections
  - 4/4 preservation through a fill bar plus harmonic-rhythm changes
  - explicit unknown-meter behavior on dropout-heavy transition fixtures

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `git diff --check`

## Notes

- The rhythm crate now carries 20 targeted tests, combining absolute checks,
  cross-scenario calibration comparisons, and transition-aware fixture cases.
- This batch keeps the rhythm work inside Signal and makes the next rounds of
  calibration materially easier because section-level behavior can now be
  described declaratively in the fixture system.
- The current result surface still holds up under dropout-heavy and fill-bar
  scenarios without regressing the earlier unknown-meter guardrails.

## Next Task

Promote the fixture system from purely synthetic section metadata toward
semi-realistic rendered offline assets or reusable fixture presets that combine
fills, groove dropout, harmonic-rhythm changes, and section boundaries, then use
those presets to tune the rhythm result surface before Finch starts depending on
specific confidence and ambiguity thresholds.
