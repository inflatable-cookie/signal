# `g09.003` VST3 reset-boundary tranche

## Summary

Proved the reset boundary for bounded broker-backed VST3 continuity so the
lane now distinguishes carried-forward attached execution from a fresh
reattach after teardown.

## Code landed

- added a broker-level reattach proof that drives
  `attach -> stream -> stream -> teardown -> attach -> stream -> stream -> teardown`
  and requires both fresh and carried-forward continuity markers in
  `~/Dev/projects/signal/crates/signal-plugin-sandbox/src/broker.rs`
- added local and server public host-edge proofs that reattach the same VST3
  sandbox id after teardown and require both fresh and carried-forward
  continuity markers in
  `~/Dev/projects/signal/crates/signal-host-local/tests/public_host_edge_sandbox_broker.rs`
  and
  `~/Dev/projects/signal/crates/signal-host-server/tests/public_host_edge_sandbox_broker.rs`

## Validation

Passed sequentially:

- `cargo test -p signal-plugin-sandbox broker::tests::broker_resets_vst3_continuity_after_teardown_and_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_resets_vst3_continuity_after_broker_reattach -- --exact --nocapture --test-threads=1`
- `effigy health`

## Notes

- the important result is explicit reset semantics, not new VST3 DSP depth
- the next real gap is carrying a bounded automation or event delta across the
  persisted baseline so continuity proves changed behavior, not only counters

## Next

Thread one bounded parameter-automation or event-application delta across the
carried VST3 state baseline itself, then prove the broker-backed host report
can distinguish a fresh reattach baseline, a carried-forward baseline, and a
new automation/event delta.
