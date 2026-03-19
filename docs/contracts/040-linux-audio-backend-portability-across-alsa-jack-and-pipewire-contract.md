# 040 Linux Audio Backend Portability Across ALSA, JACK, And PipeWire Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`, `docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`, `docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first Linux-specific audio backend portability boundary for
`g07.009` so later ALSA, JACK, and PipeWire runtime work can widen one shared
hardware vocabulary without reopening backend-private restart policy,
Linux-only health taxonomies, or host-local capability matrices.

## Authority hierarchy

Linux audio backend portability has one authority chain:

1. `signal-hardware` owns backend-neutral capability and negotiated hardware
   primitives for:
   - backend identity
   - negotiated stream contracts
   - lifecycle ownership and restart policy
   - diagnostic evidence and backend health
2. `signal-runtime` owns canonical Linux backend interpretation for:
   - active hardware contract and applied runtime configuration
   - supervision, interruption, degradation, and fault-boundary meaning
   - Linux backend portability classification and additive fallback state
   - observation, supervisor, and acceptance export delivery
3. host crates may broker backend callbacks, stream negotiation, and Linux
   backend evidence into runtime-owned summaries, but they must not become the
   authority for:
   - ALSA versus JACK versus PipeWire portability claims
   - competing Linux backend lifecycle or restart taxonomies
   - backend-specific unsupported-state matrices outside shared receipts

If a Linux backend portability claim cannot be explained through
`signal-hardware`, `signal-runtime`, and additive shared host receipts, it is
not yet part of the reusable Signal contract.

## Existing anchors

This contract is grounded in the current hardware and supervision surface
family:

- `AudioDeviceDescriptor`
- `HardwareStreamRequest`
- `HardwareStreamConfig`
- `HardwareLifecycleContract`
- `BackendPolicyRecord`
- `HardwareDiagnosticsSnapshot`
- `EffectiveRuntimeConfig`
- `RuntimeDiagnosticsSnapshot`
- `RuntimeHostHardwareSummary`
- `RuntimeHostClockingSummary`
- `RuntimeHostLatencySummary`
- `RuntimeHostAudioPumpSummary`
- `RuntimeHostIoSummary`
- `RuntimeHostObservationReport`
- `RuntimeHostSupervisorReport`
- `RuntimeSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`

Batch 9.1 does not claim these anchors already expose full ALSA, JACK, or
PipeWire breadth. It freezes how later Linux backend DTOs and proofs must
deepen from this shared surface family instead of inventing a second Linux-only
hardware shell.

## Shared vocabulary

### Linux backend identity

`Linux backend identity` means the runtime-owned classification of the active
or negotiated Linux audio backend as ALSA, JACK, PipeWire, or explicitly
unsupported or unavailable.

Backend identity belongs on shared Signal-owned receipts. It must not require
consumers to infer the answer from backend crate names, native library
handles, or host-private callback threads.

### Backend portability

`backend portability` means whether Linux hardware capability, lifecycle, and
diagnostic meaning can be consumed through one Signal-owned runtime vocabulary
across ALSA, JACK, and PipeWire.

Portability here is about shared semantics, not identical native behavior. The
contract allows backend-specific realization detail while requiring one shared
consumer answer for:

- backend identity
- negotiated stream ownership
- restart policy and supervision posture
- diagnostic and fallback delivery

### Backend capability band

`backend capability band` means the bounded shared classification of Linux
backend behavior:

- `portable`: one shared Signal-owned interpretation exists
- `guarded`: shared meaning exists, but consumers must inspect runtime-owned
  policy, fallback, or degraded state
- `unsupported`: explicitly outside the current Linux backend surface

Batch 9.1 freezes the meaning of those bands for Linux audio backends. It does
not yet require all three Linux backends to be fully implemented.

### Backend lifecycle ownership

`backend lifecycle ownership` means the runtime-owned interpretation of who
controls stream start, stop, restart, and interruption exposure for the active
Linux backend path.

Hosts may broker native callbacks or server connections, but the shared
consumer answer must still land through the existing supervision,
interruption, and host-I/O surface family.

### Backend policy guard

`backend policy guard` means a backend uses the shared Linux hardware
vocabulary, but portability still depends on runtime-owned guarded answers such
as:

- simulated versus native operation
- degraded or recovering hardware state
- backend-specific stream availability or duplex fallback
- unsupported feature or unavailable transport conclusions

Guarded does not create a new Linux-only taxonomy. It reuses the existing
runtime-owned health, supervision, and clocking families.

## Current Linux backend matrix

This contract freezes the first bounded Linux hardware backend matrix.

