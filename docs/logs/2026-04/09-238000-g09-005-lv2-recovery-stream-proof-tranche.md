# 2026-04-09 - g09.005 - LV2 recovery stream proof tranche

## Summary

Batch 5.3 Tranche 3 carried the new LV2 broker-backed stream truth through the
first recovery-owned public proof lanes.

## What changed

- widened the stable server-host public broker-backed LV2 crash recovery proof
  to require exported LV2 execution markers instead of only generic broker
  attach truth
- did the same for the deferred-teardown fault and cleanup-retry recovery
  proofs, so the broker-backed LV2 stream contract now survives both faulted
  and recovered recovery-owned paths
- kept the batch narrowly focused on proof carry-through rather than adding
  more broker or adapter behavior, which makes the next seam a milestone
  closeout question instead of more proof churn

## Validation

- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_reports_broker_backed_lv2_deferred_teardown_fault -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_recovers_after_broker_backed_lv2_cleanup_retry -- --exact --nocapture --test-threads=1`

## Next Task

Continue `g09.005` with one meaningful closeout batch: add a focused Linux LV2
acceptance or smoke lane that exercises real discovery plus the broker-backed
execution path, then reassess the remaining deferred gaps and decide whether the
milestone is ready for promotion.
