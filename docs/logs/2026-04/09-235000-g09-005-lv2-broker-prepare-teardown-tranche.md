# 2026-04-09 - g09.005 - LV2 broker prepare and teardown tranche

## Summary

Batch 5.2 Tranche 3 replaced the remaining demo-backed LV2 broker path with a
bounded real LV2 prepare and teardown lane.

## What changed

- added an adapter-owned LV2 teardown record in `signal-plugin-lv2` so the LV2
  lifecycle surface now has an explicit bounded cleanup summary alongside the
  existing prepare-negotiation record
- added `SandboxBrokerFlavor::Lv2` plus real `attach-lv2`, `run-lv2`, and
  `teardown-lv2` handling in `signal-plugin-sandbox`, driven by real LV2 bundle
  discovery, instantiation, prepare, and teardown through `signal-plugin-lv2`
- threaded real LV2 broker spawn env and LV2 broker flavor selection through
  the server-host sandbox path instead of falling back to `Demo`
- strengthened the broker-backed server-host LV2 public proof so it now asserts
  exported LV2 prepared-negotiation truth and LV2-specific broker detail rather
  than only a generic broker attach marker

## Validation

- `cargo check -p signal-plugin-lv2`
- `cargo check -p signal-plugin-sandbox`
- `cargo check -p signal-runtime`
- `cargo check -p signal-host-server`
- `cargo test -p signal-plugin-lv2 tests::lv2_teardown_record_preserves_prepared_negotiation_summary -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_emits_lv2_flavored_receipts -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_lv2_sandbox_through_broker_process -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`

## Next Task

Continue `g09.005` with one meaningful broker-execution batch: replace the
remaining zero-block LV2 broker run path with a bounded real LV2 execution
record, then carry that execution truth through runtime-owned receipts and the
stable server-host LV2 and LV2-extension public lanes before widening into
streaming or recovery-specific LV2 execution behavior.
