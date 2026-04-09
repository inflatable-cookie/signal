# 2026-04-09 - g09.004 macOS acceptance boundary and promotion

## Summary

Closed `g09.004` by turning the AU-plus-CoreAudio proof lane into a repo-owned
acceptance boundary and promoting the milestone from `active` to `complete`.

## Delivered

- added `effigy acceptance:macos-au-coreaudio-boundary` in
  `/Users/betterthanclay/Dev/projects/signal/effigy.toml`
- added the supervisor-tools descriptor family at
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-supervisor-tools/src/descriptor_families/macos_au_coreaudio.rs`
- wired the new boundary through supervisor schema constants, describe flags,
  describe dispatch, and boundary assertion tests
- recorded the promotion and explicit demo-lane handoff in
  `/Users/betterthanclay/Dev/projects/signal/docs/roadmaps/g09/004-real-au-discovery-coreaudio-backed-execution-and-macos-proof.md`
  and `/Users/betterthanclay/Dev/projects/signal/docs/roadmaps/g09/README.md`

## Validation

- `cargo check -p signal-supervisor-tools`
- `cargo run -p signal-supervisor-tools -- --describe-macos-au-coreaudio-boundary --format=json`
- `effigy acceptance:macos-au-coreaudio-boundary`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Notes

- the new acceptance lane passed end to end
- pre-existing warnings remain non-blocking:
  - unused imports in `/Users/betterthanclay/Dev/projects/signal/crates/signal-runtime/src/tests.rs`
  - dead-code warnings in local public broker test support under
    `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/tests/support/public_host_edge_sandbox_broker.rs`
- `g09.004` is treated as complete because the remaining AU omissions are
  deliberate scope and the interactive operator path is already owned by the
  dedicated demo milestones `g09.011` and `g09.012`

## Next Task

Start `g09.005` with one meaningful Linux plugin-realization batch: audit the
remaining LV2 scaffold seams in discovery, extension negotiation, and host
proof roots, then land the first production-depth pass on real LV2 bundle and
extension identity before widening worker or live execution behavior.
