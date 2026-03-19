# 010 - Linux Backend Clocking, Duplex, And Endpoint-Topology Parity

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.009
Vision tags: `LINUX`, `CLOCKING`, `HARDWARE`

## Problem

Backend presence alone is not enough. Linux needs parity on clocking, duplex,
and endpoint-topology behavior before Loophole can trust it as a real runtime
target.

## Goals

- [ ] define Linux parity for clocking, duplex, and endpoint-topology behavior
- [ ] align Linux backend observation with the shared hardware recovery model
- [ ] keep host-visible latency, drift, and mismatch state explicit

## Non-Goals

- [ ] no network-audio topology work here
- [ ] no product-local device setup UX

## Execution Plan

### Batch 10.1 - Parity Contract

- [x] define Linux parity expectations for clocking, duplex, and endpoint topology
- [x] classify backend-private behavior explicitly

### Batch 10.2 - Runtime Depth

- [x] align Linux backend observation and recovery receipts with the parity contract
- [x] keep host-edge surfaces on one hardware vocabulary

### Batch 10.3 - Focused Proof

- [x] add focused proofs for Linux clocking, duplex, and endpoint-topology behavior

## Acceptance Criteria

- [x] Signal has explicit Linux hardware parity for clocking and topology
- [x] later external-I/O and control-surface depth can rely on the same Linux base
- [x] unsupported Linux behavior remains explicit rather than implied

## Risks And Mitigations

- Risk: Linux parity claims overreach actual runtime evidence.
- Mitigation: require focused proof and unsupported-state receipts.

## Evidence Requirements

- [x] log each meaningful Linux parity tranche
- [x] run focused Linux hardware parity validation
- [x] record explicit unsupported parity explicitly

## Batch 10.1 Outcome

Batch 10.1 freezes the bounded Linux backend clocking, duplex, and
endpoint-topology parity contract in
`docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`.

Signal now has one explicit Linux-facing contract for:

- how ALSA, JACK, and PipeWire clocking posture must compose through the
  existing shared drift and discontinuity boundary rather than backend-private
  timing stories
- how duplex coherence, partial availability, and guarded mismatch remain
  shared runtime meaning instead of a Linux-only host table
- how endpoint-topology shape and mutation must stay on one Signal-owned
  vocabulary while backend-native graph detail remains advisory

That gives Batch 10.2 one fixed runtime target for Linux hardware receipt work
without drifting into daemon-local endpoint models or backend-private clocking
claims.

## Batch 10.2 Outcome

Batch 10.2 turns the Linux backend parity contract into a real runtime-owned
receipt family.

Signal now has:

- explicit Linux-specific clocking, duplex, and endpoint-topology parity
  fields on the shared `RuntimeHostClockingSummary` and
  `RuntimeExternalIoSnapshot` surfaces instead of forcing consumers to infer
  Linux parity from generic clock state plus backend identity
- one runtime-owned classifier path for ALSA, JACK, PipeWire, non-Linux, and
  unavailable host contexts, so local host, server host, and later public
  runtime proof surfaces stay on the same Linux hardware vocabulary
- focused runtime and host validation covering portable ALSA-style clocking,
  guarded JACK-style aggregate parity, and explicit unsupported parity on the
  current non-Linux and unavailable host paths

That leaves Batch 10.3 with the narrower downstream-proof job instead of more
receipt shaping.

## Batch 10.3 Outcome

Batch 10.3 closes the bounded Linux backend clocking, duplex, and
endpoint-topology consumer seam across public runtime, both stable host edges,
and `signal-supervisor-tools`.

Signal now has:

- downstream-style runtime proof that ALSA, JACK, PipeWire, non-Linux, and
  unavailable host contexts keep Linux-specific parity truth consumable through
  shared observation and supervisor receipts instead of backend-private Linux
  capability matrices
- stable local-host and server-host proof that explicit unsupported or
  unavailable Linux parity remains typed on shared host-edge export instead of
  falling back to omitted fields or host-local heuristics
- the machine-readable
  `signal.runtime.linux-backend-clock-topology-boundary` descriptor plus the
  repo-owned `effigy acceptance:linux-backend-clock-topology-boundary` task, so
  downstream consumers can inspect and rerun the proof seam without reading
  backend-private host code

That closes `g07.010` and moves the active queue to `g07.011`.

## Next Task

Continue `g07.012` with Batch 12.2 by materializing the first runtime-owned
MIDI 2.0, MPE, and richer controller-expression receipt family across runtime,
plugin, and hardware surfaces without reopening adapter-private packet
ownership.
