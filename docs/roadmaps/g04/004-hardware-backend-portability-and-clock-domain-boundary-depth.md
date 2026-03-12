# 004 - Hardware Backend Portability And Clock-Domain Boundary Depth

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g04.002, g04.003
Vision tags: `HARDWARE`, `CLOCKING`, `PORTABILITY`

## Problem

Signal’s current host/device path is credible, but the repo still needs a more
deliberate reusable answer for backend portability, aggregate/independent clock
domains, and cross-backend fallback behavior.

Without a dedicated portability milestone:

- hardware assumptions remain too shaped by the current preferred path
- aggregate or multi-clock-domain behavior stays under-specified
- later consumer packaging and backend breadth will reopen device-boundary work
- scheduling and render semantics will drift across backends

## Goals

- [ ] define backend-neutral hardware/runtime boundaries inside Signal
- [ ] make clock-domain crossings and fallback behavior explicit
- [ ] keep resampling and clock-boundary semantics runtime-owned
- [ ] provide receipts strong enough for consumers to reason about hardware state

## Non-Goals

- [ ] no app-level device picker UX work
- [ ] no network/distributed audio topology here
- [ ] no exhaustive certification matrix in the first portability pass

## Execution Plan

### Batch 4.1 - Backend And Clock Contract

- [x] define the reusable backend capability and clock-domain model
- [x] document when a path is same-clock, cross-clock, aggregate, or degraded
- [x] decide what belongs in host-neutral exports versus backend-private detail

### Batch 4.2 - Runtime Boundary Depth

- [x] implement stronger clock-domain and fallback handling in Signal-owned
  runtime/hardware crates
- [x] keep resampling and scheduling behavior aligned with the runtime-owned
  contract rather than backend-local shortcuts
- [x] preserve recovery and degradation semantics across backend changes

### Batch 4.3 - Focused Portability Proofs

- [x] add focused proofs for backend capability projection, degraded fallback,
  or clock-domain crossing behavior
- [x] record the residual backend breadth left for later work explicitly

## Progress Notes

- 2026-03-12: completed Batch 4.1 by freezing the backend-neutral hardware
  capability and clock-domain contract in
  `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`,
  grounding it in the existing `signal-hardware` negotiation types plus
  `signal-runtime` host I/O reports, and explicitly separating host-neutral
  export from backend-private aggregate or drift detail that still needs typed
  runtime receipts later.
- 2026-03-12: advanced Batch 4.2 by adding backend-neutral clock-topology
  hints in `signal-hardware`, explicit `clock_domain` and `fallback_state`
  export in `RuntimeHostClockingSummary`, and focused host proofs for
  same-clock direct, cross-clock runtime-resampled, and degraded
  recovery-constrained hardware state without pushing timing policy into
  backend-local code.
- 2026-03-12: completed the remaining Batch 4.2 and Batch 4.3 closure work by
  extending the same host receipt family with explicit `transition_state`,
  proving aggregate-clock entry plus return-to-direct recovery on the shared
  host export path, and recording the deferred scope around multi-member
  aggregate detail, drift compensation, and wider backend-matrix breadth.

## Acceptance Criteria

- [ ] hardware/runtime portability has a reusable contract inside Signal
- [ ] clock-domain and fallback behavior are explicit and inspectable
- [ ] later plugin/consumer breadth can build on a stable device boundary

## Risks and Mitigations

- Risk: backend portability work becomes a grab bag of backend-specific hacks.
- Mitigation: freeze one backend-neutral contract before widening implementation.
- Risk: clock-domain work quietly changes timing semantics.
- Mitigation: require explicit receipts and focused validation around crossings.

## Evidence Requirements

- [ ] log each meaningful hardware portability tranche
- [ ] run focused validation for backend or clock-boundary behavior
- [ ] record any intentionally deferred backend matrix scope

## Next Task

Continue `g04.005` with Batch 5.2 and deepen the typed plugin backend and
host-neutral delegation surfaces on top of the now-closed hardware portability
boundary.
