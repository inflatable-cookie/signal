# 027 External-I/O, Monitoring Tap-Point, And Loopback Measurement Contract

Status: active
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`, `docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first runtime-owned contract for external-I/O roles, monitoring tap
points, and loopback measurement so later hardware, calibration, and
media-service work can deepen one reusable boundary instead of reintroducing
host-local monitor routing, ad hoc loopback semantics, or product-specific
measurement heuristics.

## Authority hierarchy

External-I/O, monitoring, and loopback meaning have one authority chain:

1. `signal-hardware` owns backend-neutral capability and negotiated-path
   evidence for:
   - device identity, stream direction, and negotiated endpoint membership
   - clock topology, clock source, restart policy, and backend health
   - backend diagnostics and device-loss evidence relevant to the active path
2. `signal-runtime` owns canonical consumer-visible meaning for:
   - which active path is `program`, `monitor`, `external-input`, `external-output`,
     or `loopback-observed`
   - where a monitor or loopback tap point sits relative to runtime output,
     hardware output, and external return paths
   - whether loopback or monitor state is unavailable, direct, guarded,
     degraded, or continuity-breaking
   - how monitoring and measurement meaning composes with the closed
     `g06.014` supervision boundary and the closed `g06.015` clock-domain and
     endpoint-topology boundary
3. host crates may broker backend callbacks, negotiated endpoints, and bounded
   measurement evidence into runtime-owned summaries, but they must not become
   the authority for:
   - monitor-path role naming
   - where tap points are consumer-visible
   - what qualifies as loopback-ready versus unavailable or faulted
   - competing measurement taxonomies outside the runtime contract

If an external-I/O, monitor, or loopback claim cannot be explained through
`signal-hardware`, `signal-runtime`, and additive shared host receipts, it is
not yet part of the reusable Signal boundary.

## Existing runtime anchors

This contract is grounded in the current live hardware and supervision surface
family:

- `HardwareStreamConfig`
- `HardwareDiagnosticsSnapshot`
- `HardwareLifecycleContract`
- `RuntimeHostIoSummary`
- `RuntimeHostClockingSummary`
- `RuntimeHostLatencySummary`
- `RuntimeExternalIoSnapshot`
- `RuntimeHostObservationReport`
- `RuntimeHostSupervisorReport`
- `RuntimeDeviceSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`

Batch 16.1 does not claim these anchors already export full monitoring or
loopback depth. It freezes how later DTOs, receipts, and proofs must widen
from this shared surface family.

## Shared vocabulary

### External I/O

`external I/O` means runtime-owned meaning for hardware-facing input, output,
or duplex paths that exist outside the in-graph processing core but still
affect consumer-visible monitoring, routing, or measurement behavior.

It is not a raw backend device list, channel-map UI model, or host-private
stream handle inventory.

### Monitoring path

`monitoring path` means the runtime-owned consumer-visible path that exposes a
live signal for confidence, audition, cueing, or hardware verification.

A monitoring path may share the main program output, branch from an external
return, or observe a bounded loopback path, but the shared answer for
consumers must remain runtime-owned rather than product-local routing prose.

### Tap point

`tap point` means the explicit runtime-owned observation position where Signal
is claiming monitor or measurement truth.

Examples include:

- post-runtime program output before hardware submission
- post-hardware output observation
- post-external-input observation before graph ingestion
- loopback return observation after a hardware round trip

Tap points must stay bounded and typed. A consumer should not need to infer
them from callback ordering, backend timestamps, or host-private audio graph
knowledge.

### Loopback

`loopback` means a runtime-owned observation path that compares or observes
Signal output after some external round trip or hardware-return boundary.

Loopback does not require this batch to promise full calibration or automatic
alignment. It only freezes the shared meaning that such a path exists, where
it is tapped, and whether it is available, guarded, degraded, or unavailable.

### Measurement session

`measurement session` means one bounded runtime-owned attempt to observe or
summarize monitor or loopback behavior for a consumer-facing purpose.

A measurement session may later include calibration, latency estimation, or
confidence checks, but this batch freezes only the shared session meaning and
authority chain, not full algorithm depth.

### Reference path

`reference path` means the runtime-owned baseline signal or endpoint contract a
monitor or loopback observation is supposed to be compared against.

Reference-path meaning must stay aligned with the closed `g06.015`
clock-domain, endpoint-topology, and duplex-mismatch boundary instead of being
redefined per host or per product.

## Rules

### Rule 1: runtime owns monitor and tap-point meaning

Products and hosts must not rely on backend-private graph placement,
device-specific callback order, or product-local routing names to decide what
Signal means by a monitor or loopback tap point.

