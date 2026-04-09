# `g09.003` VST3 continuity tranche

## Summary

Added bounded continuity across multiple broker-backed VST3 execution runs so
the host-facing report now proves carried-forward state within one attached
session instead of treating every execution stream as isolated.

## Code landed

- persisted the attached VST3 broker execution state across repeated
  `stream-vst3` requests and exported continuity markers such as
  `execution_runs`, `continuity`, and `continued_from` in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
- updated the broker-backed VST3 ensure paths so both hosts drive two bounded
  execution streams before recording prepared transport truth in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
- tightened the public VST3 broker assertions so both hosts now require the
  continuity markers in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`

## Validation

Passed sequentially:

- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_vst3_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Notes

- this continuity is intentionally bounded to repeated execution streams within
  one attached broker session
- the next real gap is proving the reset boundary after teardown, so carried
  state and fresh reattach semantics are both explicit

## Next

Prove the VST3 continuity reset boundary by distinguishing carried-forward
attached execution from a fresh reattach after teardown, then surface that
reset truth through the broker-backed host report.
