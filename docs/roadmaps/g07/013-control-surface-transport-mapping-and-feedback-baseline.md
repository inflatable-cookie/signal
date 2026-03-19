# 013 - Control-Surface Transport, Mapping, And Feedback Baseline

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.011, g07.012
Vision tags: `CONTROL`, `HARDWARE`, `MIDI`

## Problem

Chorus already expects Signal to handle low-level control-surface discovery and
feedback transport, but Signal still needs a reusable baseline for that work.

## Goals

- [ ] define a reusable control-surface transport and feedback baseline
- [ ] keep mapping-relevant runtime meaning explicit without absorbing product policy
- [ ] support practical device feedback and controller I/O on one substrate

## Non-Goals

- [ ] no product-specific mapping UI or workflow layer
- [ ] no full scripting or extension policy yet

## Execution Plan

### Batch 13.1 - Control-Surface Contract

- [x] define device identity, transport, feedback, and capability meaning
- [x] align the contract with external MIDI endpoint and event surfaces

### Batch 13.2 - Runtime Baseline

- [x] implement the first credible control-surface transport and feedback path
- [x] keep host-visible state and diagnostics aligned with the contract

### Batch 13.3 - Focused Proof

- [x] add focused proofs for control-surface transport and feedback behavior

## Acceptance Criteria

- [x] Signal has an explicit control-surface transport and feedback baseline
- [x] later mapping and extensibility work can build on the same device substrate
- [x] low-level device transport remains outside app-local glue

## Risks And Mitigations

- Risk: control-surface work becomes privileged hardware exception handling.
- Mitigation: freeze one reusable device and feedback contract first.

## Evidence Requirements

- [x] log each meaningful control-surface tranche
- [x] run focused control-surface validation
- [x] record deferred control-surface breadth explicitly

## Batch 13.1 Outcome

Batch 13.1 freezes the first runtime-owned control-surface contract in
`docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`.

Signal now has one shared contract for:

- control-surface device identity, transport posture, feedback readiness, and
  bounded capability meaning instead of host-local controller integration logic
- mapping posture that is explicit and reusable without absorbing product
  mapping workflow or UI policy
- direct composition with the closed external MIDI endpoint and widened
  controller-expression boundaries instead of inventing a second controller or
  device shell

That gives Batch 13.2 one fixed runtime target for control-surface baseline
work while keeping scripting, vendor protocol breadth, and product-local
mapping semantics explicitly deferred.

## Batch 13.2 Outcome

Batch 13.2 turns the frozen control-surface contract into a real shared runtime
baseline.

Signal now has:

- runtime-owned `RuntimeControlSurfaceSnapshot` and per-device control-surface
  descriptors derived from the closed external MIDI endpoint graph instead of
  host-local controller tables
- explicit transport posture, mapping posture, feedback readiness, and guarded
  widened-expression capability on observation, supervisor, and stable host-edge
  surfaces
- aligned host-local and server-host JSON export that forwards the same
  control-surface baseline rather than rebuilding controller truth outside
  `signal-runtime`

This keeps Batch 13.2 broad enough to be meaningful without pretending the
public proof seam is already closed. Machine-readable boundary proof, acceptance
automation, and richer device feedback depth still belong to Batch 13.3.

## Batch 13.3 Outcome

Batch 13.3 closes the bounded control-surface consumer seam.

Signal now has:

- focused downstream-style proof that `RuntimeControlSurfaceSnapshot` remains
  consumable through public runtime, both stable host edges, and a
  machine-readable supervisor-tools boundary descriptor
- a repo-owned acceptance lane for the control-surface boundary instead of a
  prose-only claim about transport, mapping posture, feedback readiness, and
  bounded capability truth
- one explicit handoff into later advanced hardware and scripting-safe device
  policy work without reopening host-local controller-policy reconstruction

This closes `g07.013` as the bounded control-surface baseline milestone. Richer
vendor protocol, display, haptic, motor, and scripting-safe extensibility depth
remain explicit `g07.014` work rather than silent scope creep inside this
baseline.

## Next Task

Continue `g07.014` with Batch 14.1 by freezing the runtime-owned advanced
hardware extensibility, scripting-safe device policy, and guarded feedback
contract on top of the now-closed control-surface baseline.
