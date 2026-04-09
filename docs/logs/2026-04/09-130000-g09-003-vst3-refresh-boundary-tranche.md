# `g09.003` VST3 refresh-boundary tranche

## Summary

Proved a bounded in-session VST3 state-refresh boundary so the broker-backed
host report can distinguish carried execution from an explicit state refresh
without tearing the sandbox down.

## Code landed

- added a `refresh-vst3` broker command and attached-session refresh handling in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
- exposed the refresh path through the shared broker client in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-runtime/src/sandbox_broker_support.rs`
- widened the local and server broker-backed VST3 ensure flows so they now
  drive `stream -> stream -> refresh -> stream` and export that refresh boundary
  in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
- tightened the public local and server VST3 broker proofs so the host-facing
  report now requires `refresh_cycle` and `continuity_reset=refreshed` in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`

## Validation

Passed sequentially:

- `cargo test -p signal-plugin-sandbox broker::tests::broker_refreshes_vst3_state_without_teardown -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `effigy health`

## Notes

- the important result is an explicit in-session refresh boundary, not new DSP
  breadth
- the next real seam is a bounded interruption or timeout on top of this carried
  and refreshed VST3 path

## Next

Add one bounded timeout or fault boundary on top of the carried/refresh VST3
lane so the host report distinguishes healthy carried execution, explicit
refresh, and recoverable execution interruption inside the same broker-backed
proof surface.
