# 006 Runtime Hardware Portability And Clock-Domain Contract

Status: active
Owner: core-product
Updated: 2026-03-12
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/004-runtime-multicore-scheduling-and-anticipative-execution-contract.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned contract for backend capability,
negotiated hardware state, and clock-domain meaning so later `g04.004` runtime
and host work can deepen backend portability without pushing device or
resampling policy back into host-local code.

## Authority hierarchy

Hardware portability and clock-domain behavior have one authority chain:

1. `signal-hardware` owns backend capability and negotiated stream primitives.
   Today that means:
   - `AudioDeviceDescriptor`
   - `HardwareStreamRequest`
   - `HardwareStreamConfig`
   - `HardwareLatencyProfile`
   - `HardwareLifecycleContract`
   - `BackendPolicyRecord`
   - `HardwareDiagnosticsSnapshot`
2. `signal-runtime` owns the active processing contract after hardware
   negotiation is applied. Today that means:
   - `HardwareConfigRequest`
   - `EffectiveRuntimeConfig`
   - `RuntimeDiagnosticsSnapshot`
   - runtime-owned resampling and offline/export sample-rate handling
3. `RuntimeHostHardwareSummary`, `RuntimeHostClockingSummary`,
   `RuntimeHostLatencySummary`, `RuntimeHostAudioPumpSummary`, and
   `RuntimeHostIoSummary` are the canonical shared observation/export receipts
   for the negotiated live host path.
4. `RuntimeHostObservationReport` and `RuntimeHostSupervisorReport` deliver
   that host-augmented state to consumers. `RuntimeObservationReport` and
   `RuntimeSupervisorReport` remain the base runtime authority when host I/O
   detail is not available or not relevant.

If later backend or clocking detail matters to consumers, it should be promoted
into the runtime-owned summaries above rather than inferred from backend-native
handles, callback threads, or app-local device state.

## Capability model

The first portability contract divides hardware state into three layers.

### Backend capability

Backends own capability discovery and negotiation inputs:

- backend identity through `backend_name`
- policy placement through `BackendPolicyTier` and `BackendPolicyRecord`
- device identity and defaults through `AudioDeviceDescriptor`
- capability ceilings such as channel counts, nominal sample rate, and
  preferred buffer sizes

This layer answers what a backend can offer. It does not answer what the active
runtime path is currently using.

### Negotiated stream contract

Negotiation produces the concrete stream contract that runtime may apply:

- direction, device id, sample rate, and buffer size
- negotiated input/output channel counts
- sample format and interleaving
- declared clock source
- lifecycle ownership and restart policy
- base latency profile
- whether the stream is simulated

The negotiated stream contract is the only supported bridge from backend
capability into active runtime hardware state. Hosts must not apply sideband
device changes that bypass this contract.

### Active runtime/host observation

Once applied, the active path that consumers inspect is carried through
runtime-owned reports:

- runtime processing configuration through `EffectiveRuntimeConfig`
- backend policy and runtime diagnostics through `RuntimeDiagnosticsSnapshot`
- negotiated hardware, clocking, latency, and pump state through
  `RuntimeHostIoSummary`
- delivery through `RuntimeHostObservationReport` and
  `RuntimeHostSupervisorReport`

This layer answers what the live path is doing now, not what the backend could
theoretically offer.

## Clock-domain model

The first portability contract uses four semantic states.

### Same-clock

A path is same-clock when one negotiated hardware clock directly paces the live
runtime path and no additional crossing is required between the runtime
processing rate and the active hardware stream.

Today consumers can recognize the same-clock case from:

- one active hardware stream in `RuntimeHostIoSummary`
- `RuntimeHostHardwareSummary.sample_rate` aligned with
  `EffectiveRuntimeConfig.sample_rate`
- a single declared `RuntimeHostClockSource`
- no need for host-local resampling or drift compensation

### Cross-clock

A path is cross-clock when audio or timing must move between different sample
rates or pacing authorities.

Under this contract:

- resampling and timing conversion stay Signal-owned
- backend adapters may negotiate raw stream rates, but they must not silently
  hide rate conversion as backend-private magic
- hosts may request or observe the path, but they must not become the authority
  for sample-rate conversion policy

Current offline render/export resampling already follows this rule inside
`signal-runtime`. Live multi-clock receipts are not fully typed yet, so later
`g04.004` implementation work must add those receipts instead of teaching
consumers to infer crossings from backend-specific state.

### Aggregate

A path is aggregate when one logical Signal I/O path spans multiple devices,
substreams, or pacing authorities.

This state is intentionally not public by implication. Backend-private names or
device lists are not enough. Aggregate behavior becomes consumer-visible only
when Signal adds typed runtime-owned receipts for:

- the aggregate/follower relationship
- the authoritative pacing source
- any runtime-owned crossing or compensation applied between members

Until then, aggregate composition remains backend-private detail.

### Degraded

