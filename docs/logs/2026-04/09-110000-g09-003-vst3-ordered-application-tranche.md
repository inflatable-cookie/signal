# `g09.003` VST3 ordered application tranche

## Summary

Extended the bounded VST3 execution stream so the broker-backed host report now
proves ordered per-block application, not only final mutation summaries.

## Code landed

- added `parameter_application_order` and `event_packet_order` to bounded VST3
  block execution records in
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/model.rs`
  and
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/session.rs`
- aggregated those per-block records into broker-visible `application_order`
  and `packet_order` summaries in
  `~/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
- tightened the broker-backed VST3 public host-edge assertions so both hosts
  now require the ordered application history in
  `~/Dev/projects/signal/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and
  `~/Dev/projects/signal/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`

## Validation

Passed sequentially:

- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_vst3_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Notes

- this tranche still uses deterministic synthetic ordering, not a full
  realtime automation engine
- the important change is that the ordered application history now survives the
  broker boundary and is visible at the host-report layer

## Next

Thread one bounded continuity behavior across multiple VST3 broker runs so the
report proves carry-over state or parameter baseline, not only ordered
application inside a single isolated run.