If later runtime receipts expose a monitor or loopback path, the consumer
meaning must come from typed runtime-owned surfaces.

### Rule 2: loopback composes with topology and supervision

Loopback and monitor availability must compose with the closed hardware
boundaries from `025` and `026`.

Later DTOs may widen monitor or measurement detail, but they must not invent a
parallel restart, drift, or endpoint-topology taxonomy separate from:

- `RuntimeDeviceSupervisionSnapshot`
- `RuntimeExternalIoSnapshot`
- `RuntimeHostClockingSummary`
- `RuntimeFaultStatusSnapshot`

### Rule 3: runtime-facing versus supervisor-facing detail must stay explicit

This milestone intentionally freezes that some monitoring and loopback meaning
belongs on runtime-facing DTOs while other detail may only belong on broader
supervisor export.

Later batches must preserve a direct distinction between:

- bounded runtime-owned status needed for shared host edges and public runtime
  consumers
- additive supervisor/export detail that is useful for diagnostics or tooling
  but not required for every embedded consumer

### Rule 4: measurement receipts must stay bounded and typed

Consumers must not be forced to scrape logs or reinterpret raw callback deltas
to tell whether a monitor or loopback path is absent, direct, guarded,
recovering, degraded, or faulted.

Later measurement receipts may remain intentionally coarse, but they must stay
typed and Signal-owned.

### Rule 5: later calibration work must deepen this contract, not replace it

`g06.016` and later hardware or media-service milestones may widen loopback,
calibration, and monitor-depth behavior, but they must build on this contract
instead of reopening monitor ownership or moving measurement truth into
consumer-local code.

## Deferred scope

Batch 16.1 intentionally keeps the following outside the shared contract:

- full cue-mix, bus-matrix, or product-specific monitor routing UX
- room correction, speaker calibration, or automatic acoustic analysis policy
- network-audio or remote monitor-path semantics
- control-surface policy and device selection UX
- exhaustive endpoint naming, grouping, or visualization metadata
- lossless latency-compensation or alignment algorithms for loopback analysis

These may later gain additive Signal-owned surfaces, but they are not promised
by Batch 16.1.

## Batch 16.1 outcome

Batch 16.1 freezes the first bounded monitoring and loopback contract:

- Signal now has one shared vocabulary for external-I/O roles, monitor tap
  points, loopback paths, measurement sessions, and reference paths
- the authority line is explicit: hardware and backend facts remain evidence,
  while runtime-owned receipts stay canonical for consumer meaning
- monitor and loopback semantics are now explicitly composed with the closed
  `g06.014` supervision boundary and `g06.015` clock-domain and
  endpoint-topology boundary
- later `g06.016` runtime DTOs and proofs can now deepen monitoring and
  loopback observation against one fixed contract before media-service and
  calibration work widens further

## Batch 16.2 outcome

Batch 16.2 materializes the first runtime-owned receipt depth on top of this
contract:

- `RuntimeExternalIoSnapshot` now carries explicit:
  - external-I/O health
  - device-change state
  - primary role
  - monitoring state
  - monitoring tap point
  - loopback state
- runtime observation surfaces now export the shared external-I/O snapshot
  directly instead of leaving monitor-path meaning implicit in broader host-I/O
  prose
- `signal-host-local` feeds live host-I/O facts into the same runtime-owned
  receipt family
- `signal-host-server` stays aligned to the shared contract by exporting the
  same snapshot shape with explicit `Unavailable` states when no live host-I/O
  monitoring seam exists
- monitor and loopback meaning now composes directly with the already-closed
  supervision and clock-topology contracts instead of being reconstructed from
  backend-local state

This batch still stops short of the downstream proof boundary. Batch 16.3 is
responsible for proving that these widened receipts stay consumable through
shared runtime, supervisor, and stable host-edge surfaces without local
monitor-model reconstruction.

## Batch 16.3 outcome

Batch 16.3 closes the first reusable consumer proof seam for this contract:

- downstream-style runtime proofs now consume external-I/O role, monitor
  state, tap-point, and loopback meaning directly from public runtime report
  surfaces
- the stable local host edge proves the same runtime-owned receipt family
  forwards direct and explicit faulted external-I/O truth without local monitor
  reconstruction
- the stable server host edge proves the same receipt family stays consumable
  even where the host only exports explicit `Unavailable` monitoring and
  loopback state
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.external-io-boundary` descriptor
- the repo-owned `effigy acceptance:external-io-boundary --repo .` task keeps
  the proof seam runnable instead of prose-only

The closed boundary remains intentionally bounded. Richer measurement-session,
calibration, waveform, and preview-service workflows are still deferred to
later work built on top of this shared receipt family.

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of the closed
media-service boundary.
