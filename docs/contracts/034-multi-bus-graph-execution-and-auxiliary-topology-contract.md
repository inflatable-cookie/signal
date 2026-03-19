# 034 Multi-Bus Graph Execution And Auxiliary Topology Contract

Status: active
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`, `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned multi-bus and auxiliary-topology
boundary so later complex plugin-I/O, spatial routing, Linux backend breadth,
and render topology work build on one runtime-owned meaning instead of host
patch-bay convention or product-local bus heuristics.

## Authority hierarchy

Multi-bus and auxiliary-topology meaning has one authority chain:

1. this contract defines bus-role, auxiliary-path, connection identity,
   attachment class, and fallback meaning
2. `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
   remains the authority for canonical layout, channel-role, and bus-intent
   meaning that bus topology must layer on top of rather than replace
3. `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`
   remains the authority for secondary-input routing, sidechain source, target,
   and fallback meaning where an auxiliary path carries sidechain material
4. `signal-graph` owns graph-node bus endpoints, bus ids, graph edges, and
   topology metadata that later execution work must use to express multi-bus
   connections explicitly
5. `signal-runtime` must own the typed execution, observation, diagnostic, and
   render receipts that expose multi-bus and auxiliary-topology truth to hosts
   and supervisor consumers
6. hosts, adapters, and downstream products may contribute concrete plugin or
   device capabilities, but they must not redefine bus-role or auxiliary-path
   meaning once runtime-owned receipts exist

If a multi-bus routing claim cannot be explained through this contract, the
closed multichannel and sidechain contracts, and runtime-owned topology
receipts, it is not yet part of the shared routing boundary.

## Existing anchors

Batch 3.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending richer auxiliary execution already exists:

- `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
  - canonical layout, channel-role, custom-layout fallback, and bus-intent
    meaning
- `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`
  - secondary-input source, target, attachment policy, and fallback meaning
- `crates/signal-graph/src/lib.rs`
  - graph-node bus endpoints, topology metadata, and send or return identity
- `crates/signal-runtime/src/interfaces.rs`
  - execution-topology, plugin-stage, and offline render receipt families that
    later batches must widen with explicit multi-bus topology truth
- `crates/signal-runtime/src/runtime.rs`
  - current graph planning, plugin-stage realization, and offline dependency
    seams that will later need one shared multi-bus model

This contract does not claim richer multi-bus execution is already implemented.
It freezes the meaning the later runtime and graph work must obey.

## Shared vocabulary

### Bus role

A `bus role` is the runtime-owned purpose of one bus within a wider graph
topology. Batch 3.1 freezes this first bounded family:

- `ProgramMain`
- `ProgramStem`
- `AuxSend`
- `AuxReturn`
- `Submix`
- `ParallelProcess`
- `AnalysisTap`
- `HardwareIngress`
- `HardwareEgress`

Bus role is not the same thing as canonical layout or sidechain attachment. It
describes why a bus exists in the execution graph.

### Auxiliary path

An `auxiliary path` is a runtime-owned routed path that leaves or rejoins the
primary program path without losing explicit source, target, and bus-role
identity.

Batch 3.1 freezes these rules:

- an auxiliary path must retain one explicit origin and one explicit landing
  point
- an auxiliary path may pass through send, return, submix, or parallel-process
  topology, but it must remain visible as one declared routing relationship
- an auxiliary path may carry main-program, sidechain, or analysis material,
  but it must not silently change meaning mid-route

### Connection identity

A `connection identity` is the runtime-owned identity of one declared bus
relationship. Batch 3.1 freezes this bounded identity model:

- `source_node_id`
- `source_bus_id`
- `target_node_id`
- `target_bus_id`
- optional `path_id`
- optional `send_return_id`

If Signal cannot say which bus leaves which node and where it lands, it does
not yet have shared multi-bus truth.

### Attachment class

`attachment class` means how strongly Signal expects one declared bus
connection to remain attached at execution time. Batch 3.1 freezes this first
bounded family:

- `Required`
- `Optional`
- `Disabled`

This parallels the earlier sidechain policy family but applies to broader
multi-bus graph relationships instead of only secondary inputs.

### Fallback outcome

A `fallback outcome` is the runtime-owned result when a declared multi-bus or
auxiliary path cannot be attached or cannot remain active.

