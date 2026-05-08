# 003 - g09.006 AU/VST3 Preparation And Fault Shell

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.006
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md, docs/contracts/015-offline-render-recovery-and-resumability-contract.md, docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md, docs/contracts/074-shared-host-runtime-execution-and-recovery-unification-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/006-shared-host-runtime-execution-and-recovery-unification.md
Auto-start next card: no

## Objective

Continue `g09.006` with one narrower but still meaningful shared-support seam:
the duplicated AU and VST3 preparation and fault-recording shell still living
in `sandbox_sessions.rs`.

## Scope

- consolidate the shared AU and VST3 broker-preparation shell where the local
  and server semantics are still identical
- consolidate the shared AU fault-recording shell where the local and server
  semantics are still identical
- keep local/server environment assembly, instance-id prefixes, and
  server-only LV2 behavior explicit at the edge
- rerun focused local/server broker recovery proofs after the extraction

## Steps

1. Extract the duplicated AU broker-preparation shell out of both host
   `sandbox_sessions.rs` files into the shared runtime-owned support layer.
2. Extract the duplicated VST3 broker-preparation shell out of both host
   `sandbox_sessions.rs` files into the same support layer without flattening
   host env assembly or instance-id differences.
3. Extract the duplicated AU fault-recording shell only if the remaining local
   and server behavior is still semantically identical after the first two
   moves.
4. Rerun the focused local/server broker recovery proofs needed to confirm the
   shared seam remains correct.

## Acceptance Criteria

- equivalent AU and VST3 preparation now goes through one shared shell where
  semantics are identical
- AU fault-recording only moves if the local/server behavior remains truly
  identical
- local/server environment assembly and server-only LV2 behavior remain
  explicit at the true edge
- focused broker recovery proofs still pass for both hosts

## Evidence Required

- batch log for the next `g09.006` tranche
- validation actually run

## Outcome

The duplicated AU and VST3 preparation shell is no longer split across the two
host copies. The runtime-owned broker support layer now owns the shared
prepared-session flow, while the hosts keep only the true edge inputs:

- local versus server environment assembly and instance-id prefixes
- server-only LV2 preparation, negotiation, and execution depth
- the remaining format-specific failure behavior that is not actually shared

The identical AU protocol-violation prepare-fault recording shell moved into
the same runtime-owned layer as well. That leaves `sandbox_sessions.rs` with
smaller, edge-specific responsibilities rather than another broad shared seam.

## Validation Run

- `cargo check -p signal-runtime`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`

## Stop Conditions

- the extraction starts forcing LV2 behavior or server-only negotiation through
  a fake shared abstraction
- the batch starts folding host-specific environment assembly into the shared
  layer
- the batch grows into a full lifecycle or runtime rewrite instead of staying
  inside the remaining `sandbox_sessions.rs` seam

## Next Task

`g09.006` no longer has a clearly batch-cardable broad shared-support seam in
`sandbox_sessions.rs`. Leave the strict lane awaiting the next planning
decision rather than inventing another ready batch card.
