# 059 Renderer-Capability Negotiation And Immersive Export Contract

Status: complete
Owner: core-product
Updated: 2026-03-21
Related contracts: `docs/contracts/036-spatial-adapter-execution-contract.md`, `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`, `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`, `docs/contracts/058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned renderer-capability negotiation and
immersive export boundary so later immersive export, renderer selection, and
packaging work builds on one runtime-owned meaning instead of renderer-private
capability tables, host-local export glue, or product-local immersive release
policy.

## Authority hierarchy

Renderer-capability negotiation and immersive export meaning has one authority
chain:

1. this contract defines renderer-capability negotiation posture, capability
   authority, immersive export class, export authority, and immersive export
   outcome
2. `docs/contracts/036-spatial-adapter-execution-contract.md` remains the
   authority for baseline spatial adapter execution, target environment, and
   fallback posture that renderer-capability negotiation must widen rather than
   replace
3. `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`
   remains the authority for bed/object/mix-policy and render-scope meaning
   that immersive export must compose with rather than reopen
4. `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`
   remains the authority for immersive object-rendering posture, room-policy
   class, room-policy authority, and immersive room outcome that renderer and
   export posture must layer on top of
5. `docs/contracts/058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`
   remains the authority for deployment class, fold-down policy,
   monitoring-scene authority, and monitoring outcome that export and renderer
   negotiation must obey rather than bypass
6. `signal-graph` owns node-local graph identity and any later structural
   metadata needed to stage export-aware or renderer-aware immersive execution
7. `signal-runtime` must own the typed execution, observation, diagnostic, and
   render receipts that expose renderer capability and immersive export truth
   to hosts and supervisor consumers
8. hosts, renderers, adapters, and downstream products may contribute concrete
   renderer vendor detail, export container detail, or UX, but they must not
   redefine renderer-capability or immersive export meaning once runtime-owned
   receipts exist

If a renderer-capability or immersive export claim cannot be explained through
this contract, the closed spatial, room-policy, deployment-monitoring, and
runtime-owned topology receipts, it is not yet part of the shared immersive
export boundary.

## Existing anchors

Batch 8.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending renderer-aware immersive export depth already
exists:

- `docs/contracts/036-spatial-adapter-execution-contract.md`
  - baseline spatial adapter, execution-mode, and fallback meaning that
    renderer-capability negotiation must widen rather than replace
- `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`
  - bed/object, mix-policy, and render-scope vocabulary that immersive export
    meaning must obey
- `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`
  - immersive room-policy meaning that renderer capability and export posture
    must layer on top of instead of replacing
- `docs/contracts/058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`
  - deployment and monitoring meaning that immersive export and renderer
    negotiation must compose with
- `crates/signal-runtime/src/interfaces.rs`
  - current spatial execution, immersive room-policy, deployment-monitoring,
    observation, and offline-preview seams that later batches must widen
- `crates/signal-runtime/src/runtime.rs`
  - current routing, richer-spatial fallback, room-policy projection, and
    render-preview seams that define the bounded baseline this contract expands
    from

This contract does not claim renderer-capability negotiation or immersive
export packaging is already implemented. It freezes the meaning later runtime
and renderer work must obey.

## Shared vocabulary

### Renderer-capability negotiation posture

A `renderer-capability negotiation posture` is the runtime-owned description of
how strongly Signal can establish renderer compatibility for the current
immersive request.

Batch 8.1 freezes this bounded family:

- `NotRequested`
- `DeclaredCompatible`
- `NegotiatedCompatible`
- `FallbackNegotiation`
- `Unavailable`

This posture is stronger than raw plugin or renderer presence. It says whether
Signal has only a declaration, a negotiated result, or only a bounded fallback
answer.

### Capability authority

`capability authority` means where the currently active renderer-capability
meaning is allowed to come from.

Batch 8.1 freezes this bounded family:

- `RuntimeDefault`
- `RuntimeDeclared`
- `HostForwarded`
- `RendererAdvisory`

This keeps the shared ownership line explicit. Hosts and renderers may forward
detail, but the shared capability boundary must still reduce to one
runtime-owned authority answer.

### Immersive export class

An `immersive export class` is the runtime-owned category of the export shape
Signal is trying to satisfy for the current immersive material.

Batch 8.1 freezes this bounded family:

- `NoImmersiveExport`
- `BedOnlyExport`
- `ObjectAwareExport`
- `MonitoringPreviewExport`
- `FallbackExport`

Immersive export class is not a product packaging SKU and not a renderer
vendor format blob. It describes reusable runtime-owned export intent.

### Export authority

`export authority` means where the currently active immersive export meaning is
allowed to come from.

Batch 8.1 freezes this bounded family:

- `RuntimeDefault`
- `RuntimeDeclared`
- `HostForwarded`
- `RendererAdvisory`

This keeps export packaging ownership explicit and prevents host-local export
rules from silently becoming shared truth.

### Immersive export outcome

An `immersive export outcome` is the runtime-owned result when renderer
capability, room-policy, deployment, and export intent are projected onto the
currently available immersive substrate.

Batch 8.1 freezes this bounded family:

- `PreserveDeclaredExport`
- `CollapseToBedExport`
- `PreserveMetadataOnly`
- `BypassImmersiveExport`
- `TerminalExportFailure`

This outcome is distinct from room-policy and monitoring outcome. It explains
what happened at the immersive export boundary, not only during live playback
or monitoring.

## Rules

### Rule 1: renderer capability and immersive export meaning must stay runtime-owned

Renderer-capability negotiation posture, capability authority, immersive export
class, export authority, and immersive export outcome belong to runtime-owned
receipts, not to renderer-private capability tables, host-local export glue, or
product-local immersive release policy.

### Rule 2: renderer and export meaning must compose with the closed immersive seams

This contract widens the closed spatial, richer-spatial, immersive room-policy,
and deployment-monitoring seams. It must not replace render scope, room-policy
truth, deployment class, or monitoring outcome with a second renderer/export
taxonomy.

### Rule 3: renderer-private detail stays advisory

Concrete vendor capability blobs, export manifest internals, encoder feature
lists, and renderer-native package metadata may exist internally, but the
shared boundary must remain grounded in runtime-owned capability posture,
export class, authority, and outcome.

### Rule 4: live, offline, and export surfaces must converge

Later export work may stage rollout, but it must not create one renderer model
for live execution, another for offline export, and a third for observation or
supervisor export.

### Rule 5: product release packaging stays out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze product-local release UX, publishing workflows, store metadata, or
distribution-channel policy.

## Deferred scope

Batch 8.1 intentionally leaves these out:

- final runtime execution, render, and observation receipts for renderer
  capability and immersive export
- public runtime, supervisor, and host-edge proof surfaces
- concrete export containers, renderer vendor payload schemas, or downstream
  release packaging
- renderer-backed calibration, binauralization, or product-local immersive
  export UX

## Batch 8.1 outcome

Batch 8.1 freezes the first reusable renderer-capability negotiation and
immersive export authority line for Signal:

- renderer-capability negotiation posture, capability authority, immersive
  export class, export authority, and immersive export outcome are now
  explicit Signal-owned vocabulary
- later runtime execution can widen on top of the closed spatial, room-policy,
  and deployment-monitoring seams instead of inventing a parallel renderer or
  export policy model
- unsupported renderer/export paths are now required to explain themselves
  through bounded export outcome meaning rather than host-local packaging glue
  or renderer-private interpretation

## Batch 8.2 outcome

Batch 8.2 turns this contract into a real runtime seam:

- `signal-runtime` now owns typed renderer capability and immersive export
  posture through `RuntimeRendererImmersiveExportSummary` on the shared spatial
  execution surface
- topology and offline render preview now count renderer-capability,
  negotiated-renderer, immersive-export, and fallback-export spatial work
  directly from runtime-owned receipts rather than consumer-side recounting
- the stable local and server host edges now export the same bounded renderer
  negotiation and immersive export answers as the runtime-owned spatial seam

The bounded baseline is intentionally conservative:

- stereo balance-only paths still report no renderer/export receipt
- the current surround fallback path reports `FallbackNegotiation`,
  `RuntimeDefault`, `FallbackExport`, `RuntimeDefault`, and
  `BypassImmersiveExport`
- Batch 8.3 still has to prove the widened seam through the shared supervisor
  consumer boundary before this milestone is complete

## Batch 8.3 outcome

Batch 8.3 closes the widened consumer boundary without inventing a second
renderer-only proof seam:

- `signal-supervisor-tools` now points the existing
  `signal.runtime.spatial-boundary` descriptor at this contract instead of the
  earlier deployment-only seam
- the shared supervisor descriptor now names renderer-capability,
  negotiated-renderer, immersive-export, and fallback-export topology and
  render-preview anchors alongside `spatial_execution.renderer_export`
- the existing `effigy acceptance:spatial-boundary` lane now proves the same
  bounded renderer-capability and immersive-export answers across public
  runtime, stable host-edge, and supervisor surfaces

This closes the bounded contract meaningfully:

- later renderer or export-depth work now has one explicit runtime-owned
  authority line to build on
- renderer-private capability tables and host-local export shells are no
  longer needed to inspect the current fallback surround path
- deeper renderer-backed execution, vendor export package schemas, and
  publication workflows remain intentionally deferred

## Next Task

Continue `g08.009` with Batch 9.1 by freezing the first runtime-owned advanced
control-surface display, motor, and haptic transport contract on top of the
closed controller-expression, control-surface, advanced-hardware, and richer
workflow seams.
