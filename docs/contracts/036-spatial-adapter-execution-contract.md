# 036 Spatial Adapter Execution Contract

Status: active
Owner: core-product
Updated: 2026-03-17
Related contracts: `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`, `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`, `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned spatial adapter execution boundary so
later surround-bed, object, mix-policy, Linux, and adapter breadth work build
on one runtime-owned meaning instead of product-local pan policy,
format-specific renderer behavior, or host-local speaker heuristics.

## Authority hierarchy

Spatial adapter execution meaning has one authority chain:

1. this contract defines spatial adapter class, execution mode, target
   environment, control family, activation policy, and fallback outcome
2. `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
   remains the authority for canonical layout, channel-role, and bus-intent
   meaning that spatial execution must layer on top of rather than replace
3. `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`
   remains the authority for sidechain source, target, attachment policy, and
   fallback meaning where a spatial-capable path also consumes secondary-input
   material
4. `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
   remains the authority for bus-role, auxiliary-path, and connection identity
   where spatial execution participates in broader send, return, submix, or
   parallel topology
5. `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`
   remains the authority for plugin-facing port class, multi-output instrument,
   and bus-capable FX meaning where a spatial-capable plugin exposes richer
   input or output topology
6. `signal-graph` owns node-local graph identity, structural spatial selection,
   and any later graph-facing topology metadata required to stage spatial
   execution
7. `signal-runtime` must own the typed execution, observation, diagnostic, and
   render receipts that expose spatial truth to hosts and supervisor consumers
8. hosts, adapters, and downstream products may contribute concrete renderer
   capabilities, speaker-device detail, or UI policy, but they must not
   redefine spatial execution meaning once runtime-owned receipts exist

If a spatial execution claim cannot be explained through this contract, the
closed multichannel, sidechain, multi-bus, and complex plugin-I/O contracts,
and runtime-owned topology receipts, it is not yet part of the shared spatial
boundary.

## Existing anchors

Batch 5.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending a richer spatial runtime already exists:

- `../loophole/chorus/specs/engine/spatial-adapters.md`
  - node-owned spatial intent, structural versus runtime control split, and the
    initial `balance` / `perChannelGain` adapter posture
- `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
  - canonical layout, channel-role, custom-layout fallback, and bus-intent
    meaning
