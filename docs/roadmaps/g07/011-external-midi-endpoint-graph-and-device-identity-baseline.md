# 011 - External MIDI Endpoint Graph And Device-Identity Baseline

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g06.012, g06.016
Vision tags: `MIDI`, `HARDWARE`, `IDENTITY`

## Problem

Loophole's next hardware and controller depth needs a reusable Signal-owned
model for external MIDI endpoints, identity, capability, and routing.

## Goals

- [ ] define a reusable external MIDI endpoint graph and identity surface
- [ ] support runtime-owned MIDI endpoint discovery and routing semantics
- [ ] keep host-visible device and endpoint state explicit

## Non-Goals

- [ ] no product-specific MIDI browser or mapping UX
- [ ] no control-surface scripting depth yet

## Execution Plan

### Batch 11.1 - Endpoint Contract

- [x] define MIDI endpoint identity, topology, capability, and lifecycle meaning
- [x] align the contract with existing hardware and event models

### Batch 11.2 - Runtime Baseline

- [x] implement the first credible external MIDI endpoint graph baseline
- [x] keep discovery, health, and routing observation aligned with the contract

### Batch 11.3 - Focused Proof

- [x] add focused proofs for external MIDI endpoint discovery and routing behavior

## Acceptance Criteria

- [x] Signal has an explicit external MIDI endpoint graph and identity surface
- [x] later control-surface and richer controller-expression work can build on it
- [x] hosts can observe MIDI endpoint truth without local shims

## Risks And Mitigations

- Risk: MIDI hardware depth gets rebuilt as app-local glue.
- Mitigation: freeze one reusable endpoint graph and capability contract first.

## Evidence Requirements

- [x] log each meaningful external-MIDI tranche
- [x] run focused endpoint and routing validation
- [x] record deferred MIDI device breadth explicitly

## Batch 11.1 Outcome

Batch 11.1 freezes the bounded external MIDI endpoint and device-identity
contract in
`docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`.

Signal now has one shared contract for:

- runtime-owned external MIDI device identity instead of backend-native port,
  client, or product-local naming becoming the consumer boundary
- bounded endpoint graph, capability, lifecycle, and route meaning that later
  runtime work must target
- explicit reuse of the closed generic MIDI event contract and shared hardware
  or supervision boundary instead of inventing a second MIDI-only lifecycle or
  event shell

That gives Batch 11.2 one fixed runtime target for external MIDI endpoint
receipt work without drifting into product-local browser models or backend-
private patchbay semantics.

## Batch 11.2 Outcome

Batch 11.2 turns the external MIDI contract into a real runtime-owned receipt
family instead of leaving it as roadmap prose.

Signal now has:

- explicit runtime-owned external MIDI graph, device, endpoint, capability, and
  route receipts on observation and supervisor surfaces, with typed
  `Unavailable` and `Empty` outcomes instead of missing-field gaps
- one shared runtime snapshot for downstream host export, so local and server
  host edges both publish the same `Empty` graph baseline rather than
  rebuilding device state in host-local code
- focused runtime and host coverage proving the first baseline through compact,
  multiline, and JSON report rendering before the public proof batch widens

That leaves Batch 11.3 with the narrower public runtime, supervisor-tools, and
stable host-edge consumer proof job rather than more DTO shaping.

## Batch 11.3 Outcome

Batch 11.3 closes the bounded external MIDI consumer seam across public
runtime, both stable host edges, and `signal-supervisor-tools`.

Signal now has:

- downstream-style runtime proof that typed `Unavailable` and `Empty` external
  MIDI graph state stays consumable through shared observation and supervisor
  receipts instead of hidden host-private device tables
- stable local-host and server-host proof that both shared host edges forward
  the same runtime-owned empty external MIDI graph baseline rather than
  reconstructing MIDI device truth locally
- the machine-readable `signal.runtime.external-midi-boundary` descriptor plus
  the repo-owned `effigy acceptance:external-midi-boundary` task, so
  downstream consumers can inspect and rerun the proof seam without reading
  host-private MIDI integration code

That closes `g07.011` and moves the active queue to `g07.012`.

## Next Task

Continue `g07.012` with Batch 12.1 by freezing the widened MIDI 2.0, MPE, and
richer controller-expression contract on top of the now-closed external MIDI
endpoint graph and generic event boundaries.
