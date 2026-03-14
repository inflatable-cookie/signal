# 008 Backend-Neutral Plugin Capability And Adapter Breadth Contract

Status: active
Owner: core-product
Updated: 2026-03-12
Related contracts: `docs/contracts/002-supervisor-export-schema-and-report-boundary.md`, `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/007-plugin-backend-and-host-neutral-delegation-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first post-CLAP backend-neutral plugin capability and adapter-breadth
contract so later `g05.001` work can widen supported plugin backends without
reopening host-local ownership, adapter-private lifecycle meaning, or
consumer-local capability reconstruction.

## Authority hierarchy

Backend-neutral plugin breadth has one authority chain:

1. `signal-plugin` owns the format-neutral plugin vocabulary and the shared
   capability meaning for all backends:
   - plugin identity, format, features, and I/O layout
   - parameter, processing, lifecycle, state, and fault contracts
   - sandbox capability and readiness abstractions
2. `signal-runtime` owns the active runtime-side interpretation of those plugin
   capabilities:
   - discovery snapshots and scan receipts
   - runtime lifecycle, chain, recall, compensation, and delegation surfaces
   - report/export delivery through runtime-owned observation and supervisor
     receipts
3. adapter crates such as `signal-plugin-clap` or future backend adapters own
   protocol-specific realization detail:
   - discovery protocol, extension negotiation, control surfaces
   - backend-native transport or event packet mapping
   - backend-specific prepare/block/execution helpers
4. host crates may request scans, broker adapters, or fulfill delegated work,
   but they must not become the authority for backend capability meaning,
   lifecycle semantics, or widened discovery contracts when typed Signal-owned
   surfaces already exist

If later backend breadth needs richer consumer-visible meaning, that meaning
should be promoted into `signal-plugin` or `signal-runtime`, not inferred from
backend adapter internals.

## Backend-neutral capability promises

The widened consumer boundary keeps four promises.

### Shared capability meaning stays format-neutral

The following capability questions must continue to be answered through
Signal-owned shared vocabulary rather than adapter-local enums or records:

- what the plugin is
- which feature classes it belongs to
- which I/O shape it supports by default
- which state, processing, and lifecycle guarantees it exposes
- which sandbox or delegation capabilities it requires or permits

`PluginFormat` remains the backend identity tag, but it is not enough on its
own. Consumer-visible capability meaning belongs to the existing
format-neutral `signal-plugin` contract family and any future additive
extensions to that family.

### Runtime-owned discovery and execution receipts stay authoritative

When a backend is discovered, bound, recalled, delegated, or exported, the
authoritative shared receipts remain:

- `RuntimePluginDiscoverySnapshot`
- `RuntimePluginScanReceipt`
- `RuntimePluginDiscoveredTypeRecord`
- runtime plugin lifecycle, chain, recall, and compensation snapshots
- runtime delegated offline execution request/receipt/outcome families
- `RuntimeObservationReport` and `RuntimeSupervisorReport`

Hosts or consumers may format those surfaces, but they must not reconstruct
capability or lifecycle meaning from adapter-private scan results when the
runtime-owned receipts already expose it.

The widened discovery/catalog receipt family now also includes runtime-owned
aggregate breadth surfaces so consumers can inspect backend coverage without
recounting raw discovered-type records themselves:

- `RuntimePluginFormatCoverageRecord`
- `RuntimePluginCapabilityCoverageSummary`

The current conformance proof for those widened receipts is also Signal-owned
rather than adapter-local:

- `public_runtime_plugin_discovery_coverage_is_consumable_from_reexports`
- `export_json_carries_runtime_owned_plugin_discovery_capability_coverage`
- `effigy acceptance:plugin-backend-breadth`

### Adapter breadth is additive, not substitutive

New adapter families may widen backend breadth only by adding to the shared
Signal contract:

- add new format-neutral capability fields only in `signal-plugin` or
  `signal-runtime`
- add backend identity through `PluginFormat` or additive Signal-owned DTOs
- add runtime-owned receipts when broader backend behavior becomes
  consumer-visible

New backend support must not replace shared Signal-owned capability meaning with
adapter-native structs, message names, or backend-local report shells.

### Delegation and lifecycle semantics must not fork by backend

Delegated execution, lifecycle, recall, and fault semantics remain shared
Signal-owned meaning even when the fulfilling backend differs:

- lifecycle state and readiness are described through Signal-owned surfaces
- delegated stage ownership and fulfillment remain runtime-owned
- backend-specific refusal or extension detail must be additive to the shared
  receipt family rather than a parallel backend-only contract

If a later adapter needs richer refusal, capability, or stage-policy detail,
that detail should extend the shared receipt family additively before consumers
depend on it.

## Adapter-private detail versus promoted shared detail

### Shared today

The following are already part of the widened shared boundary:

- `PluginFormat` as the canonical backend identity tag
- format-neutral descriptor, feature, I/O, state, processing, lifecycle, and
  fault contracts in `signal-plugin`
- runtime-owned discovery/catalog/export surfaces in `signal-runtime`
- runtime-owned delegated execution boundaries and receipts
- supervisor/export delivery of those runtime-owned plugin receipts

### Adapter-private for now

The following remain adapter-private until Signal explicitly promotes them:

- backend-specific extension negotiation and protocol helpers
- backend-native scan traversal detail beyond current runtime-owned scan roots
  and format filters
- adapter-local prepare plans, event/block packet structures, and transport
  headers
- backend-specific capability knobs that are not yet represented through
  format-neutral Signal-owned DTOs

Consumers may depend on shared Signal-owned capability and receipt surfaces,
but they must not depend on backend crate internals unless they are writing that
backend adapter itself.

## Canonical inspection order

Consumers should inspect widened backend state in this order:

- use `signal-plugin` types when the question is backend-neutral plugin
  capability meaning
- use runtime discovery, lifecycle, recall, compensation, and delegated
  execution receipts when the question is how those capabilities are realized in
  active Signal runtime behavior
- use runtime observation/supervisor export when the question is how those
  runtime-owned receipts are delivered to automation or external consumers

Adapter-private detail is explanatory only. It is not a substitute authority
for consumer-visible backend breadth.

## Initial deferred breadth

This first `g05.001` contract intentionally defers several areas:

- publication of backend-neutral capability fields that Signal does not yet
  model in `signal-plugin` or `signal-runtime`
- deciding which host convenience APIs expose wider backend breadth directly
- broader release-packaging or downstream automation claims based on widened
  backend support
- any backend-specific UX, workflow, or consumer-local orchestration

Those areas belong to later `g05` milestones after the widened capability
boundary is explicit.

## Next Task

Continue `g05.005` with Batch 5.1 by defining the combined `g05`
generation-closeout descriptor and task without weakening the runtime-owned
backend-neutral boundary.
