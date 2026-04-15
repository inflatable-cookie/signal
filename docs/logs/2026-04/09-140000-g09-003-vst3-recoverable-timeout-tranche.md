# `g09.003` VST3 recoverable-timeout tranche

## Summary

Added a bounded recoverable interruption boundary on top of the carried and
refreshed broker-backed VST3 lane so the host-facing report now distinguishes
healthy execution, explicit refresh, and recoverable timeout inside one
attached session.

## Code landed

- added an attached-session `timeout-vst3` broker command in
  `~/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
  that reports `execution_interrupted`, `timeout=recoverable`, and
  `resume_hint=refresh_or_stream` without forcing teardown
- exposed that timeout path through the shared broker client in
  `~/Dev/projects/signal/crates/signal-runtime/src/sandbox_broker_support.rs`
- widened the local and server broker-backed VST3 ensure flows so the report
  now captures carried execution, refresh, and recoverable interruption in
  `~/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and
  `~/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
- tightened the public VST3 broker proofs so the exported host report now
  requires the timeout markers in
  `~/Dev/projects/signal/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and
  `~/Dev/projects/signal/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`

## Validation

Passed sequentially:

- `cargo test -p signal-plugin-sandbox broker::tests::broker_reports_recoverable_vst3_timeout_after_refresh_cycle -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_vst3_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `effigy health`

## Notes

- this is still a bounded interruption marker, not full timeout recovery orchestration
- the next step should be a broader `g09.003` closeout audit so the milestone
  does not devolve into proof-only atomicity

## Next

Audit what VST3 depth is still simulated in the broker lane versus genuinely
adapter-backed, then either close `g09.003` or land one final batch on the
biggest remaining fake seam before promoting the milestone.
