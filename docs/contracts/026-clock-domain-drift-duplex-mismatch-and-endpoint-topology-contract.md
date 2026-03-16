# 026 Clock-Domain Drift, Duplex Mismatch, And Endpoint-Topology Contract

Status: complete
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`, `docs/architecture/package-map.md`

## Purpose

Freeze the first runtime-owned contract for clock drift, discontinuity, duplex
mismatch, and endpoint topology so later hardware, monitoring, loopback, and
external-I/O work can deepen one shared topology boundary instead of
reintroducing backend-local drift heuristics, host-only endpoint models, or
ad hoc fault labels.

## Authority hierarchy

Clock drift, duplex mismatch, and endpoint-topology meaning have one authority
chain:

1. `signal-hardware` owns backend-neutral capability, negotiated stream
   contract, and diagnostic evidence for:
   - device identity and negotiated streams
   - clock topology hints and clock source
   - lifecycle ownership, restart policy, and backend health
   - diagnostic counters, device loss, and restart evidence
2. `signal-runtime` owns canonical live-path meaning for:
   - runtime processing sample rate and applied hardware contract
   - clock-domain crossing and fallback state
   - drift, discontinuity, duplex mismatch, and endpoint-topology
     classification
   - aligned interruption, degradation, supervision, and fault-boundary
     interpretation
3. host crates may broker backend callbacks and negotiated-path evidence into
   runtime-owned summaries, but they must not become the authority for:
   - when drift is consumer-visible
   - when a discontinuity is only advisory versus continuity-breaking
   - whether input and output are duplex-aligned, split, partial, or degraded
   - endpoint-topology meaning outside the shared runtime contract

If a drift, mismatch, or topology claim cannot be explained through
`signal-hardware`, `signal-runtime`, and additive shared host receipts, it is
not yet part of the reusable Signal boundary.

## Existing runtime anchors

This contract is grounded in the existing live hardware and supervision surface
family:

- `HardwareStreamConfig`
- `HardwareClockTopology`
- `HardwareDiagnosticsSnapshot`
- `HardwareLifecycleContract`
- `EffectiveRuntimeConfig`
- `RuntimeDiagnosticsSnapshot`
- `RuntimeHostClockingSummary`
- `RuntimeHostHardwareSummary`
- `RuntimeHostIoSummary`
- `RuntimeHostObservationReport`
- `RuntimeHostSupervisorReport`
- `RuntimeDeviceSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`

Batch 15.1 does not claim these anchors already export full drift or topology
depth. It freezes how later DTOs and proofs must deepen from this shared
surface family.

## Shared vocabulary

### Clock drift

`clock drift` means the runtime-owned observation that active pacing
authorities are diverging over time strongly enough to matter to continuity,
resampling, duplex alignment, or endpoint interpretation.

It is not raw backend jitter, one callback delta, or a backend-private
timestamp delta in isolation. Those remain evidence until Signal promotes them
into shared runtime receipts.

### Discontinuity

`discontinuity` means a bounded break in expected clock or endpoint continuity
for the active live path.

Examples include:

- device-loss or restart-induced pacing breaks
- sudden clock-domain transition or reconfiguration
- endpoint removal or mutation that invalidates the previously active path

Discontinuity must stay aligned with supervision and interruption state instead
of becoming a separate host-local incident taxonomy.

### Duplex mismatch

`duplex mismatch` means the active input and output sides no longer share the
same practical live-path contract even if both are present.

This can include:

- different pacing authorities
- different sample-rate or buffer assumptions
- asymmetric channel or endpoint availability
- one direction remaining available while the other is degraded, absent, or
  forced through a different crossing path

Duplex mismatch is broader than “cross-clock.” It answers whether the
consumer-visible duplex path is still coherent as one paired live contract.

### Endpoint topology

`endpoint topology` means the runtime-owned shape of the active live hardware
path:

- which directions are present
- whether the path is single-endpoint, split, aggregate, or partial
- whether the active topology still matches the negotiated expectation
- whether topology change is steady, degraded, or continuity-breaking

Endpoint topology is the shared shape consumers depend on. Backend-native
device lists or host-private graph reconstruction do not replace it.

### Partial availability

`partial availability` means some portion of the negotiated hardware path
remains usable while another portion is missing, degraded, or waiting on
recovery.

This term is intentionally topology-oriented. It does not by itself answer
whether the remaining path is healthy, recovering, or faulted; that still comes
from the supervision boundary in `025`.

### Resync

`resync` means the runtime-owned process of re-establishing coherent live-path
timing or topology after drift, discontinuity, or a topology mutation has been
observed.

Resync may stay invisible when it is bounded and non-disruptive, but if later
receipts expose it, they must do so as runtime-owned continuity meaning rather
than a host-only callback story.

## Rules

### Rule 1: drift stays runtime-owned, not backend-private

Hosts and tools must not require backend-native timestamps, callback cadence
heuristics, or adapter-private resampler state to explain whether drift is
consumer-visible.

Backend evidence may contribute to runtime classification, but the shared
consumer answer must come from runtime-owned receipts.

### Rule 2: discontinuity composes with supervision

Discontinuity and restart or exhaustion meaning must stay aligned with the
device-supervision contract from `025`.

Later batches may add richer drift or topology DTOs, but they must not invent a
new hardware fault boundary separate from:

- `RuntimeDeviceSupervisionSnapshot`
- `RuntimeFaultStatusSnapshot`
- `RuntimeInterruptionSummary`
- `RuntimeDegradationSummary`

### Rule 3: duplex mismatch must be explicit

Consumers must not have to infer duplex mismatch from separate input and output
sample-rate fields, channel counts, or backend-private device identities
alone.

If later runtime receipts expose the live duplex path, they must preserve a
direct way to tell whether the duplex contract is:

- aligned
- cross-clock but still coherent
- partially available
- degraded or continuity-breaking

### Rule 4: endpoint topology must stay host-neutral

Endpoint topology meaning belongs on Signal-owned runtime and shared host
surfaces, not product-local device pickers or backend-specific endpoint graphs.

Later work may add richer endpoint members, roles, or mutations, but those
details become reusable only when promoted into typed Signal receipts.

### Rule 5: partial availability is not silent success

If one direction, endpoint, or member of the negotiated path drops out,
consumers must not be forced to treat the remaining path as unchanged.

Later runtime depth must preserve enough shared meaning to distinguish:

- fully steady topology
- partial but recoverable availability
- partial availability that has crossed into a faulted or exhausted episode

### Rule 6: later external-I/O work must deepen this contract, not replace it

`g06.016` and later monitoring or loopback milestones may widen endpoint,
tap-point, loopback, and duplex detail, but they must build on this contract
instead of reopening drift ownership or topology semantics.

## Deferred scope

Batch 15.1 intentionally keeps the following outside the shared contract:

- network-audio or distributed clock synchronization
- backend-specific drift algorithms or timestamp math
- control-surface or external MIDI hardware policy
- exhaustive endpoint role metadata for monitoring and loopback paths
- user-facing device selection or recovery UX
- product-local routing views or endpoint naming conventions

These may later gain additive Signal-owned surfaces, but they are not promised
by this opening contract.

## Batch 15.1 outcome

Batch 15.1 freezes the first bounded clock-drift and endpoint-topology
contract:

- Signal now has one shared vocabulary for drift, discontinuity, duplex
  mismatch, endpoint topology, partial availability, and resync
- the authority line is explicit: backend diagnostics and host callbacks remain
  evidence, while runtime-owned receipts stay canonical for consumer meaning
- the new contract composes directly with `g06.014` device supervision instead
  of creating a parallel hardware fault model
- later `g06.015` runtime DTOs and proofs can now deepen drift and topology
  observation against one fixed contract before external-I/O and monitoring
  work widens further

## Batch 15.2 outcome

Batch 15.2 materializes the first runtime-owned receipt depth on top of this
contract:

- `RuntimeHostClockingSummary` now carries explicit:
  - drift-state classification
  - discontinuity-state classification
  - duplex-mismatch classification
  - endpoint-topology classification
  - partial-availability visibility
- `RuntimeExternalIoSnapshot` now preserves the same bounded meaning instead of
  collapsing back to fallback-only or device-change-only health
- `signal-host-local` now derives those fields in one place from the active
  stream contract, backend health, transition state, and stream state
- the stable host-edge and embedded supervisor observation now reuse the same
  `host_io` receipt for a given report emission instead of recomputing
  divergent first-observation transitions
- focused runtime and local-host proofs now cover:
  - output-only same-clock steady state
  - cross-clock fallback with explicit drift and discontinuity export
  - aggregate-clock topology export
  - duplex cross-clock mismatch
  - duplex partial availability
  - faulted recovery-constrained hardware state
  - return-to-direct continuity after cross-clock fallback

## Batch 15.3 outcome

Batch 15.3 closes the bounded consumer proof seam for this contract:

- the public runtime boundary now proves drift, duplex-mismatch, and
  endpoint-topology truth through `RuntimeHostObservationReport` and
  `RuntimeHostSupervisorReport`
- the stable local host edge now proves live host-I/O clocking export without
  private host derivation helpers
- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.clock-topology-boundary` descriptor and the repo-owned task
  `effigy acceptance:clock-topology-boundary --repo .`
- richer duplex cross-clock and partial-availability cases stay on the same
  shared proof spine through focused local-host tests, while the stable server
  host edge still does not expose live host-I/O receipts directly

## Next Task

Continue `g06.016` with Batch 16.1 by freezing the external-I/O, monitoring
tap-point, and loopback measurement contract on top of this closed clocking
and endpoint-topology boundary.
