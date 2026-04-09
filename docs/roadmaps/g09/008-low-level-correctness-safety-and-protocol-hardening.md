# 008 - Low-Level Correctness, Safety, And Protocol Hardening

Status: active
Owner: core-product
Created: 2026-04-08
Depends on: g09.001
Vision tags: `CORRECTNESS`, `IPC`, `SAFETY`
Contract refs: `001`, `032`, `056`, `076`
Strict lane refs: `docs/specs/001-g09-lane-first-strict-adoption.md`, `docs/specs/batch-cards/008-g09-008-shared-memory-lifecycle-hardening.md`

## Problem

Several low-level surfaces still hide invalid state or failure through silent
fallbacks, lossy constructors, or panic-oriented protocol handling.

## Goals

- [ ] make invalid low-level states explicit and rejectable
- [ ] remove silent graph and primitive fallback behavior
- [ ] replace panic-oriented protocol paths with typed failures

## Non-Goals

- [ ] no large performance rewrites unrelated to correctness
- [ ] no new feature breadth beyond the hardened semantics

## Execution Plan

### Batch 8.1 - Graph And Primitive Invariants

- [x] freeze the first substrate-hardening boundary so execution can begin
      without fresh planning decisions
- [x] replace unsupported channel-layout silent zeroing with explicit failure or
      degraded receipts
- [x] harden `signal-primitives` constructors against zero-count or lossy
      interleaved-buffer states
- [x] add focused negative tests for invalid graph and primitive inputs

### Batch 8.2 - Protocol Hardening

- [x] freeze the CLAP sandbox protocol hardening seam as the next ready batch
- [x] remove `expect(...)` request handling from the CLAP sandbox harness
- [x] convert internal drift cases into typed protocol failure envelopes
- [x] ensure recovery paths preserve sandbox and runtime continuity semantics

### Batch 8.3 - Shared-Memory Lifecycle Hardening

- [x] freeze the shared-memory lifecycle seam as the next ready batch
- [x] define and implement explicit ownership, stale-region cleanup, and file
      permission posture for the shared-memory broker
- [x] expose lifecycle or cleanup failures as machine-readable transport faults
- [x] add focused lifecycle tests around stale or partially-torn-down regions

## Acceptance Criteria

- [x] invalid graph or primitive states no longer fail silently
- [x] protocol handlers no longer rely on panic for expected drift handling
- [x] shared-memory ownership and cleanup are explicit and inspectable

## Risks And Mitigations

- Risk: stricter invariants break callers unexpectedly.
- Mitigation: prefer deliberate breaking changes plus migration logs over hidden
  compatibility shims.

- Risk: hardening introduces blocking or allocation in realtime paths.
- Mitigation: keep validation and cleanup off the audio thread and document any
  runtime-owned degraded receipts.

## Evidence Requirements

- [x] log each invariant and protocol-hardening tranche
- [x] run `cargo check -p signal-graph`
- [x] run `cargo check -p signal-primitives`
- [x] run `cargo check -p signal-plugin-clap`
- [x] run `cargo check -p signal-ipc`
- [x] run `effigy health`

## Batch 8.1 Tranche 1 Outcome

The first `g09.008` hardening seam is complete. `signal-primitives` now
rejects invalid zero-channel and lossy interleaved construction explicitly,
and `signal-graph` no longer lets unsupported layout adaptation masquerade as
an ordinary successful zeroed buffer path. The batch also surfaced and fixed a
real canonical-layout bug: counted two-channel layouts are now normalized back
to `Stereo`, which stops normal stereo buffers from slipping into unsupported
adaptation logic.

## Reassessment Outcome

The next honest `g09.008` seam is the remaining panic-oriented CLAP sandbox
lifecycle handling, not shared-memory lifecycle hardening yet. The CLAP
prepare/activate/deactivate/reset path is narrower, directly governed by the
current hardening contract, and can be batch-carded cleanly without pulling in
the broader ownership and stale-region design work that shared-memory cleanup
still wants.

## Batch 8.2 Tranche 1 Outcome

The targeted CLAP lifecycle hardening seam is complete. The prepare,
activate, deactivate, reset, and create-instance handlers no longer depend on
panic-oriented `expect(...)` assumptions; they now return explicit typed
protocol failures when validated instance or state projection drift occurs.
Focused lifecycle tests now cover those drift cases directly.

## Updated Reassessment Outcome

The next honest `g09.008` seam is now shared-memory lifecycle hardening. The
remaining CLAP warning noise is pre-existing and too small to justify another
strict batch card, while the shared-memory broker still has the broader
ownership and cleanup posture gap that the roadmap already captures.

## Batch 8.3 Tranche 1 Outcome

The shared-memory lifecycle seam is complete. `signal-ipc` now records brokered
region identity and byte-shape metadata in a sidecar, validates that metadata
during attach and destroy, and emits explicit lifecycle errors for missing
metadata, missing backing files, malformed sidecars, and size mismatch instead
of silently relying on best-effort temp-file cleanup. Focused IPC tests cover
the new stale and partially-torn-down cases, and the dependent CLAP and broker
recovery compile/proof surfaces stayed green after the typed error boundary was
introduced.

## Final Reassessment Outcome

There is no further honest bounded seam left inside `g09.008` without widening
into the next milestone. The low-level hardening goals frozen under contract
`076` are satisfied, so the strict lane should stop here and re-enter planning
before inventing another ready card.

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether
`g09.008` closes here or hands off into `g09.009` before creating another
ready batch card.
