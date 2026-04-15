# `g09.003` VST3 state-mutation validation closeout tranche

## Summary

Validated and recorded the bounded VST3 state-mutation execution tranche after
rebooting and rerunning the focused adapter, broker, and host-edge proof lanes
sequentially.

## Code confirmed

- VST3 block execution now emits `parameter_signature`, `state_transition`, and
  `next_state_digest` in
  `~/Dev/projects/signal/crates/signal-plugin-vst3/src/vst3_host_adapter/session.rs`
- the broker-owned VST3 execution stream carries those mutation markers through
  the short execution run in
  `~/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
- broker-backed VST3 host reports now surface the mutation summary through the
  attached transport detail in
  `~/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and
  `~/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`

## Validation

Passed sequentially after reboot:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 tests::vst3_session_plan_preserves_controller_pairing_and_transport -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_vst3_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Notes

- the earlier validation stalls were environmental rather than code-level; the
  focused sequential rerun after reboot completed cleanly
- this tranche still claims bounded deterministic state mutation, not full DSP
  or rich automation semantics

## Next

Thread one bounded parameter-automation or event-packet application record
through the VST3 broker run so host-facing reports prove ordered per-block
application, not only final mutation summaries.
