# 037 Surround Bed, Object, And Mix-Policy Expansion Contract

Status: active
Owner: core-product
Updated: 2026-03-17
Related contracts: `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`, `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`, `docs/contracts/036-spatial-adapter-execution-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the next reusable Signal-owned spatial expansion boundary so later
surround-bed, object, and mix-policy work builds on one runtime-owned meaning
instead of product-local immersive-console policy, renderer-private object
models, or host-local speaker heuristics.

## Authority hierarchy

Richer spatial expansion meaning has one authority chain:

1. this contract defines bounded surround-bed class, object role, mix-policy,
   render scope, and fallback meaning
2. `docs/contracts/036-spatial-adapter-execution-contract.md` remains the
   authority for the baseline spatial adapter class, execution mode, target
   environment, control family, activation policy, and fallback posture that
   richer surround or object work must widen rather than replace
3. `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
   remains the authority for canonical layout, channel-role, and custom-layout
   fallback meaning that bed and object expansion must layer on top of
4. `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
   remains the authority for bus-role, auxiliary-path, and connection identity
   where beds, objects, or mix-policy participate in broader send, return,
   submix, or parallel topology
5. `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`
   remains the authority for plugin-facing complex I/O and bus-capable topology
   meaning where a spatial-capable plugin exposes richer bed or object outputs
6. `signal-graph` owns node-local graph identity, staged spatial selection, and
   any later graph-facing metadata needed to stage bed and object execution
7. `signal-runtime` must own the typed execution, observation, diagnostic, and
   render receipts that expose surround-bed, object, and mix-policy truth to
   hosts and supervisor consumers
8. hosts, adapters, renderers, and downstream products may contribute concrete
   renderer capabilities, speaker-device detail, or authoring UX, but they
   must not redefine bed, object, or mix-policy meaning once runtime-owned
   receipts exist

If a richer spatial claim cannot be explained through this contract, the closed
multichannel, multi-bus, complex plugin-I/O, and baseline spatial contracts,
and runtime-owned topology receipts, it is not yet part of the shared richer
spatial boundary.

## Existing anchors

Batch 6.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending richer surround or object runtime depth already
exists:

- `../loophole/chorus/specs/engine/spatial-adapters.md`
  - node-owned spatial intent, structural-versus-runtime control split, and the
    broader future posture beyond the current `balance` baseline
- `docs/contracts/036-spatial-adapter-execution-contract.md`
  - the baseline spatial adapter class, execution mode, target environment, and
    fallback vocabulary that richer work must widen rather than replace
- `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
  - canonical layout and channel-role meaning that beds must anchor to
- `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`
  - multi-bus and auxiliary-path meaning that richer spatial routing must obey
- `docs/contracts/035-plugin-complex-io-topology-and-multi-output-instrument-contract.md`
  - plugin-facing topology meaning for multi-output and bus-capable plugins
- `crates/signal-runtime/src/interfaces.rs`
  - current spatial execution summaries, multichannel receipts, and
    offline-render dependency preview seams that later batches must widen
- `crates/signal-runtime/src/runtime.rs`
  - current bounded stereo-balance realization and explicit non-stereo fallback
    behavior that define the baseline this contract expands from

This contract does not claim richer surround beds, objects, or mix-policy are
implemented yet. It freezes the meaning later runtime and adapter work must
obey.

## Shared vocabulary

### Surround bed class

A `surround bed class` is the runtime-owned category of one fixed-layout
speaker bed that remains anchored to canonical layout meaning.

Batch 6.1 freezes this bounded family:

- `StereoBed`
- `CanonicalSurroundBed`
- `CustomDiscreteBed`

Bed class is stronger than raw channel count. It states whether Signal is
working with a fixed reusable bed, not just a channel bucket.

### Object role

An `object role` is the runtime-owned purpose of one position-independent
spatial element relative to a bed.

Batch 6.1 freezes this bounded family:

- `PrimaryObject`
- `AuxiliaryObject`
- `EffectObject`
- `AnalysisObject`

Object role is not a product authoring label. It describes runtime-owned
meaning for a movable or separately rendered spatial element.

### Mix policy

A `mix policy` is the runtime-owned rule for how beds and objects should be
combined or collapsed for the currently declared render target.

Batch 6.1 freezes this bounded family:

- `BedOnly`
- `BedWithObjects`
- `ObjectPreferredWithBedFallback`
- `DownmixToCanonicalBed`
- `CollapseToBaselineSpatial`

Mix policy must stay explicit because richer spatial expansion cannot depend on
host-local assumptions about whether objects survive, collapse, or disappear.

### Render scope

A `render scope` is the runtime-owned description of what spatial material is
currently expected to survive through execution or offline render.

Batch 6.1 freezes this bounded family:

