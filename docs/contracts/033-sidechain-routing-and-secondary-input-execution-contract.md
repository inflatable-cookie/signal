# 033 Sidechain Routing And Secondary-Input Execution Contract

Status: complete
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`, `docs/contracts/015-offline-render-recovery-and-resumability-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned sidechain and secondary-input routing
boundary so later multi-bus, complex plugin-I/O, spatial, and render work
build on one runtime-owned meaning instead of host-local patch routing or
adapter-private sidechain reconstruction.

## Authority hierarchy

Sidechain and secondary-input meaning has one authority chain:

1. this contract defines sidechain source, target, secondary-input identity,
   fallback meaning, and bounded failure semantics
2. `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
   remains the base authority for channel-role, canonical layout, and bus-intent
   meaning that sidechain routing must layer on top of rather than replace
3. `signal-graph` owns graph-node bus endpoints, bus ids, topology metadata,
   and graph-edge structure that later execution work must use to express
   secondary-input relationships
4. `signal-runtime` must own the typed routing, execution, observation, and
   render receipts that expose sidechain truth to hosts and supervisor
   consumers
5. hosts, adapters, and downstream products may contribute concrete plugin or
   device capabilities, but they must not redefine sidechain source, target, or
   fallback meaning once runtime-owned receipts exist

If a secondary-input routing claim cannot be explained through this contract,
the closed multichannel substrate, and runtime-owned routing receipts, it is
not yet part of the shared sidechain boundary.

## Existing anchors

Batch 2.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending sidechain execution already exists:

- `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
  - canonical layout, channel-role, and bus-intent meaning
  - explicit `Sidechain`, `AuxSend`, and `AuxReturn` bus-intent vocabulary
- `crates/signal-graph/src/lib.rs`
  - graph-node bus endpoints and bus ids
  - graph-node topology metadata including send or return identity
- `crates/signal-runtime/src/interfaces.rs`
  - execution-topology, plugin discovery, and plugin-stage receipt families
    that later batches must widen with secondary-input truth
- `crates/signal-runtime/src/runtime.rs`
  - current graph planning, plugin-stage realization, and render execution
    seams that will later need one shared secondary-input model

This contract does not claim live or offline sidechain execution is already
implemented. It freezes the meaning the later runtime and graph work must obey.

## Shared vocabulary

### Secondary input

A `secondary input` is a non-primary audio or control input that a node or
plugin consumes in addition to its main program path.

Batch 2.1 freezes these rules:

- a secondary input must remain explicitly distinct from the primary program
  input
- a secondary input may be associated with a `Sidechain`, `AuxReturn`, or later
  additive routing intent, but it does not inherit meaning from raw bus count
  alone
- secondary-input identity must survive graph planning, runtime execution, and
  render receipts without host-local relabeling

### Sidechain source

A `sidechain source` is the runtime-owned origin of a secondary-input signal.
It identifies which node or bus is sending secondary-input material and why.

Batch 2.1 freezes this bounded source family:

- `NodeOutput(node_id, bus_id)`
- `BusGroup(bus_group_id)`
- `HardwareInput(endpoint_id)` for later hardware-driven secondary-input paths
- `AnalysisTap(node_id)` for later bounded analysis-driven control routing

Later batches may widen the source family, but they must stay additive and
Signal-owned.

### Sidechain target

A `sidechain target` is the runtime-owned consumer of a secondary input.

Batch 2.1 freezes this bounded target family:

- `NodeInput(node_id, bus_id)`
- `PluginInput(node_id, plugin_bus_id)`
- `RenderInput(render_session_id, bus_id)` for later offline alignment

Targets define routing identity. They do not imply product mixer UX,
plugin-editor wiring, or host-specific patch-bay representation.

### Attachment policy

`attachment policy` means how strongly Signal expects a secondary input to be
present at execution time.

Batch 2.1 freezes this bounded policy family:

- `Required`
- `Optional`
- `Disabled`

`Required` means execution or render behavior must surface explicit fallback or
failure truth when the secondary input cannot be attached. `Optional` means the
node may proceed on its main path while surfacing the missing or inactive
secondary-input state. `Disabled` means the declared secondary-input path is
intentionally inactive and must not be guessed back into use.

