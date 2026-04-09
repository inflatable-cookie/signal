# 2026-04-09 - g09.004 AU sandbox lifecycle proof tranche

## Summary

Moved the AU lane off the generic demo broker path and onto a real bounded AU
state-store, activation, and teardown contract that is now visible through both
hosts' public AU proof surfaces.

## Work Completed

- updated `/crates/signal-plugin-au/src/au_host_adapter/model.rs`
  - added bounded `AuStateSnapshot`, `AuActivationRecord`, and
    `AuTeardownRecord` lifecycle DTOs
- updated `/crates/signal-plugin-au/src/au_host_adapter/session.rs`
  - added metadata-driven `store_state_snapshot(...)`,
    `activate_instance(...)`, and `teardown_instance(...)`
- updated `/crates/signal-plugin-au/src/lib.rs`
  - added adapter coverage for the new bounded AU lifecycle records over a real
    temp `.component` bundle
- updated `/crates/signal-runtime/src/sandbox_broker_support.rs`
  - added explicit `SandboxBrokerFlavor::Au` attach and teardown command
    routing
- updated `/crates/signal-plugin-sandbox/Cargo.toml`
  - added the `signal-plugin-au` dependency so the sandbox broker can realize
    AU lifecycle truth directly
- updated `/crates/signal-plugin-sandbox/src/broker.rs`
  - added explicit `attach-au`, `run-au`, and `teardown-au` broker commands
  - added AU bundle env-backed discovery and lifecycle realization inside the
    sandbox broker
  - added focused broker coverage for AU-flavored lifecycle receipts
- updated `/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  - routed the AU broker-enabled path through real AU state-store and
    activation detail instead of `SandboxBrokerFlavor::Demo`
  - enriched the non-broker AU path with AU lifecycle summary detail
- updated `/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
  - applied the same AU lifecycle wiring to the server host path
- updated `/crates/signal-host-local/tests/public_host_edge_au.rs`
  - enabled the broker-backed AU bring-up path and asserted AU lifecycle detail
    in the exported supervisor report
- updated `/crates/signal-host-server/tests/public_host_edge_au.rs`
  - applied the same broker-backed AU lifecycle assertions on the server host
- updated `/docs/roadmaps/g09/004-real-au-discovery-coreaudio-backed-execution-and-macos-proof.md`
  - recorded `Batch 4.2 Tranche 2 Outcome`
  - checked the hardened AU sandbox instantiation item complete

## Validation

- `cargo test -p signal-plugin-au --lib`
- `cargo check -p signal-plugin-sandbox`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_au_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_au -- --exact local_shared_host_edge_exports_runtime_au_baseline_truth --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_au -- --exact server_shared_host_edge_exports_runtime_au_baseline_truth --nocapture --test-threads=1`
- `effigy health`

## Outcome

`g09.004` now has an honest AU sandbox bring-up lane. AU discovery remains
bundle-local and production-backed, CoreAudio device truth is already real, and
the host-facing AU proof surfaces now export bounded AU lifecycle detail instead
of only generic sandbox attachment. The remaining large seam in this milestone
is not more lifecycle plumbing; it is explicit AU failure mapping plus a tighter
combined AU-plus-CoreAudio proof that exports both device and AU lifecycle or
fault truth from the same stable host surface.
