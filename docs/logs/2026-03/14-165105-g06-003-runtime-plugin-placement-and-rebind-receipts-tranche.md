# 2026-03-14 16:51:05 g06.003 Runtime Plugin Placement And Rebind Receipts Tranche

## Summary

- completed Batch 3.2 of `g06.003`
- added runtime-owned plugin placement policy and assignment meaning
- widened shared-sandbox lifecycle and chain receipts with typed continuity and
  rebind fields
- exported plugin lifecycle placement or rebind meaning through supervisor JSON

## Delivered

- `signal-runtime` now owns `RuntimePluginPlacementPolicy`,
  `RuntimePluginPlacementRule`, `RuntimePluginPlacementRuleMatcher`, and
  `RuntimePluginIsolationOutcome`
- `RuntimePluginSandboxSnapshot` and `RuntimePluginChainStageSnapshot` now
  carry placement outcome, matched rule, grouping key, shared-boundary member
  count, continuity class, and rebindability
- `RuntimePluginLifecycleSnapshot` and `RuntimePluginChainSnapshot` now carry
  shared versus isolated stage or sandbox counts plus rebindable or terminal
  totals
- `RuntimeObservationReport` or `RuntimeSupervisorReport` JSON now exports
  `plugin_lifecycle_snapshot` in addition to the widened chain snapshot

## Focused proof

- runtime-owned placement policy drives shared versus isolated assignment
  receipts without host-local reconstruction
- shared sandbox detach and quarantine state exports restartable versus
  terminal continuity directly across all affected member stages
- existing lifecycle recovery and quarantine proof still passes on the widened
  receipt family

## Validation

- `cargo test -p signal-runtime runtime_plugin_placement_policy_drives_shared_and_isolated_assignment_receipts -- --nocapture`
- `cargo test -p signal-runtime runtime_shared_sandbox_rebind_receipts_track_restartable_and_terminal_boundaries -- --nocapture`
- `cargo test -p signal-runtime runtime_tracks_plugin_lifecycle_recovery_and_quarantine_state -- --nocapture`
- `cargo test -p signal-runtime --lib --no-run`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- multi-instance proof breadth for allowlist, denylist, and by-format cases is
  still Batch 3.3 work
- explicit shared-boundary blast-radius export is still deferred beyond the
  current receipt shape
- in-process placement remains a runtime-owned outcome in the shared policy
  surface, but the current exercised proof path is still sandbox-first

## Next Task

Continue `g06.003` with Batch 3.3 by proving the widened placement, grouping,
and shared-boundary continuity receipts across multi-instance degradation,
recovery, and policy cases without host-local rule reconstruction.
