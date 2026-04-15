# `g09.003` VST3 automation-delta tranche

## Summary

Threaded a bounded automation/event delta through the carried VST3 state
baseline so the broker-backed host report now proves changed applied events
across runs, not only continuity counters.

## Code landed

- added an `automation_delta` marker to bounded VST3 block execution records in
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/model.rs`
  and
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/session.rs`
- widened the broker-backed VST3 stream so carried-forward runs apply a bounded
  incremented event delta and aggregate that delta into the broker execution
  summary in
  `~/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
- tightened the local and server public VST3 broker proofs so the exported host
  report now requires both the fresh baseline delta and the carried-forward
  delta in
  `~/Dev/projects/signal/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and
  `~/Dev/projects/signal/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`

## Validation

Passed sequentially:

- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_resets_vst3_continuity_after_teardown_and_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `effigy health`

## Notes

- the important change is behavioral distinction across carried state, not wider
  DSP realism
- the next seam is a bounded state-refresh or suspend/resume cycle inside one
  broker-backed VST3 session

## Next

Prove a bounded suspend/resume or state-store refresh boundary inside the
broker-backed VST3 lane so the host report distinguishes carried execution
deltas from an explicit state-refresh cycle, not only from teardown and
reattach.
