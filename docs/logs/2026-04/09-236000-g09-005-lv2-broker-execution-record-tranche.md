# 2026-04-09 - g09.005 - LV2 broker execution record tranche

## Summary

Batch 5.3 Tranche 1 replaced the zero-block LV2 broker placeholder with one
bounded real LV2 execution record.

## What changed

- added an adapter-owned `Lv2BlockProcessingRecord` in `signal-plugin-lv2` plus
  a bounded `execute_block(...)` path so LV2 execution is no longer implied only
  by prepare and teardown summaries
- added an attached-session `execute-lv2` command in `signal-plugin-sandbox`
  that executes one real LV2 adapter-owned block record and returns it through
  the broker instead of only reporting zero processed blocks
- extended the shared broker client and server-host LV2 sandbox path so the
  bounded LV2 execution summary is recorded back into runtime-owned transport
  detail after broker attach
- strengthened the stable server broker-backed LV2 public proof so it now
  asserts exported LV2 execution truth including processed blocks, block frames,
  patch messages, MIDI events, and completion state

## Validation

- `cargo check -p signal-plugin-lv2`
- `cargo check -p signal-plugin-sandbox`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-server`
- `cargo test -p signal-plugin-lv2 tests::lv2_execute_block_reports_bounded_execution_truth -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_lv2_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_lv2_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`

## Next Task

Continue `g09.005` with one meaningful execution-depth batch: widen the bounded
LV2 broker execution record into either a short multi-block execution stream or
one recovery-owned LV2 execution lane, then carry that deeper execution truth
through runtime-owned receipts and the stable server-host LV2 public surfaces
before adding Linux acceptance smoke.
