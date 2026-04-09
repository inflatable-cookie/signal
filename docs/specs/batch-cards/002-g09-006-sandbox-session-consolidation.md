# 002 - g09.006 Sandbox Session Consolidation

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.006
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md, docs/contracts/015-offline-render-recovery-and-resumability-contract.md, docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md, docs/contracts/074-shared-host-runtime-execution-and-recovery-unification-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/006-shared-host-runtime-execution-and-recovery-unification.md
Auto-start next card: no

## Objective

Continue `g09.006` with one broad consolidation batch on the next
highest-leverage seam: `sandbox_sessions.rs`.

## Scope

- consolidate the shared broker session and transport orchestration shell now
  duplicated across local and server hosts
- keep format-specific preparation and host-edge environment differences
  explicit at the edges
- rerun focused local/server broker recovery proofs

## Steps

1. Consolidate the common attach, record, and teardown shell now concentrated
   in `sandbox_sessions.rs`.
2. Keep format-specific preparation and environment-specific host edges outside
   the shared shell.
3. Rerun the focused local/server broker recovery proofs needed to prove the
   shared seam still behaves correctly.

## Acceptance Criteria

- equivalent broker session and transport orchestration now goes through a
  shared shell where the semantics are identical
- local and server host differences remain explicit only at the true edge
- focused broker recovery proofs pass for both hosts

## Evidence Required

- batch log for the next `g09.006` tranche
- validation actually run

## Outcome

The common broker session shell is now runtime-owned instead of living in two
parallel host copies. The shared attach, prepared-state recording, attached
execution summary recording, VST3 broker execution sequence, and teardown flow
now live in `signal-runtime`'s broker support layer, while the host files keep
only the true edge responsibilities:

- format-specific preparation and failure mapping
- host-specific environment assembly and instance-id prefixes
- server-only LV2 negotiation and execution depth

This keeps the consolidation inside the lane boundary. It removes the broad
local/server duplication in `sandbox_sessions.rs` without flattening the
remaining adapter- and host-specific behavior into a false common layer.

## Validation Run

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`

## Stop Conditions

- the extraction starts flattening genuinely different host-edge behavior
- the batch grows into a full runtime execution rewrite

## Next Task

Continue the active strict lane from
`docs/specs/batch-cards/003-g09-006-au-vst3-preparation-fault-shell.md`.
