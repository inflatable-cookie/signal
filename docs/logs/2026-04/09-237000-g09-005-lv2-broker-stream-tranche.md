# 2026-04-09 - g09.005 - LV2 broker stream tranche

## Summary

Batch 5.3 Tranche 2 widened the bounded LV2 broker execution lane from one
record into a short multi-block stream.

## What changed

- replaced the single attached-session LV2 execution receipt in
  `signal-plugin-sandbox` with a short three-block LV2 stream and a broker-side
  aggregate execution summary
- updated the shared broker client to collect LV2 running receipts until the
  broker returns to `Attached`, mirroring the existing VST3 attached-stream
  pattern without widening into LV2 refresh or timeout behavior yet
- threaded the LV2 stream summary through the server-host broker-backed LV2
  path so runtime-owned transport detail now exports processed block count,
  stream order, and last-block completion truth
- strengthened the stable public broker-backed LV2 proof so it now asserts the
  ordered LV2 stream instead of a single execution receipt

## Validation

- `cargo check -p signal-plugin-sandbox`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-server`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_lv2_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_lv2_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`

## Next Task

Continue `g09.005` with one meaningful closeout-oriented batch: thread one
recovery-owned LV2 execution lane through the broker-backed path, most likely a
deferred-teardown or crash-recovery proof that preserves the new LV2 stream
truth, then decide whether the milestone is ready for Linux acceptance smoke
and promotion.