Batch 3.1 freezes this bounded family:

- `BypassAuxPath`
- `MuteDependentPath`
- `SafeModeDegradation`
- `TerminalTopologyFailure`

Later batches may widen this family, but they must remain additive and
runtime-owned.

## Rules

### Rule 1: multi-bus topology must stay explicit

Richer bus behavior must be declared through explicit bus roles, connection
identity, and attachment class. Raw node count, stage count, or channel count
is not enough.

### Rule 2: auxiliary paths must not become host-local patch wiring

Hosts and downstream products may visualize or edit routing later, but shared
Signal meaning must stay in runtime-owned topology receipts rather than host
patch-bay reconstruction.

### Rule 3: send, return, submix, and parallel semantics must share one model

Signal may stage implementation rollout, but it must not create one topology
model for sends, another for returns, and a third for offline render preview.

### Rule 4: sidechain remains distinct from broader multi-bus topology

Sidechain routing may travel through auxiliary topology, but sidechain source,
target, and fallback meaning remain governed by Contract 033 rather than being
flattened into generic bus attachment.

### Rule 5: product workflow stays out of scope

This contract freezes reusable runtime and graph meaning only. It does not
freeze console UX, bus-color policy, editor layouts, or final product routing
affordances.

## Deferred scope

Batch 3.1 intentionally leaves these out:

- plugin-format-specific multi-bus realization and negotiation
- final live execution, render, and diagnostic receipts
- spatial/object routing and immersive bus semantics
- product-local patch-bay, console, or mixer UI
- distributed or remote routing behavior

## Batch 3.1 outcome

Batch 3.1 freezes the first reusable multi-bus authority line for Signal:

- bus role, auxiliary path, connection identity, attachment class, and
  fallback outcome are now explicit Signal-owned routing vocabulary
- richer send, return, submix, and parallel-process behavior now have one
  shared meaning target instead of reopening routing semantics later
- later `g07` batches can widen runtime execution, complex plugin-I/O,
  spatial routing, and Linux-native topology depth without reopening the base
  multi-bus question

## Batch 3.2 outcome

Batch 3.2 applies this contract to active runtime-owned execution surfaces.

The shared routing family now includes:

- `RuntimeBusConnectionSummary`
  - connection identity, source and target bus role, optional auxiliary-path
    grouping, attachment class, and fallback outcome
- `RuntimeAuxiliaryPathSummary`
  - path kind, runtime-owned bus role, material intent, source and target node
    identity, and the grouped connection ids that make one auxiliary path
    explicit instead of inferred
- widened `RuntimeExecutionTopologySummary`,
  `RuntimeMeteringSnapshot`, and
  `RuntimeOfflineRenderChainDependencyPreview`
  - bus-connection counts, auxiliary-path counts, and the concrete connection
    and path receipts above

Batch 3.2 therefore closes the first real runtime-owned multi-bus execution
depth:

- live execution topology, offline render dependency preview, and diagnostic
  metering now share one explicit multi-bus receipt family
- send or return and submix paths remain visible as runtime-owned auxiliary
  paths rather than being flattened into raw node, bus, or group counts
- fallback and attachment meaning is now carried by typed routing receipts
  rather than only implied by the contract text

## Batch 3.3 outcome

Batch 3.3 proves that the widened multi-bus receipt family is consumable on
shared public surfaces rather than only inside runtime internals.

The following boundary is now explicit and repo-owned:

- public `signal-runtime` reports expose bus-role, connection-identity, and
  auxiliary-path receipts directly
- stable local and server host edges forward the same receipts through
  `RuntimeSupervisorReport`
- `signal-supervisor-tools` exposes a machine-readable
  `signal.runtime.multi-bus-boundary` descriptor plus a runnable
  `effigy acceptance:multi-bus-boundary` task

That closes the first bounded consumer seam for this contract and leaves later
complex plugin-I/O and spatial work building on a proven shared topology
substrate rather than reopening multi-bus meaning.

## Next Task

Continue `g07.004` with Batch 4.2 by materializing runtime-owned complex
plugin-I/O, multi-output instrument, and bus-capable FX receipts across
discovery, execution, render, and stable host-edge surfaces without reopening
adapter-local pin ownership.
