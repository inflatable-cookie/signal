# 2026-03-09 00:30:00 UTC: Payload-Only Debug Section Freeze

Status: completed
Owner: core-product

## Summary

Replaced the loose payload-debug boolean path in `signal-supervisor-tools` with
an explicit debug-section model and froze `payload` as the only supported
opt-in debug section for now.

## Changes

- introduced an explicit host-summary debug section model in
  `crates/signal-supervisor-tools`
- routed payload inclusion through that model instead of a loose boolean-only
  rendering path
- added tests that pin `payload` as the only currently supported debug section
- updated the supervisor export contract and roadmap notes to require an
  explicit batch before any second debug section is added

## Validation

- `cargo fmt --all`
- `cargo check -p signal-supervisor-tools`
- `cargo test -p signal-supervisor-tools --no-run`
- `git diff --check`

## Notes

- this keeps the current export policy narrow in code, not just in docs
- runtime execution is still not used as the validation gate in this
  environment because fresh Rust binaries can intermittently stall after launch

## Next Task

Decide whether there is a concrete inspection need strong enough to justify a
second explicit debug section beyond payload; until then, keep the current
payload-only targeted model frozen.
