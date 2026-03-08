# 2026-03-09 00:45:00 UTC: Debug Section Policy Metadata

Status: completed
Owner: core-product

## Summary

Made the current payload-only debug policy machine-readable in
`signal-supervisor-tools` by exporting supported and enabled debug-section
metadata alongside the existing host-summary section list.

## Changes

- added `debug_sections_supported` and `debug_sections_enabled` metadata to the
  text and JSON host-summary renderers
- kept `payload` as the only currently supported opt-in debug section
- added renderer assertions for default and payload-enabled outputs
- updated the Signal export contract and README so the policy is frozen in both
  code and docs

## Validation

- `cargo fmt --all`
- `cargo check -p signal-supervisor-tools`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- this keeps the export boundary additive and self-describing without adding a
  second debug section
- runtime execution is still not used as the validation gate in this
  environment because fresh Rust binaries can intermittently stall after launch

## Next Task

Decide whether the payload-only debug policy is now sufficiently frozen to
leave this export boundary alone for a while, or whether there is a concrete
inspection need strong enough to justify a second explicit debug section.
