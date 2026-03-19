# 041 Linux Backend Clocking, Duplex, And Endpoint-Topology Parity Contract

Status: complete
Owner: core-product
Updated: 2026-03-18
Related contracts: `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`, `docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`, `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the bounded Linux-specific clocking, duplex, and endpoint-topology
parity boundary for `g07.010` so later ALSA, JACK, and PipeWire runtime work
can deepen one shared hardware vocabulary without reopening backend-private
clock stories, daemon-local endpoint graphs, or host-only duplex heuristics.

## Authority hierarchy

Linux backend clocking and topology parity have one authority chain:

1. `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`
   remains the authority for Linux backend identity, portability band,
   lifecycle ownership, and additive backend fallback posture
2. `signal-hardware` owns backend-neutral capability, negotiated stream
   contract, and backend evidence for:
   - clock source and pacing hints
   - duplex availability and negotiated directionality
   - endpoint membership and stream-shape evidence
   - backend diagnostics that may later contribute to parity classification
3. `docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`
   remains the authority for consumer-visible drift, discontinuity, duplex
   mismatch, endpoint topology, partial availability, and resync meaning
4. `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`
   remains the authority for recovering, exhausted, and faulted hardware state
   when Linux backend clocking or topology crosses from guarded parity into
   supervision-visible failure
5. `signal-runtime` must own the canonical Linux parity interpretation for:
   - ALSA, JACK, and PipeWire clocking posture
   - duplex alignment and guarded mismatch conclusions
   - endpoint-topology class and topology mutation visibility
   - observation, supervisor, and acceptance export delivery
6. host crates may broker backend callbacks, server notifications, graph
   changes, or negotiated-path detail into runtime-owned receipts, but they
   must not become the authority for:
   - backend-private clock drift or duplex taxonomies
   - daemon-specific endpoint-graph truth as a consumer contract
   - Linux-only unsupported-state tables detached from shared runtime receipts

If a Linux clocking, duplex, or endpoint-topology parity claim cannot be
explained through `040`, `026`, `025`, and runtime-owned hardware receipts, it
is not yet part of the reusable Signal contract.

## Existing anchors

Batch 10.1 freezes this contract on top of the current shared hardware and
clock-domain surface family:

- `HardwareStreamConfig`
- `HardwareClockTopology`
- `HardwareDiagnosticsSnapshot`
- `HardwareLifecycleContract`
- `EffectiveRuntimeConfig`
- `RuntimeHostHardwareSummary`
- `RuntimeHostClockingSummary`
- `RuntimeHostLatencySummary`
- `RuntimeHostIoSummary`
- `RuntimeHostObservationReport`
- `RuntimeHostSupervisorReport`
- `RuntimeExternalIoSnapshot`
- `RuntimeSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`

Batch 10.1 does not claim these anchors already expose full live ALSA, JACK,
or PipeWire host ownership. It freezes how later Linux backend DTOs and proofs
must deepen from this shared surface family instead of inventing a second
Linux-only clocking shell.

## Shared vocabulary

### Linux backend clocking parity

`Linux backend clocking parity` means whether ALSA, JACK, and PipeWire expose
consumer-visible clocking meaning through one Signal-owned runtime vocabulary
for:

- pacing authority and guarded drift posture
- discontinuity visibility
- same-clock versus cross-clock interpretation
- topology-aware clocking fallback state

Parity here does not require identical backend-native timing models. It
requires one shared consumer answer grounded in runtime-owned receipts.

### Linux duplex parity

`Linux duplex parity` means whether the active Linux input and output path can
be interpreted through one Signal-owned duplex vocabulary across ALSA, JACK,
and PipeWire.

Batch 10.1 freezes the bounded Linux duplex posture family:

- `aligned`
- `guarded`
- `partial`
- `unsupported`

These are parity classes, not new DTO enums. Later runtime work must express
them through the existing shared clocking, endpoint-topology, supervision, and
fallback receipts instead of a backend-private Linux duplex table.

### Linux endpoint-topology parity

`Linux endpoint-topology parity` means whether the active Linux path exposes
one shared Signal-owned answer for endpoint shape, split versus aggregate
behavior, and continuity-relevant topology mutation.

It is not a promise of identical native node graphs or identical backend
device naming. Endpoint-topology parity is about one runtime-owned consumer
shape across ALSA, JACK, and PipeWire.

### Backend-private behavior

`backend-private behavior` means Linux backend detail that may inform runtime
classification but is not yet part of the reusable Signal boundary.

Batch 10.1 keeps the following backend-private:

- ALSA-specific PCM or card-node detail
- JACK client, port-graph, or callback-thread detail
- PipeWire node, session-manager, portal, or graph detail
- daemon- or distro-specific reconnect and endpoint-naming policy

Consumers must not depend on those details for shared Linux parity claims.

### Unsupported parity

`unsupported parity` means the current Signal boundary does not yet claim a
portable Linux backend answer for a clocking, duplex, or endpoint-topology
question.

Unsupported parity must stay explicit through shared runtime-owned guarded or
unsupported outcomes. It must not become an implied gap that consumers
reconstruct from missing backend fields or host-local capability matrices.

## Linux parity matrix

This contract freezes the first bounded Linux backend clocking and topology
matrix.

| Capability family | ALSA | JACK | PipeWire | Notes |
| --- | --- | --- | --- | --- |
| Backend identity and portability band through shared runtime receipts | portable | portable | portable | Closed in `040` and reused here |
| Clocking posture through shared runtime-owned summaries | guarded | guarded | guarded | Must compose through `026` rather than backend-private timing stories |
| Duplex coherence and mismatch interpretation | guarded | guarded | guarded | Must stay in the shared drift and endpoint-topology family |
| Endpoint-topology class and mutation visibility | guarded | guarded | guarded | Shared runtime answer required even when native graph detail differs |
| Backend-native node, graph, or daemon detail | private | private | private | Still backend-private until later promotion |
| Non-Linux parity claims | unsupported | unsupported | unsupported | Outside this Linux queue |

The matrix is intentionally guarded-first. Batch 10.1 freezes one bounded
Linux parity target before later runtime work proves how much live backend
depth is already realized.

## Rules

### Rule 1: Linux clocking parity reuses the closed drift contract

ALSA, JACK, and PipeWire parity must deepen the existing drift, discontinuity,
duplex mismatch, and endpoint-topology boundary from `026`. This milestone
must not create a second Linux-only clocking taxonomy.

### Rule 2: backend identity stays separate from topology interpretation

`040` owns Linux backend identity and portability band. This contract layers
clocking, duplex, and topology parity on top of that identity rather than
blurring backend selection together with endpoint-topology meaning.

### Rule 3: supervision and fault posture remain shared

If Linux backend clocking or endpoint mutation becomes recovering, exhausted,
or faulted, the authoritative answer must still compose through `025` instead
of a backend-native reconnect or daemon incident model.

### Rule 4: backend-private graph detail stays advisory

Hosts may observe ALSA node detail, JACK graph churn, or PipeWire session
mutation, but shared consumers must not depend on those backend-private views
to determine whether Linux parity is steady, guarded, partial, or unsupported.

### Rule 5: unsupported Linux parity must stay typed

If one Linux backend cannot yet provide the same bounded clocking or topology
answer as another, the result must land through shared runtime-owned guarded
or unsupported receipts rather than host-local Linux capability matrices.

### Rule 6: later external-I/O and MIDI work must deepen this contract

Future Linux external-I/O, control-surface, or MIDI endpoint work may widen
backend-native details, but it must build on this contract instead of
reopening Linux clocking or endpoint ownership.

## Deferred scope

Batch 10.1 intentionally does not claim:

- live ALSA, JACK, and PipeWire host ownership parity
- daemon-specific graph, node, or session semantics
- network-audio or distributed Linux clock synchronization
- distro certification or packaging guarantees
- user-facing Linux device setup or repair UX
- control-surface, external MIDI, or broader endpoint-role expansion

Those belong to later `g07.010` batches and later Linux hardware milestones.

## Batch 10.1 outcome

Batch 10.1 freezes the bounded Linux backend clocking, duplex, and
endpoint-topology parity contract:

- ALSA, JACK, and PipeWire now have one explicit Linux-facing parity target
  for clocking, duplex, and endpoint topology instead of implicit future
  backend-private behavior
- Linux backend identity and portability remain anchored in `040`, while
  clocking and topology meaning are now explicitly required to compose through
  the shared runtime-owned drift and supervision contracts
- backend-private graph and daemon detail stays advisory until later
  promotion, which keeps Batch 10.2 focused on shared runtime receipts rather
  than backend-local narratives

## Batch 10.2 outcome

Batch 10.2 materializes the first runtime-owned Linux backend parity receipts
on top of this contract:

- `RuntimeHostClockingSummary` now carries explicit
  `linux_clocking_parity`, `linux_duplex_parity`, and
  `linux_endpoint_topology_parity` so Linux backend conclusions remain
  runtime-owned instead of product- or host-reconstructed
- `RuntimeExternalIoSnapshot` now preserves the same Linux parity answer
  alongside the existing generic clocking, duplex-mismatch, and endpoint
  topology fields so supervisor and host-edge consumers can rely on one
  shared Linux hardware vocabulary
- local-host and server-host shared reports now forward explicit unsupported
  Linux parity on current non-Linux and unavailable paths, while focused
  runtime tests cover portable ALSA-style and guarded JACK-style classification

Batch 10.2 still does not close the public proof seam. That is now the
remaining Batch 10.3 job rather than hidden runtime-shaping work.

## Batch 10.3 outcome

Batch 10.3 closes the bounded Linux backend clocking, duplex, and
endpoint-topology proof seam:

- public runtime proofs now show Linux-specific parity truth stays consumable
  through shared observation and supervisor surfaces for ALSA, JACK, PipeWire,
  non-Linux, and unavailable host contexts
- the stable local host edge now forwards explicit unsupported Linux parity on
  non-Linux hardware, and the stable server host edge forwards explicit
  unavailable Linux parity instead of rebuilding backend-private Linux
  capability matrices
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.linux-backend-clock-topology-boundary` descriptor, and Effigy
  now owns `acceptance:linux-backend-clock-topology-boundary` as the repo-owned
  consumer rerun lane

This contract is now closed for `g07.010`. Later Linux queues may deepen live
backend ownership, but they must build on this bounded parity seam rather than
reopening Linux clocking or topology meaning.

## Next Task

Continue `g07.012` with Batch 12.2 by materializing the first runtime-owned
MIDI 2.0, MPE, and richer controller-expression receipt family across runtime,
plugin, and hardware surfaces without reopening adapter-private packet
ownership.
