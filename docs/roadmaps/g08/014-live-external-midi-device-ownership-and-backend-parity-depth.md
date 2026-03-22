# 014 - Live External MIDI Device Ownership And Backend Parity Depth

Status: complete
Owner: core-product
Created: 2026-03-21
Depends on: g08.013
Vision tags: `MIDI`, `LIVE`, `BACKEND`

## Problem

`g08.013` closes the bounded transform-persistence seam, but live external
MIDI device ownership and backend parity are still at risk of drifting into
backend-local endpoint policy, host-local device identity glue, or product-
local live workflow assumptions.

Without a runtime-owned contract here, later live MIDI and acceptance work
will either reopen endpoint ownership outside Signal-owned receipts or split
device truth across backend adapters, hosts, and workflow-specific transport
logic.

## Goals

- [ ] freeze one runtime-owned authority line for live external MIDI device
      ownership and backend parity
- [ ] keep external MIDI ownership composable with the closed external MIDI,
      controller, Linux live-backend, and transform-persistence seams
- [ ] avoid backend-local device identity or host-local endpoint policy
      becoming shared truth

## Non-Goals

- [ ] no product-local controller UX or live performance workflow policy
- [ ] no backend-specific device browser, routing console, or session-manager
      shell as the shared contract

## Execution Plan

### Batch 14.1 - Live MIDI Ownership Contract

- [x] freeze runtime-owned live external MIDI device ownership and backend
      parity meaning
- [x] define shared runtime versus backend-local or host-local authority
      explicitly

### Batch 14.2 - Runtime Live MIDI Ownership Baseline

- [x] materialize the first runtime-owned live external MIDI ownership and
      backend-parity receipts
- [x] align stable host-edge export with the same bounded model

### Batch 14.3 - Consumer Proof

- [x] prove the widened live MIDI ownership seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] live external MIDI device ownership and backend parity are runtime-owned
      and inspectable
- [x] backend-local or host-local MIDI device detail stays bounded and typed
- [x] later live MIDI and acceptance work can build on one explicit endpoint
      ownership and parity authority line

## Risks And Mitigations

- Risk: live MIDI endpoint ownership drifts into backend-local device identity,
  host-local endpoint policy, or product-specific live workflow glue.
- Mitigation: freeze one runtime-owned contract before widening runtime
  realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Batch 14.1 Outcome

- `g08` now has a frozen live external MIDI ownership and backend-parity
  contract in
  `docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md`
  instead of leaving this seam implicit under the older external MIDI, live
  backend, and backend-parity contracts
- live external MIDI ownership, attach continuity, and backend parity are now
  required to compose through the closed external MIDI graph, controller-
  expression, live backend lifecycle, backend parity, and transform-
  persistence seams rather than backend-local endpoint policy or host-local
  device picks
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until Batch 14.2 and Batch 14.3

## Batch 14.2 Outcome

- `signal-runtime` now widens the existing external MIDI seam with a typed
  `live_ownership` summary on `RuntimeExternalMidiEndpointGraphSnapshot`
  instead of opening a second live-MIDI-only report family
- the new runtime-owned receipt family carries ownership posture, attach
  continuity, backend parity, and guarded parity outcome, derived from the
  existing Linux-session and interruption seams rather than backend-local
  device picks or session-manager policy
- the same live ownership and parity truth now flows through public runtime
  surfaces and stable local or server host-edge export without host-local
  reclassification

## Batch 14.3 Outcome

- `signal-supervisor-tools` now widens the existing
  `signal.runtime.external-midi-boundary` so it proves the bounded live
  external MIDI ownership and backend-parity seam on the same shared
  supervisor descriptor instead of opening a second live-MIDI-only acceptance
  lane
- the shared boundary now points at
  `docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md`
  and explicitly describes `live_ownership`, `ownership_posture`,
  `attach_continuity`, `backend_parity`, and `guarded_parity_outcome`
  alongside the earlier external MIDI graph anchors
- `g08.014` is now complete, and the next queue is cross-backend device
  protocol and live workflow acceptance

## Next Task

Continue `g08.015` with Batch 15.1 by freezing the shared cross-backend
device protocol and live workflow acceptance contract on top of the closed
live external MIDI ownership seam.