| Capability family | ALSA | JACK | PipeWire | Notes |
| --- | --- | --- | --- | --- |
| Backend identity through shared runtime-owned receipts | guarded | guarded | guarded | The contract freezes one Linux backend vocabulary before all three baselines exist |
| Negotiated stream contract and runtime-owned hardware summaries | guarded | guarded | guarded | Must widen through the existing `signal-hardware` and runtime hardware contract |
| Restart, supervision, and fault-boundary meaning | guarded | guarded | guarded | Must reuse `025` instead of Linux-private restart models |
| Clocking, duplex, and endpoint-topology interpretation | guarded | guarded | guarded | Must compose with `026` rather than redefine it here |
| Backend-native extension or session detail | private | private | private | Remains backend-specific until later promotion |
| Non-Linux backend claims | unsupported | unsupported | unsupported | Outside this Linux backend queue |

The matrix is intentionally guarded-first. Batch 9.1 freezes the shared Linux
meaning before later implementation proves how much of that guarded breadth is
already realized.

## Rules

### Rule 1: Linux backend breadth reuses the existing hardware contract

ALSA, JACK, and PipeWire portability must widen the existing Signal-owned
hardware and clock-domain contract from `006`, not create a second Linux-only
hardware model.

### Rule 2: supervision and restart stay shared

Linux backend restart, recovering, exhaustion, and fault-boundary meaning must
reuse `025`. Backend-native reconnect loops or daemon reconnect semantics do
not become a competing consumer taxonomy.

### Rule 3: clocking and endpoint interpretation compose instead of fork

If ALSA, JACK, or PipeWire differ in duplex, clocking, or endpoint topology,
the shared consumer answer must still compose through `026` rather than a
backend-private Linux endpoint model.

### Rule 4: backend-native detail remains private until promoted

The following remain backend-private until later batches promote them into
shared DTOs:

- ALSA-specific device or PCM node detail
- JACK client, graph, or callback-thread detail
- PipeWire node, session, portal, or graph detail
- distro-specific daemon policy or setup conventions

Consumers must not depend on those details for reusable Linux backend
portability claims.

### Rule 5: guarded Linux support must stay typed

If a backend is only partially available, simulated, degraded, or unsupported,
that answer must land through shared runtime-owned hardware and supervision
receipts instead of host-private Linux capability tables.

## Deferred scope

Batch 9.1 intentionally does not claim:

- full ALSA, JACK, and PipeWire implementation parity
- distro certification or packaging breadth
- network-audio or distributed Linux session behavior
- control-surface or external MIDI backend policy
- richer daemon, graph, or portal semantics specific to one Linux backend

Those belong to later `g07.009`, `g07.010`, and broader hardware milestones.

## Batch 9.1 outcome

Batch 9.1 freezes the first bounded Linux audio backend portability contract:

- ALSA, JACK, and PipeWire now share one explicit Signal-owned portability and
  lifecycle vocabulary instead of being treated as future backend-private
  implementation detail
- Linux backend restart, supervision, clocking, and endpoint interpretation
  are now explicitly required to compose through the existing shared hardware,
  supervision, and clock-domain contracts
- Batch 9.2 can now materialize backend baselines against one bounded contract
  instead of reopening Linux backend ownership

## Batch 9.2 outcome

Batch 9.2 materializes the first shared Linux backend baseline against this
contract:

- `signal-hardware` now carries typed backend identity through
  `HardwareBackendIdentity` and `LinuxAudioBackendKind`
- ALSA, JACK, and PipeWire now have simulated baseline backends with distinct
  lifecycle and clock posture so backend differences land through one bounded
  hardware contract instead of backend-private host glue
- `signal-runtime` now exports runtime-owned Linux backend identity and
  portability-band answers through `RuntimeHostHardwareSummary` and
  `RuntimeExternalIoSnapshot`
- non-Linux hardware paths remain explicit as `NotLinux` and `Unsupported`
  instead of being ambiguously outside the Linux contract

Batch 9.2 still does not claim live ALSA, JACK, or PipeWire host ownership.
It establishes the first typed Linux backend baseline and shared diagnostic
classification that later proof work can consume directly.

## Batch 9.3 outcome

Batch 9.3 closes the bounded consumer seam for Linux audio backend
portability:

- public runtime proof now shows ALSA, JACK, PipeWire, and unavailable Linux
  backend identity plus portability-band truth through the shared external-I/O
  receipt family
- the stable Linux-facing server host edge now proves explicit unavailable
  Linux backend and fallback export instead of host-local Linux capability
  heuristics
- `signal-supervisor-tools` now exposes a machine-readable
  `signal.runtime.linux-audio-backend-boundary` descriptor and repo-owned
  acceptance seam

This contract is now closed for the bounded backend portability question. Live
backend-native clocking, duplex, and endpoint-topology parity remains the next
Linux queue rather than hidden scope inside this contract.

## Next Task

Continue `g07.010` with Batch 10.3 by adding focused proofs that the widened
Linux backend clocking, duplex, and endpoint-topology parity receipts remain
consumable through shared runtime, supervisor, and stable host-edge surfaces
without backend-private Linux capability matrices.
