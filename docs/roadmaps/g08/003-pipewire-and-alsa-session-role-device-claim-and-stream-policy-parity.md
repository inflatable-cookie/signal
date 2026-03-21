# 003 - PipeWire And ALSA Session-Role, Device-Claim, And Stream-Policy Parity

Status: complete
Owner: core-product
Created: 2026-03-19
Depends on: g08.002
Vision tags: `LINUX`, `PIPEWIRE`, `ALSA`

## Problem

`g08.001` closed the bounded live Linux ownership seam and `g08.002` closed
the first JACK-native coordination seam, but PipeWire and ALSA still do not
share one explicit runtime-owned answer for session role, device-claim
posture, and stream-policy parity. Without that boundary, later Linux device,
preview, and workflow work will drift back into backend-private or host-local
policy.

## Goals

- [x] freeze runtime-owned PipeWire and ALSA session-role, device-claim, and stream-policy parity meaning
- [x] expose one bounded parity substrate across shared runtime and stable host edges
- [x] keep backend-native daemon, node, and stream detail additive rather than authoritative

## Non-Goals

- [ ] no exhaustive PipeWire graph-policy or ALSA distro-policy matrix here
- [ ] no product-local device browser, session UI, or repair UX

## Execution Plan

### Batch 3.1 - PipeWire And ALSA Parity Contract

- [x] freeze runtime-owned PipeWire and ALSA session-role, device-claim, and stream-policy parity meaning
- [x] define shared runtime versus backend-native authority explicitly

### Batch 3.2 - Runtime PipeWire And ALSA Baseline

- [x] materialize the first runtime-owned PipeWire and ALSA parity receipts
- [x] align stable host-edge export with the same parity model

### Batch 3.3 - Consumer Proof

- [x] prove the widened PipeWire and ALSA parity seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] PipeWire and ALSA session-role, device-claim, and stream-policy parity are runtime-owned and inspectable
- [x] backend-native daemon or stream detail stays bounded and typed
- [x] later Linux workflow and acceptance work can build on one explicit PipeWire and ALSA authority line

## Risks And Mitigations

- Risk: PipeWire and ALSA stream-policy truth drifts into host-private daemon or stream wrappers.
- Mitigation: freeze one runtime-owned parity contract before widening runtime realization.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after the runtime baseline lands
- [x] record the next milestone step explicitly

## Batch 3.1 Outcome

Batch 3.1 freezes the bounded PipeWire and ALSA parity contract in
`docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`.
That contract layers PipeWire and ALSA parity on top of the closed live Linux
ownership and JACK coordination seams instead of inventing separate daemon or
callback policy shells.

It now makes the authority line explicit:

- live ownership, lifecycle, device-claim, and guarded fallback remain
  anchored in the closed `052` contract
- JACK-native transport and graph coordination remain anchored in the closed
  `053` contract and must not be reopened as generic Linux parity terms
- PipeWire and ALSA session-role, device-claim, and stream-policy parity must
  compose through shared host-I/O, clocking, transfer-policy, and supervision
  seams instead of backend-private wrappers
- Batch 3.2 now has one bounded contract target for runtime-owned PipeWire and
  ALSA parity receipts before public proof widens in Batch 3.3

## Batch 3.2 Outcome

Batch 3.2 lands the first runtime-owned PipeWire and ALSA parity receipt
family directly in `signal-runtime` and threads it through the same
observation, supervisor, and stable host-edge export path already used for
live Linux ownership and JACK coordination.

The widened baseline now proves:

- `RuntimePipeWireAlsaParitySnapshot` owns the shared session-role,
  device-claim, stream-policy, and guarded-parity answer for ALSA and
  PipeWire instead of leaving those classifications inside daemon-local or
  callback-local host summaries
- local and server host report assembly now feed the same runtime-owned parity
  receipt into stable host-edge export instead of reconstructing PipeWire or
  ALSA posture on the host boundary
- focused runtime and stable host-edge proofs now cover:
  - non-target local host export (`NotPipeWireOrAlsa`)
  - direct ALSA callback parity
  - backend-managed PipeWire parity
  - recovery-guarded PipeWire parity

Batch 3.3 can now stay narrowly about consumer proof and acceptance surfacing
instead of reopening runtime classification.

## Batch 3.3 Outcome

Batch 3.3 closes `g08.003` by turning the widened PipeWire and ALSA parity
receipt family into a repo-owned consumer boundary instead of leaving it as
runtime-only proof.

It now proves:

- `signal-supervisor-tools` exposes
  `signal.runtime.pipewire-alsa-parity-boundary` as the machine-readable
  consumer descriptor for the new receipt family
- `effigy acceptance:pipewire-alsa-parity-boundary` composes the public
  runtime proof, stable local/server host-edge proofs, and descriptor proof
  into one reusable acceptance lane
- later Linux workflow and acceptance work can now build on one explicit
  PipeWire and ALSA authority line instead of reopening host-local parity
  reconstruction

`g08.003` is complete.

## Next Task

Open `g08.004` with Batch 4.1 by freezing the first runtime-owned LV2 worker,
URID, patch, and extension-negotiation contract on top of the now-closed live
Linux ownership, JACK coordination, and PipeWire/ALSA parity seams.
