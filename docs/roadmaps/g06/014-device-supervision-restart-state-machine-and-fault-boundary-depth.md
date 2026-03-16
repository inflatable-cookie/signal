# 014 - Device Supervision, Restart-State Machine, And Fault-Boundary Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.005
Vision tags: `HARDWARE`, `RECOVERY`, `RUNTIME`

## Problem

Signal already owns basic hardware portability and device-backed runtime state,
but Loophole's next runtime lane needs deeper reusable supervision and restart
policy so products do not keep carrying too much restart logic.

## Goals

- [ ] define a stronger runtime-owned device supervision and restart-state model
- [ ] make recovering-versus-faulted hardware behavior explicit
- [ ] support later monitoring, external-I/O, and soak work on top of one
  supervision substrate

## Non-Goals

- [ ] no product-specific device setup UX
- [ ] no exhaustive hardware certification matrix

## Execution Plan

### Batch 14.1 - Supervision Contract

- [x] define restart states, recovery exhaustion, and fault-boundary semantics
- [x] align device supervision with the shared interruption and fault taxonomy

### Batch 14.2 - Runtime Supervision Depth

- [x] materialize stronger device supervision and restart receipts in runtime
- [x] keep host-edge and supervisor exports aligned with the new state model

### Batch 14.3 - Focused Recovery Proof

- [x] add focused proofs for recovery, exhaustion, and explicit faulted hardware
  outcomes

## Acceptance Criteria

- [ ] Signal has a stronger device supervision and restart-state model
- [ ] products can observe hardware recovery truth without owning restart policy
- [ ] later hardware/monitoring work builds on reusable runtime supervision

## Risks And Mitigations

- Risk: supervision policy stays backend-private.
- Mitigation: freeze backend-neutral restart and fault semantics first.
- Risk: recovery logic still lives mostly in host wrappers.
- Mitigation: require runtime-owned receipts for restart and exhaustion outcomes.

## Evidence Requirements

- [ ] log each meaningful hardware-supervision tranche
- [ ] run focused validation for recovery and exhaustion behavior
- [ ] record deferred hardware-matrix breadth explicitly

## Batch 14.1 Outcome

Batch 14.1 freezes the first bounded runtime-owned device supervision and
restart-state contract in
`docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`.

The repo now has an explicit rule set for:

- treating restart episodes, recovering hardware state, exhaustion, and the
  hardware fault boundary as shared Signal-owned vocabulary rather than host
  restart-loop folklore
- keeping `signal-hardware` diagnostics and host callback evidence additive to
  runtime supervision instead of letting them become a competing consumer
  taxonomy
- aligning later hardware supervision work to the existing interruption and
  fault-cause contracts instead of inventing a separate hardware-only recovery
  model
- giving Batch 14.2 one fixed supervision target before deeper runtime receipt
  and export work begins

## Batch 14.2 Outcome

Batch 14.2 turns the supervision contract into a real shared receipt family.

The runtime now carries a bounded `RuntimeDeviceSupervisionSnapshot` through
runtime observation and supervisor export, with explicit state for:

- steady versus recovering versus exhausted versus faulted device supervision
- restart episode classification (`Unneeded`, `Attempting`, `Recovered`,
  `Exhausted`, `Faulted`)
- the hardware fault boundary and interruption alignment
- host-fed evidence such as device-loss counts, restart attempts, restart
  failures, watchdog restarts, restart policy, backend health, stream state,
  and active device identity

`signal-host-local` now enriches the shared runtime-owned observation and
supervisor reports with host I/O evidence instead of forcing consumers to infer
 recovery state from host-private summaries. Batch 14.2 also adds focused proof
 coverage for recovered and exhausted device-loss episodes on the shared report
 seam.

## Batch 14.3 Outcome

Batch 14.3 closes `g06.014` at the shared consumer boundary.

The repo now has:

- a downstream-style runtime proof for the public device-supervision seam
- stable host-edge proofs for local and server supervisor reports
- a machine-readable `signal.runtime.device-supervision-boundary` descriptor in
  `signal-supervisor-tools`
- a repo-owned `effigy acceptance:device-supervision-boundary` task that keeps
  the proof surface runnable

This closes the first bounded runtime-owned hardware supervision lane for
recovered, exhausted, and explicit faulted outcomes. Later hardware work can
now deepen clock drift, duplex mismatch, and endpoint-topology behavior on top
of that supervision substrate instead of reopening restart ownership.

## Next Task

Continue `g06.015` with Batch 15.1 by freezing the runtime-owned clock-domain
drift, duplex mismatch, discontinuity, and endpoint-topology contract on top of
the closed `g06.014` supervision boundary.
