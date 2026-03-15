# g06.008 - Runtime Deferred-Work Policy Receipts Tranche

Date: 2026-03-14
Milestone: `g06.008`
Batch: `8.2`
Status: complete

## Summary

Deepened the deferred-work scheduler-policy lane from contract-only meaning
into real runtime-owned receipts. Deferred-service orchestration now exposes
typed priority-band, blocking-priority, backpressure, starvation, and
cancellation fields, and the same policy evidence now rolls into bounded
performance snapshots and trace receipts.

## What changed

- widened `RuntimeDeferredServiceReceipt` with typed scheduler-policy fields:
  - priority band
  - blocking priority band
  - backpressure source
  - starvation flag and starved work-item count
  - cancellation cause and cancelled work-item count
- moved queue, purge, and invalid-request receipt derivation onto one shared
  runtime helper instead of leaving policy assembly at individual call sites
- widened `RuntimePerformanceSnapshot` to preserve the latest deferred-work
  policy state alongside existing timing and hotspot context
- widened `RuntimePerformanceTraceReceipt` to preserve bounded starvation,
  cancellation, and backpressure evidence across an observation window
- added focused runtime proofs for:
  - run, throttle, defer, and abort deferred-work outcomes
  - performance snapshot export of the widened policy surface
  - performance trace export of bounded starvation and backpressure evidence

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime --lib --no-run`
- `cargo test -p signal-runtime runtime_offline_render_queue_ -- --nocapture`
- `cargo test -p signal-runtime runtime_purge_defers_in_safe_mode_and_observation_export_surfaces_last_decision -- --nocapture`
- `cargo test -p signal-runtime runtime_offline_render_invalid_request_abort_surfaces_typed_cancellation_policy -- --nocapture`
- `cargo test -p signal-runtime runtime_performance_snapshot_captures_scheduler_pressure_and_background_policy -- --nocapture`
- `cargo test -p signal-runtime runtime_performance_trace_receipt_summarizes_playback_recording_and_deferred_work_window -- --nocapture`

## Deferred

- public runtime, supervisor, and stable host-edge proof for the widened
  deferred-work scheduler-policy receipt family
- repo-owned acceptance or descriptor seam for that consumer boundary
- any broader distributed or remote deferred-work scheduler ownership

## Next

Continue `g06.008` with Batch 8.3 by proving the widened deferred-work
priority, backpressure, starvation, and cancellation receipts remain
consumable through shared runtime, supervisor, and stable host-edge surfaces
without private queue-state helpers or host-local policy forks.
