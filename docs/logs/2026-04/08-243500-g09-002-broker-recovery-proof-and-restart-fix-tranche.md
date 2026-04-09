# 2026-04-08 - g09.002 broker recovery proof and restart fix tranche

## Summary

Broadened Batch 2.3 from steady-state broker proof into real broker-backed
recovery proof, and fixed the restart gap that the new proof exposed.

## Work Completed

- extended `/crates/signal-host-local/src/host_support/demo.rs`
  - local demo assemblies can now opt into a test-only broker-backed real
    plugin format, plugin type id, and scan root through explicit env overrides
- updated `/crates/signal-host-local/src/host_support/boot_setup.rs`
  - boot recovery now scans the demo assembly's requested roots and formats
    instead of a fixed CLAP-only scan
- extended `/crates/signal-host-server/src/host_support/demo.rs`
  - server demo assemblies can now opt into the same test-only broker-backed
    plugin override surface
- updated `/crates/signal-host-server/src/host_support/boot_entrypoints.rs`
  - server boot recovery now scans the demo assembly's requested roots and
    formats instead of a fixed CLAP-only scan
- updated `/crates/signal-host-local/src/host.rs`
  - local host now persists active sandbox specs for restart
- updated `/crates/signal-host-local/src/host_api.rs`
  - `restart_plugin_sandbox(...)` now re-establishes broker-backed AU and VST3
    sessions from the persisted sandbox spec and cached discoveries
- updated `/crates/signal-host-server/src/host.rs`
  - server host now persists active sandbox specs and `restart_plugin_sandbox(...)`
    now re-establishes broker-backed AU, LV2, and VST3 sessions from the
    persisted sandbox spec and cached discoveries
- updated `/crates/signal-host-local/tests/support/public_host_edge_sandbox_broker.rs`
  - added a broker demo-plugin env guard helper
  - direct broker process tests now clear and restore demo-plugin override env
    to avoid cross-test contamination
- updated `/crates/signal-host-server/tests/support/public_host_edge_sandbox_broker.rs`
  - added a matching broker demo-plugin env guard helper
- updated `/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  - added a broker-backed VST3 crash-recovery public proof
- updated `/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`
  - added a broker-backed LV2 crash-recovery public proof
- updated `/docs/roadmaps/g09/002-shared-plugin-hosting-substrate-and-hardened-sandbox-execution.md`
  - added Batch 2.3 Tranche 16 outcome and refreshed the next task
- updated `/docs/roadmaps/g09/README.md`
  - refreshed the generation next task toward the next broker-backed recovery
    proof lane

## Validation

- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker`
- `effigy health`

## Outcome

Batch 2.3 now has repo-owned proof that the shared broker process survives and
supports recovery behavior, not only steady-state ensure and teardown. The new
proof also flushed out a real production bug: both hosts had been recording
`SandboxRestarted` without actually re-establishing broker-backed sessions.
That is now fixed for the formats already on the broker path.

The next useful tranche should broaden recovery proof one step further, most
likely overlap contention or deferred old-transport teardown through the shared
broker path, so the queue closes on verified recovery behavior instead of only
verified steady-state ownership and one successful restart lane.
