# 2026-03-09 00:15:00 UTC: Targeted Host Summary Section Declaration

Status: completed
Owner: core-product

## Summary

Kept the supervisor export on a targeted-flag model rather than introducing
broader debug profiles, and made the export self-describing by declaring the
present `host_summary` sections in both text and JSON output.

## Changes

- added stable default section metadata for `execution`, `transport`, and
  `faults` in `crates/signal-supervisor-tools`
- expanded the section list to include `payload` only when
  `--include-payload` is set
- added renderer tests for default and payload-augmented section lists
- updated the README and export contract to make the targeted-flag policy
  explicit

## Validation

- `cargo fmt --all`
- `cargo check -p signal-supervisor-tools`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- this keeps schema version 1 additive and self-describing without broadening
  the default export surface
- runtime execution is still not used as the validation gate in this
  environment because fresh Rust binaries can intermittently stall after launch

## Next Task

Decide whether the current targeted-flag model needs one more explicit debug
section beyond payload, or whether the supervisor export should stay at
payload-only opt-in detail until a concrete inspection need appears.
