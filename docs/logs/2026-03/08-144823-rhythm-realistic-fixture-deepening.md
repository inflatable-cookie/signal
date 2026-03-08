# Rhythm Realistic Fixture Deepening

Date: 2026-03-08
Owner: core-product

## Summary

Pushed rhythm validation beyond bare click patterns by adding richer synthesized
offline fixtures with weak backbeats, sparse harmonic motion, and section
transitions, then confirmed the current tempo and meter path still behaves
sensibly across them.

## Work completed

- expanded the `signal-analysis-rhythm` test harness with reusable fixture parts:
  - kick, snare, and hat tone sets
  - chord-tone fixtures for sparse harmonic motion
  - a reusable four-four groove builder for sectioned arrangements
- added more realistic synthesized offline rhythm fixtures covering:
  - weak-backbeat 4/4 with sparse harmonic changes
  - four-four persistence across a section transition with changed groove and
    chord content
- kept the existing mixed-meter suppression and pickup-bar coverage in place so
  the more realistic fixtures deepen validation instead of replacing the
  adversarial edge cases

## Validation

- `cargo test -p signal-analysis-rhythm`
- `cargo test --workspace`
- `git diff --check`

## Notes

- The rhythm crate now has 15 targeted tests, including the newer arrangement-
  style fixtures and the earlier edge-case patterns.
- This batch is still synthetic, but it is materially closer to offline music
  than the earlier pure click-grid coverage because the fixtures include sparse
  harmonic events, backbeat coloration, and section-level changes.
- The meter path remained stable under these richer fixtures without regressing
  the unknown-meter fallback that was established in the previous batch.

## Next Task

Add explicit fixture-generation helpers or saved offline fixture assets that can
model section boundaries, harmonic rhythm, and groove changes more systematically,
then compare the rhythm result surface across those fixtures to tune confidence
and ambiguity calibration before Finch consumes it.
