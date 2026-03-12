# 007 Plugin Backend And Host-Neutral Delegation Contract

Status: active
Owner: core-product
Updated: 2026-03-12
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`, `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned contract for format-neutral plugin
backend surfaces and host-neutral delegated execution receipts so later
`g04.005` work can widen backend breadth without pushing plugin lifecycle,
capability, or delegation meaning into adapter-local or consumer-local code.

## Authority hierarchy

Plugin backend and delegation behavior has one authority chain:

1. `signal-plugin` owns the format-neutral plugin vocabulary:
   - plugin identity, format, feature, and I/O description
   - parameter, state, processing, and lifecycle contracts
   - instance snapshot, readiness, and fault vocabulary
   - sandbox capability, transport, and state-machine abstractions
2. `signal-runtime` owns the runtime-side execution and observation contract
   built on top of those plugin concepts:
   - plugin-backed node binding projections
   - runtime plugin lifecycle, chain, recall, and compensation snapshots
   - runtime-owned delegated offline execution boundary, request, receipt, and
     outcome/merge surfaces
   - report/export delivery through `RuntimeObservationReport` and
     `RuntimeSupervisorReport`
3. adapter crates such as `signal-plugin-clap` own format-specific discovery,
   protocol, extension, and instance-control detail needed to realize one
   backend against the format-neutral substrate
4. host crates such as `signal-host-local` and `signal-host-server` may request
   scans, ensure sandboxes, or fulfill delegated execution, but they must not
   become the authority for plugin lifecycle semantics, capability meaning, or
   receipt schemas when typed Signal-owned surfaces already exist

If later consumers need richer backend detail, that detail should be promoted
into `signal-plugin` or `signal-runtime` rather than inferred from CLAP/VST/AU
adapter internals.

## Format-neutral plugin surfaces

The current reusable plugin boundary is anchored in `signal-plugin`.

### Descriptor and capability surfaces

These surfaces are format-neutral and reusable:

- `PluginFormat`
- `PluginDescriptor`
- `PluginFeature`
- `PluginAudioBusDescriptor`
- `PluginParameterDescriptor`
- `PluginParameterFlags`
- `PluginStateContract`
- `PluginProcessingContract`
- `PluginLifecycleContract`
- `PluginSandboxCapabilities`
- `SandboxTransport`

These types answer what a plugin backend can expose or require without binding
that meaning to one concrete format protocol.

### Instance and fault surfaces

These surfaces are also format-neutral and reusable:

- `PluginInstanceSnapshot`
- `PluginProcessConfiguration`
- `PluginLifecycleState`
- `PluginReadiness`
- `PluginFault`
- `PluginFaultKind`
- `PluginFaultSeverity`
- `SandboxStateMachine`

These types answer what state a realized plugin instance is in and how faults
or readiness should be described before runtime-specific export is layered on
top.

## Runtime-owned plugin execution and export surfaces

The runtime layer owns plugin state as it participates in graph execution and
offline delegation.

### Live runtime observation

The current runtime-owned authority surfaces are:

- `RuntimePluginDiscoverySnapshot`
- `RuntimePluginScanReceipt`
- `RuntimePluginDiscoveredTypeRecord`
- `PluginBackedNodeBindingProjection`
- `RuntimePluginLifecycleSnapshot`
- `RuntimePluginSandboxSnapshot`
- `RuntimePluginChainSnapshot`
- `RuntimePluginExecutionChainSummary`
- `RuntimePluginChainStageSnapshot`
- `RuntimePluginRecallSnapshot`
- `RuntimePluginRecallHandoffSnapshot`
- `RuntimePluginRecallHandoffStage`
- `RuntimePluginCompensationState`

These surfaces are the only supported authority for how plugin instances,
chains, recall, and compensation appear inside Signal runtime execution and
supervisor export.

### Host-neutral delegated execution

The current host-neutral delegation boundary is:

- `RuntimeOfflinePluginExecutionBoundary`
- `RuntimeOfflinePluginExecutionStageBoundary`
- `RuntimeOfflinePluginExecutionOwner`
- `RuntimeOfflinePluginOverrideState`
- `RuntimeOfflinePluginDelegatedExecutionRequest`
- `RuntimeOfflinePluginDelegatedExecutionStageRequest`
- `RuntimeOfflinePluginDelegatedExecutionReceipt`
- `RuntimeOfflinePluginDelegatedExecutionStageReceipt`
- `RuntimeOfflinePluginDelegatedExecutionOutcome`
- `RuntimeOfflinePluginDelegatedExecutionMerge`

These types are reusable because they describe runtime-owned stage authority,
selection, fulfillment, and merge semantics without exposing one adapter's
format protocol or one host's private execution path.

## Adapter-specific versus reusable detail

### Reusable today

The following belong in shared Signal-owned contracts:

- format-neutral plugin descriptor and lifecycle vocabulary in `signal-plugin`
- runtime plugin lifecycle, chain, recall, compensation, and delegation
  receipts in `signal-runtime`