- `BedRender`
- `BedAndObjectRender`
- `BedFoldDownRender`
- `ObjectMetadataOnly`

Render scope is distinct from target environment. It describes what kind of
spatial payload survives, not only where it is targeted.

### Expanded fallback outcome

An `expanded fallback outcome` is the runtime-owned result when richer bed,
object, or mix-policy semantics cannot be realized.

Batch 6.1 freezes this bounded family:

- `CollapseObjectsIntoBed`
- `CollapseToCanonicalBed`
- `CollapseToBaselineSpatial`
- `BypassExpandedSpatial`
- `TerminalExpandedSpatialFailure`

These outcomes widen the baseline fallback family without replacing it.

## Rules

### Rule 1: richer spatial meaning must stay runtime-owned

Bed, object, and mix-policy meaning belongs to runtime-owned receipts, not to
host-local speaker maps, product-local room UX, or renderer-private metadata.

### Rule 2: bed identity must stay anchored to canonical layout

A bed must retain explicit canonical-layout or custom-discrete identity. Signal
must not silently relabel a custom multichannel path as an immersive bed.

### Rule 3: object meaning must stay explicit

If Signal claims object-aware behavior later, it must preserve explicit object
role and mix policy rather than flattening the path back into unnamed channels.

### Rule 4: fallback must remain inspectable

If richer surround or object depth cannot be realized, Signal must explain that
through bounded expanded fallback meaning instead of silently dropping into
host-local renderer behavior.

### Rule 5: live, offline, and diagnostic surfaces must converge

Later richer spatial work may stage rollout, but it must not create one model
for live execution, another for offline render, and a third for diagnostics or
supervisor export.

### Rule 6: room design and product authoring stay out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze room visualization, immersive-console UX, speaker-setup flows, or
product-local authoring workflows.

## Deferred scope

The contract remains intentionally bounded after Batch 6.2. Signal still does
not claim:

- renderer-specific object payload or device-calibration contracts
- public runtime, supervisor, and host-edge proof surfaces for richer spatial
  depth
- real object rendering or non-zero object-role execution beyond typed receipt
  placeholders
- room calibration, headphone virtualization, or speaker-management policy
- product-local immersive authoring or room-design UX

## Batch 6.1 outcome

Batch 6.1 freezes the next reusable richer-spatial authority line for Signal:

- surround-bed class, object role, mix policy, render scope, and expanded
  fallback outcome are now explicit Signal-owned vocabulary
- later runtime execution can widen on top of the closed multichannel,
  multi-bus, complex plugin-I/O, and baseline spatial seams instead of
  inventing a parallel immersive taxonomy
- unsupported object-aware paths, richer surround render targets, and mix-policy
  gaps are now required to explain themselves through bounded expanded fallback
  meaning rather than host-local or product-local heuristics

## Batch 6.2 outcome

Batch 6.2 materializes the first runtime-owned richer-spatial receipt layer on
top of that frozen vocabulary:

- `signal-runtime` now carries surround-bed class, object-role placeholder,
  object count, mix policy, render scope, and expanded fallback directly on
  the existing spatial execution summaries used by planned nodes, execution
  topology, plugin-chain stages, and offline-render previews
- execution-topology and offline-render dependency preview now expose aggregate
  richer-spatial counts for surround-bed, object-aware, and expanded-fallback
  paths instead of leaving that meaning buried in per-node summaries
- the bounded current path stays explicit rather than aspirational:
  stereo `StereoBalance` stages realize `StereoBed` plus `BedOnly`, while
  canonical surround stages surface `CanonicalSurroundBed` plus
  `CollapseToBaselineSpatial` as a runtime-owned expanded fallback

This batch still does not claim true object rendering or renderer-specific
immersive execution depth. It freezes and materializes the reusable substrate
only.

## Batch 6.3 outcome

Batch 6.3 closes the focused public proof seam for that widened substrate:

- public runtime proofs now verify surround-bed, mix-policy, render-scope, and
  expanded-fallback receipts on observation, supervisor, and offline-render
  preview surfaces
- both stable host edges now forward the same richer spatial receipt family on
  supervisor export without host-local speaker heuristics or renderer-local
  reinterpretation
- `signal-supervisor-tools` now points the existing
  `signal.runtime.spatial-boundary` descriptor at this richer-spatial contract
  and describes the widened topology, plugin-chain, and render-preview anchors
  rather than the baseline-only spatial contract

This contract is now frozen and proven for the bounded richer-spatial seam.
Later work may widen it, but it should not reopen ownership.

## Next Task

Continue `g07.007` with Batch 7.1 by mapping LV2-specific discovery,
lifecycle, and Linux-native capability details onto the existing backend-neutral
plugin contract before runtime-owned LV2 realization widens.