### Fallback outcome

A `fallback outcome` is the runtime-owned result when the declared secondary
input cannot be attached or cannot remain active.

Batch 2.1 freezes this bounded outcome family:

- `BypassSecondaryInput`
- `MuteDependentPath`
- `SafeModeDegradation`
- `TerminalRoutingFailure`

Later batches may add more precise outcomes, but they must remain additive and
runtime-owned.

## Rules

### Rule 1: sidechain is routing meaning, not just another channel count

Secondary-input and sidechain routing must be declared through explicit source,
target, and attachment policy. Raw channel count or bus count is not enough.

### Rule 2: source and target identity must remain explicit

If Signal cannot say where a secondary input comes from and where it is meant
to land, it does not yet have shared sidechain truth.

### Rule 3: fallback must stay runtime-owned

Hosts and downstream products may observe fallback outcomes, but they must not
invent their own missing-sidechain policy once runtime-owned receipts exist.

### Rule 4: live and offline paths must share one contract

This contract applies to both live execution and offline render or export
paths. Later implementation work may stage rollout, but it must not create one
host-local live sidechain model and a separate render-only model.

### Rule 5: multi-bus breadth stays out of scope for now

This contract freezes sidechain and secondary-input meaning only. Broader
multi-bus execution, richer auxiliary topology, and complex plugin-I/O
expansion belong to later `g07` milestones.

## Deferred scope

Batch 2.1 intentionally leaves these out:

- full multi-bus routing matrices and arbitrary bus fan-out
- plugin-format-specific multi-bus attachment semantics
- product-local sidechain UX, patch-bay editing, or visual routing tools
- final live or offline runtime receipts and proof surfaces
- sidechain-aware spatial policy or surround-specific ducking behavior

## Batch 2.1 outcome

Batch 2.1 freezes the first reusable sidechain authority line for Signal:

- sidechain and secondary-input identity are now explicit routing meaning
  instead of an implied host patch convention
- source, target, attachment policy, and fallback outcome are now bounded
  Signal-owned vocabulary
- live and offline paths are now required to converge on one sidechain model
  instead of forking later
- later `g07` batches can widen runtime execution, multi-bus, complex plugin
  I/O, and spatial depth without reopening the base sidechain question

## Batch 2.2 outcome

Batch 2.2 materializes this contract on shared runtime surfaces:

- `GraphNodeBufferContractProjection` now carries bounded
  `secondary_input` projection so sidechain intent survives graph-contract
  application rather than being inferred later
- `RuntimePlannedGraphNode`, `RuntimeExecutionTopologySummary`, and
  `RuntimeExecutionNodeSummary` now expose runtime-owned sidechain route
  receipts with explicit source, target, attachment policy, and fallback
  outcome
- `RuntimePluginChainStageSnapshot` now keeps that same route family on the
  plugin-facing execution boundary instead of adapter-local sidechain naming
- `RuntimeOfflineRenderChainDependencyPreview` now carries aligned sidechain
  dependency receipts and counts so offline execution no longer needs a
  separate render-only sidechain model

This batch freezes the first real execution depth for the contract while still
deferring consumer-boundary proof to Batch 2.3.

## Batch 2.3 outcome

Batch 2.3 closes the first public sidechain consumer boundary:

- public runtime proof now covers sidechain source, target, attachment policy,
  and fallback receipts through shared runtime reports
- stable local and server host edges now prove the same runtime-owned
  secondary-input receipts survive `supervisor_report()` without host-local
  routing reinterpretation
- `signal-supervisor-tools` now exposes `signal.runtime.sidechain-boundary`
- `effigy acceptance:sidechain-boundary` now keeps the focused runtime, host,
  and descriptor proof spine runnable as one repo-owned task

This closes the bounded sidechain milestone while still deferring broader
multi-bus, complex plugin-I/O, and spatial routing breadth to later `g07`
work.

## Next Task

Continue `g07.003` with Batch 3.1 by freezing the runtime-owned multi-bus
graph execution and auxiliary-topology contract on top of the now-closed
multichannel and sidechain routing boundaries.
