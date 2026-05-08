# 008 - g09.008 Shared-Memory Lifecycle Hardening

Status: complete
Owner: core-product
Updated: 2026-04-09
Master spec refs: docs/specs/001-g09-lane-first-strict-adoption.md
Roadmap refs: g09.008
Governing refs: docs/contracts/001-working-rules.md, docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md, docs/contracts/076-low-level-correctness-safety-and-protocol-hardening-contract.md, docs/specs/001-g09-lane-first-strict-adoption.md, docs/roadmaps/g09/008-low-level-correctness-safety-and-protocol-hardening.md
Auto-start next card: no

## Objective

Continue `g09.008` with the next substrate-hardening seam: make shared-memory
region ownership and cleanup explicit in `signal-ipc` so stale, missing, or
partially torn-down regions return machine-readable lifecycle failures instead
of relying on best-effort temp-file behavior alone.

## Scope

- harden `crates/signal-ipc/src/shared_memory.rs` ownership and cleanup posture
- add explicit lifecycle checks around missing, stale, or size-mismatched
  shared-memory regions
- keep the batch inside shared-memory lifecycle and ownership semantics; do not
  widen into broader sandbox protocol redesign
- add focused IPC lifecycle tests for the hardened region behavior

## Steps

1. Define explicit region-lifecycle failure outcomes for stale, missing, or
   mismatched mapped-file regions.
2. Tighten create/attach/destroy behavior so region ownership and cleanup are
   inspectable instead of relying on best-effort temp-dir deletion only.
3. Add focused lifecycle tests that prove stale or partially torn-down regions
   fail explicitly.
4. Rerun the focused IPC validation surface plus repo health.

## Acceptance Criteria

- shared-memory region lifecycle and cleanup behavior is explicit and
  machine-readable
- stale or mismatched region attachment no longer depends on implicit temp-file
  posture
- focused IPC lifecycle tests cover the hardened ownership cases
- focused validation passes

## Evidence Required

- batch log for the next `g09.008` tranche
- validation actually run
- explicit note if any broader permission-policy work remains intentionally
  deferred

## Stop Conditions

- the batch starts redesigning the wider sandbox transport contract instead of
  hardening mapped-region lifecycle
- the change needs broader host/runtime lease semantics not already captured in
  contract `076`
- the work drifts into unrelated CLAP or runtime recovery cleanup

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
`g09.008` closes here or hands off into `g09.009` before creating another
ready batch card.

## Outcome

`signal-ipc` now exposes explicit shared-memory lifecycle posture instead of
best-effort mapped-file cleanup alone. Region creation writes a metadata
sidecar with ownership and byte-shape data, attach and destroy validate that
sidecar before trusting the mapped file, and missing, stale, malformed, or
size-mismatched regions now return typed machine-readable lifecycle failures.
The broker also tightens root/file permission posture on Unix and downstream
recovery cleanup continues to compile and pass focused broker recovery proofs
through the new typed error boundary.

## Validation Run

- `cargo test -p signal-ipc`
- `cargo check -p signal-plugin-clap`
- `cargo check -p signal-host-local`
- `cargo check -p signal-host-server`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `effigy health`
