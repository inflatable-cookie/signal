# Rhythm Variant Family Calibration

Date: 2026-03-08
Owner: core-product

## Summary

Extended the rhythm preset layer from one preset per scenario into reusable
family variants for harmonic-rhythm support, fill density, and groove dropout,
then added calibration tests that compare those variants against the actual
behavior of the current meter and ambiguity surface.

## Work completed

- widened the test-only preset surface with variant families for:
  - structured harmony support
  - fill-density escalation
  - groove-dropout escalation
- refactored preset rendering so those families share a single fixture path
  instead of duplicating ad hoc setup in each test
- added new family-level calibration coverage that now checks:
  - active harmonic support can surface four-four meter while sparser harmonic
    support stays meter-unknown without a large confidence collapse
  - denser fill variants raise tempo ambiguity while preserving four-four meter
  - lighter and medium dropout variants remain meter-unknown and show declining
    confidence before the heavy-dropout fallback case
- updated the preset expectation table so it encodes the measured current
  behavior instead of forcing unsupported meter assumptions onto sparse-harmony
  and partial-dropout cases

## Validation

- `cargo test -p signal-analysis-rhythm` in a temporary four-crate Rust
  workspace copy rooted at `/tmp/signal-rhythm-validate-copy-152021`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The root Cargo workspace is currently blocked by unrelated workspace members
  with no Rust target files. The first visible blocker from the repo root is
  `signal-ipc`, and after temporarily unblocking that path another missing
  target in `signal-plugin-clap` surfaced. I did not widen this batch into
  fixing those unrelated crates.
- The new calibration layer now makes a clearer distinction between:
  - scenarios that should force a stable bar interpretation
  - scenarios that should stay intentionally meter-unknown but still carry
    usable tempo confidence
- The rhythm crate now has 25 targeted tests in the calibrated temp Rust
  workspace path used for validation.

## Next Task

Add explicit bar-structure transition variants such as pickup bars, late
downbeat shifts, and bar-length disruptions to the preset families, then tune
how `meter: None`, confidence, and tempo ambiguity evolve across those
transitions before Finch starts depending on threshold behavior.
