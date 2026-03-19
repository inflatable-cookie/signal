# 009 - Linux Audio Backend Portability Across ALSA, JACK, And PipeWire

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.008, g06.014, g06.015
Vision tags: `LINUX`, `HARDWARE`, `BACKENDS`

## Problem

Signal's current backend portability work does not yet provide a deliberate
Linux-native hardware story across ALSA, JACK, and PipeWire.

## Goals

- [ ] define the first explicit Linux audio backend portability surface
- [ ] support ALSA, JACK, and PipeWire under one runtime-owned contract
- [ ] keep hardware, diagnostics, and restart semantics coherent across Linux backends

## Non-Goals

- [ ] no exhaustive distro certification matrix
- [ ] no product-specific Linux setup UX

## Execution Plan

### Batch 9.1 - Linux Backend Contract

- [x] define backend identity, capability, and lifecycle meaning across ALSA, JACK, and PipeWire
- [x] align the contract with the existing hardware portability model

### Batch 9.2 - Backend Baselines

- [x] add the first credible Linux backend baselines as needed
- [x] keep diagnostics, restart policy, and host-edge receipts aligned

### Batch 9.3 - Focused Proof

- [x] add focused proofs for Linux backend portability and fallback behavior

## Acceptance Criteria

- [x] Signal has an explicit Linux hardware backend portability surface
- [x] Linux hardware behavior stays runtime-owned and inspectable
- [x] later endpoint-topology and control-surface work can build on the same base

## Risks And Mitigations

- Risk: Linux backend work fragments into backend-private shells.
- Mitigation: freeze one hardware contract first and prove widened paths through it.

## Evidence Requirements

- [x] log each meaningful Linux backend tranche
- [x] run focused ALSA, JACK, and PipeWire validation as available
- [x] record deferred Linux backend breadth explicitly

## Batch 9.1 Outcome

Batch 9.1 freezes the first bounded Linux audio backend portability contract
in `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`.

Signal now has one explicit Linux hardware backend vocabulary for:

- ALSA, JACK, and PipeWire backend identity and guarded portability meaning
- how Linux backend lifecycle, supervision, restart, clocking, and endpoint
  interpretation must reuse the existing shared hardware, supervision, and
  clock-domain contracts instead of growing Linux-private shells
- what remains explicitly deferred, including distro-specific setup breadth,
  backend-native daemon or graph semantics, and richer Linux session detail

That gives Batch 9.2 one fixed contract target for runtime baseline work
without drifting into backend-local ownership.

## Batch 9.2 Outcome

Batch 9.2 materializes the first real Linux backend baseline through the
shared hardware and runtime receipt family instead of inventing backend-private
host shells.

Signal now has:

- typed backend identity in `signal-hardware` through
  `HardwareBackendIdentity` and `LinuxAudioBackendKind`
- simulated ALSA, JACK, and PipeWire baselines with distinct lifecycle and
  clock posture so Linux backend differences land through one shared hardware
  contract
- runtime-owned Linux backend identity and portability-band classification on
  `RuntimeHostHardwareSummary` and `RuntimeExternalIoSnapshot`
- focused proofs that ALSA, JACK, and PipeWire now surface as shared typed
  runtime baselines while non-Linux paths stay explicit as `NotLinux` /
  `Unsupported`

This keeps Batch 9.2 at the right level: one runtime-owned Linux backend
baseline and diagnostic story, not a premature live ALSA, JACK, or PipeWire
host implementation.

## Batch 9.3 Outcome

Batch 9.3 closes the bounded Linux backend portability proof seam across
public runtime, the stable server host edge, and a machine-readable
supervisor-tools descriptor.

Signal now has:

- downstream-style public runtime proof that ALSA, JACK, PipeWire, and
  unavailable Linux backend identity plus portability-band answers remain
  consumable through `RuntimeObservationReport` and `RuntimeSupervisorReport`
- stable server-host proof that the Linux-facing host edge forwards explicit
  runtime-owned unavailable Linux backend and fallback truth instead of
  inventing host-local Linux capability matrices
- a repo-owned `signal.runtime.linux-audio-backend-boundary` descriptor and
  Effigy acceptance task so the bounded proof seam is inspectable without
  reading backend-private host code

This closes `g07.009` as a bounded Linux backend portability queue. The next
Linux hardware depth now belongs to clocking, duplex, and endpoint-topology
parity in `g07.010`.

## Next Task

Continue `g07.010` with Batch 10.1 by freezing the runtime-owned Linux backend
clocking, duplex, and endpoint-topology parity contract on top of the now-
closed Linux backend portability boundary.