- `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
  - multi-bus connection identity, auxiliary-path meaning, and bounded fallback
- `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`
  - plugin-facing topology meaning that later spatial-capable adapters or FX
    must align to instead of bypassing
- `crates/signal-runtime/src/interfaces.rs`
  - current execution-topology, plugin-stage, hardware, and observation
    receipt families that later batches must widen with spatial execution truth
- `crates/signal-runtime/src/runtime.rs`
  - current graph planning, topology realization, and render-preview seams that
    later batches must converge on one bounded spatial model

This contract does not claim richer surround or renderer depth is already
implemented. It freezes the meaning later runtime and adapter work must obey.

## Shared vocabulary

### Spatial adapter class

A `spatial adapter class` is the runtime-owned category of one node-local
spatial strategy.

Batch 5.1 freezes this bounded family:

- `Balance`
- `PerChannelGain`
- `LayoutTransform`
- `Renderer`

Adapter class is not a product UI mode and is not a format-private renderer
name. It describes how Signal expects to transform node-local audio meaning.

### Spatial execution mode

A `spatial execution mode` is the runtime-owned description of how one spatial
adapter is currently being applied.

Batch 5.1 freezes this bounded family:

- `Bypassed`
- `BalanceGroups`
- `PerChannelAttenuation`
- `TransformToTargetLayout`
- `RenderToEnvironment`

Execution mode is distinct from adapter identity. The same adapter class may
fall back to a different execution mode when the declared target environment or
layout is not supported directly.

### Target environment

A `target environment` is the runtime-owned environment the spatial adapter is
trying to satisfy.

Batch 5.1 freezes this bounded family:

- `SourceLayout`
- `CanonicalLayout`
- `DeviceLayout`
- `CustomEnvironment`

Target environment must stay explicit because layout-preserving balance,
layout-transform, and renderer behavior are not the same thing.

### Control family

A `control family` is the runtime-owned shape of the runtime-adjustable spatial
controls attached to the adapter.

Batch 5.1 freezes this bounded family:

- `BalanceScalar`
- `PerChannelVector`
- `AdapterParameterSet`

This keeps the Chorus structural-versus-runtime split intact: adapter choice
and static policy stay structural, while controls stay runtime-owned and
parameter-safe.

### Activation policy

`activation policy` means how strongly Signal expects a declared spatial path
to remain active.

Batch 5.1 freezes this bounded family:

- `Disabled`
- `EnabledIfSupported`
- `Required`

This is not the same thing as a broader bus attachment class. It applies to
node-local spatial execution itself.

### Fallback outcome

A `fallback outcome` is the runtime-owned result when a spatial adapter or
target environment cannot be realized.

Batch 5.1 freezes this bounded family:

- `BypassSpatialProcessing`
- `CollapseToBalance`
- `CollapseToPerChannelGain`
- `SafeModeDegradation`
- `TerminalSpatialFailure`

Later batches may add more precise outcomes, but they must remain additive and
runtime-owned.

## Rules

### Rule 1: spatial execution must stay node-owned

Spatial adapter selection and runtime controls belong to node-local runtime
meaning, not to product mixer defaults or host-local speaker policy.

### Rule 2: layout and routing truth remain authoritative

Spatial execution must layer on top of the closed multichannel, sidechain,
multi-bus, and complex plugin-I/O contracts rather than silently replacing
their layout or routing meaning.

### Rule 3: unsupported layouts and adapters require explicit fallback

If a declared adapter class or target environment cannot be realized, Signal
must explain the outcome through bounded fallback vocabulary rather than
dropping back to hidden host behavior or adapter-private interpretation.

### Rule 4: live, offline, and diagnostic surfaces must converge

Later spatial execution work may stage its rollout, but it must not create one
model for live graph execution, another for offline render preview, and a
third for diagnostics or observation.

### Rule 5: adapter-private renderer internals stay advisory

Concrete speaker feeds, plugin-format details, and renderer-native control
names may exist internally, but the shared boundary must remain grounded in
runtime-owned adapter class, execution mode, target environment, and fallback
meaning.

### Rule 6: product room-design and UI policy stay out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze room visualization, surround-console UX, consumer speaker setup flows,
or product-local authoring policy.

## Deferred scope

Batch 5.1 intentionally leaves these out:

- final runtime execution, render, and observation receipts for spatial paths
- public runtime, supervisor, and host-edge proof surfaces
- surround bed, object, and mix-policy semantics
- room calibration, headphone virtualization, and speaker-management policy
- product-local panner UX, room visualization, or immersive authoring tools

## Batch 5.1 outcome

Batch 5.1 freezes the first reusable spatial execution authority line for
Signal:

- spatial adapter class, execution mode, target environment, control family,
  activation policy, and fallback outcome are now explicit Signal-owned
  vocabulary
- later runtime execution can widen on top of the closed multichannel,
  sidechain, multi-bus, and complex plugin-I/O seams instead of inventing a
  separate spatial taxonomy
- unsupported custom layouts, unsupported adapters, and renderer gaps are now
  required to explain themselves through bounded fallback meaning rather than
  host-local or product-local pan heuristics

## Batch 5.2 outcome

Batch 5.2 now materializes the first bounded spatial execution baseline on
runtime-owned surfaces instead of leaving spatial meaning frozen only in prose.

The shared runtime boundary now carries:

- typed spatial execution summaries on planned nodes, execution-topology nodes,
  and plugin-chain stages
- explicit runtime-owned counts for active, bypassed, and fallback spatial
  nodes on `RuntimeExecutionTopologySummary`
- aligned offline-render dependency preview receipts that enumerate spatial
  stages and their fallback state instead of leaving render-only spatial truth
  implicit

The current realized baseline is deliberately narrow and explicit:

- `GraphStageSpec::StereoBalance` is the first real adapter-backed spatial path
- stereo paths realize bounded `BalanceGroups` execution
- non-stereo paths currently surface explicit
  `BypassSpatialProcessing` fallback instead of pretending broader spatial
  execution already exists

This closes the runtime realization tranche for the contract while still
deferring the public proof boundary to Batch 5.3.

## Batch 5.3 outcome

Batch 5.3 now closes the bounded spatial consumer seam instead of leaving
spatial execution as a runtime-only internal receipt family.

The shared proof spine now covers:

- public runtime execution-topology, plugin-chain, and offline-render preview
  surfaces
- stable host-edge supervisor reports for both local and server hosts
- a machine-readable `signal.runtime.spatial-boundary` descriptor and repo-owned
  `effigy acceptance:spatial-boundary` task

This means the first spatial baseline is now consumable through one shared
runtime-owned vocabulary for active versus bypassed execution, target
environment, and explicit fallback outcome, without host-local speaker
heuristics or adapter-local reinterpretation.

## Next Task

Continue `g07.006` with Batch 6.2 by materializing runtime-owned surround-bed,
object-role, mix-policy, render-scope, and expanded-fallback receipts across
execution, render, and observation surfaces without reopening host-local or
renderer-local spatial ownership.
