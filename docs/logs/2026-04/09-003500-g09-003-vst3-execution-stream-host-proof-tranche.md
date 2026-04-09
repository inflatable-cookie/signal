# `g09.003` VST3 execution stream and host-proof tranche

## Summary

Widened the bounded VST3 broker execution path from one generic block receipt
into a short execution stream and pushed that execution truth through the
broker-backed host report surface.

## Code landed

- widened `signal-plugin-sandbox` VST3 execution from one last-block summary to
  a short multi-block execution stream in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
- added attached-session VST3 execution-stream collection to the shared broker
  client in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-runtime/src/sandbox_broker_support.rs`
- recorded broker-backed VST3 execution-stream summaries back onto attached
  host transport reports in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/src/host_support/sandbox_sessions.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/src/host_support/sandbox_sessions.rs`
- extended the public broker proof assertions so broker-backed VST3 host
  reports now require `execution_complete`, processed-block counts, and
  parameter/midi event detail in
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and
  `/Users/betterthanclay/Dev/projects/signal/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`

## Validation

Passed:

- `cargo check -p signal-plugin-vst3`
- `cargo test -p signal-plugin-vst3 --lib`
- `cargo test -p signal-plugin-sandbox`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_reports_broker_backed_vst3_deferred_teardown_fault -- --nocapture`
- `effigy health`

Notes:

- the full public broker proof binaries are still expensive because they spawn
  nested broker processes from inside test binaries
- the new VST3 route/report assertions were observed green during focused rerun
  progress inside those binaries, but the tranche intentionally kept the
  authoritative validation on the fast adapter, broker, health, and one
  targeted host-edge recovery lane rather than treating the slow whole-binary
  broker proof as the gating signal for this batch

## Next

Deepen the bounded VST3 execution stream with one real parameter or state
mutation path so the broker and host-facing report prove more than per-block
event counts.