- host-visible report/export delivery of those receipts
- the rule that delegated execution is described by runtime-owned stage
  boundaries and request/receipt/outcome families rather than adapter-native
  render messages

### Adapter-specific for now

The following remain adapter-specific until Signal promotes them into typed
format-neutral contracts:

- `signal-plugin-clap` extension negotiation such as `ClapHostExtension`
- CLAP discovery/control helpers such as:
  - `ClapDiscoveredPluginType`
  - `ClapInstanceControlSurface`
  - `ClapPreparePlan`
  - `ClapBlockProtocol`
  - CLAP event packet and shared-memory header details
- adapter-native message names, correlation rules, or transport payload mapping
- backend-specific catalog traversal or scan-root semantics beyond the current
  `PluginScanRequest { roots, formats }` shell and runtime-owned discovery
  receipt family

Consumers may rely on the Signal-owned descriptor, snapshot, and delegation
receipts, but they must not depend on CLAP-specific protocol structs unless
they are deliberately writing a CLAP adapter.

## Delegation rules

Delegated execution must remain host-neutral and runtime-owned in meaning:

- runtime decides stage ownership through `RuntimeOfflinePluginExecutionOwner`
  and `host_delegate_required`
- runtime decides what recall payload, override freshness, and stage identity
  a delegate must fulfill
- hosts or backend adapters may execute delegated work, but they must return
  the result through the typed runtime-owned request/receipt/outcome family
- hosts must not invent alternate delegation manifests, completion counters, or
  merge schemas when the runtime-owned receipt family already answers the
  question

If a later backend needs richer delegated capability or refusal detail, it
should be added additively to the runtime-owned receipt family rather than
forked into adapter-specific result contracts.

## Canonical inspection surfaces

Consumers should inspect plugin backend and delegation state in this order:

- use `signal-plugin` descriptor, capability, lifecycle, and fault types when
  the question is backend-neutral plugin meaning
- use `RuntimePluginLifecycleSnapshot`, `RuntimePluginChainSnapshot`, and
  `RuntimePluginRecallHandoffSnapshot` when the question is how runtime is
  currently binding or exporting plugin state
- use `RuntimeOfflinePluginExecutionBoundary` and the delegated
  request/receipt/outcome family when the question is delegated offline
  execution ownership or fulfillment
- use `RuntimeObservationReport` and `RuntimeSupervisorReport` when the
  question is how those runtime-owned receipts are delivered to consumers or
  automation

Hosts and tools may format these surfaces, but they must not reconstruct plugin
ownership from adapter-local state when the typed Signal-owned surfaces already
expose it.

## Current proof boundary

The contract is grounded in implementation that already exists:

- `signal-plugin` already exposes format-neutral descriptor, lifecycle,
  readiness, fault, capability, and sandbox-state-machine types
- `signal-plugin-clap` currently proves one concrete adapter path against that
  substrate, including discovery, control, prepare, and block protocol detail
- `signal-runtime` already exports runtime-owned plugin lifecycle, chain,
  recall, compensation, and delegated offline execution receipts
- `signal-runtime` now also exports runtime-owned plugin scan/discovery
  receipts plus typed plugin-format identity on sandbox, recall, and delegated
  execution stage DTOs, so hosts do not need adapter-local format inference to
  prepare delegated work
- `signal-runtime` now also exports discovered-plugin catalog records with
  format-neutral identity, feature, I/O, state, processing, and lifecycle
  detail through `RuntimePluginDiscoveredTypeRecord`, keeping capability
  meaning on Signal-owned surfaces instead of adapter-local structs
- `signal-runtime/tests/public_contract_boundary.rs` now proves a downstream-
  style consumer can read those discovery records through public runtime
  reexports without touching crate-private implementation detail
- `signal-supervisor-tools` now proves a consumer/export path can carry the
  same discovery catalog through runtime-owned supervisor export instead of
  rebuilding it from host or adapter-local state
- delegated offline execution already round-trips through the same runtime-owned
  manifest/report finalization path instead of a parallel host export model
- `signal-host-local` already proves one concrete delegated executor adapter on
  top of the runtime-owned boundary while still feeding results back through
  runtime-owned outcomes and receipts

Batch 5.1 froze the interpretation of those surfaces, and the first Batch 5.2
depth tranche now proves that scan/filter intent and plugin-format identity can
stay runtime-owned rather than host-local. Later `g04.005` work may deepen
backend breadth or discovery depth, but it should do so by extending the same
Signal-owned contract rather than replacing it with format-specific or
host-specific ownership.

The deferred breadth after this second Batch 5.2 tranche is explicit: broader
consumer conformance fixtures, backend-neutral capability projection beyond the
current discovery catalog/report boundary, and wider adapter coverage such as
VST3/AU remain later work.

## Next Task

COMPLETE. This contract closed with `g04.005`, and the full `g04` generation
is now complete. The next likely queue is recorded in
`docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md`.