A path is degraded when hardware or callback health means the negotiated clock
or stream contract is no longer fully healthy, even if some audio service still
continues.

Today consumers should treat the path as degraded when runtime-owned receipts
show evidence such as:

- `BackendHealth::Degraded` or `BackendHealth::Recovering`
- device-loss, restart-attempt, or restart-failure growth
- xrun or callback-overrun growth that changes health interpretation
- paired runtime degradation or safe-mode state that constrains hardware work

Degraded hardware state must remain visible through runtime-owned receipts. It
must not be encoded only in host logs or backend-private diagnostics.

## Host-neutral export versus backend-private detail

### Host-neutral export today

The following are already acceptable reusable consumer-facing surfaces:

- runtime processing configuration and diagnostics:
  - `EffectiveRuntimeConfig`
  - `RuntimeDiagnosticsSnapshot`
- host-augmented hardware observation:
  - `RuntimeHostHardwareSummary`
  - `RuntimeHostClockingSummary`
  - `RuntimeHostLatencySummary`
  - `RuntimeHostAudioPumpSummary`
  - `RuntimeHostIoSummary`
  - `RuntimeHostObservationReport`
  - `RuntimeHostSupervisorReport`
- profiling and soak receipts that aggregate host/runtime counters without
  replacing those authority surfaces

These surfaces are host-neutral because they describe the Signal contract for
the live path rather than one backend's native API structure.

### Backend-private detail for now

The following remain internal until Signal promotes them into typed
runtime-owned receipts:

- backend-native device handles, callback-thread layout, and stream objects
- platform error codes or backend-specific diagnostic payloads
- raw device-enumeration breadth beyond the current negotiated contract
- aggregate-device membership, leader/follower topology, or drift estimates
- backend-local resampling, drift correction, or fallback rules that are not
  surfaced through Signal-owned runtime DTOs

Consumers may depend on the public runtime/host reports, not on backend crate
internals.

## Portability and fallback rules

- backends negotiate capabilities and stream contracts; runtime owns active
  processing configuration and clock-boundary semantics
- sample-rate conversion belongs in Signal-owned DSP/runtime paths, not in
  hidden backend adapters
- degraded hardware state must stay aligned with the scheduler and deferred
  work contracts instead of creating a separate host-only recovery model
- if a later backend needs richer fallback or aggregate-clock detail, that
  detail must appear as additive runtime-owned receipts before consumers depend
  on it

## Canonical inspection surfaces

Consumers should inspect portability state in this order:

- use `signal-hardware` negotiation types when the question is capability
  discovery or stream negotiation before runtime activation
- use `RuntimeDiagnosticsSnapshot` and `EffectiveRuntimeConfig` when the
  question is what runtime accepted as the active processing contract
- use `RuntimeHostIoSummary`, `RuntimeHostObservationReport`, and
  `RuntimeHostSupervisorReport` when the question is live hardware, clocking,
  latency, and pump behavior
- use `RuntimeProfilingReceipt` and `RuntimeSoakReceipt` when the question is
  comparative health or long-running behavior rather than direct authority over
  hardware state

Hosts and tools may format or aggregate these surfaces, but they must not build
their own portability model from private backend state when the typed Signal
surfaces already answer the question.

## Current proof boundary

The contract is grounded in implementation that already exists:

- `signal-hardware` and `signal-hardware-coreaudio` expose backend-neutral
  device, stream, lifecycle, clock-source, clock-topology, latency, and
  diagnostic primitives
- `signal-runtime` accepts negotiated hardware through `HardwareConfigRequest`
  and tracks backend policy in runtime diagnostics
- `signal-host-local` already projects negotiated hardware, clocking, latency,
  and callback-pump state into `RuntimeHostIoSummary`
- `RuntimeHostClockingSummary` now carries explicit:
  - processing versus hardware sample-rate visibility
  - `clock_domain` classification
  - `fallback_state` classification
  - `transition_state` classification
  - `crossing_required` visibility
- focused host proofs already cover:
  - negotiated hardware contract export at boot
  - same-clock direct host observation/supervisor export
  - cross-clock runtime-resampled host export when processing and hardware
    sample rates differ
  - aggregate-clock host export through the same runtime-owned receipt family
  - degraded recovery-constrained host export during device-loss restart
    failure
  - transition-aware host export that distinguishes first observation,
    cross-clock entry, aggregate-clock entry, and return-to-direct recovery
- offline render/export resampling already stays runtime-owned rather than
  backend-private

`g04.004` closes with that receipt family in place. Residual breadth is now
explicitly deferred rather than left ambiguous: multi-member aggregate detail,
clock drift compensation, and broader backend-matrix coverage still belong to
later work unless consumers actually need them.

## Next Task

Continue `g04.005` with Batch 5.2 and deepen the typed plugin backend and
host-neutral delegation surfaces on top of the now-closed runtime, deferred
work, and hardware portability boundaries.
