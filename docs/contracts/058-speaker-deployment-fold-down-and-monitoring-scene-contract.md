# 058 Speaker Deployment, Fold-Down, And Monitoring Scene Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/036-spatial-adapter-execution-contract.md`, `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`, `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first reusable Signal-owned speaker deployment, fold-down, and
monitoring-scene boundary so later immersive monitoring and deployment work
builds on one runtime-owned meaning instead of renderer-private speaker maps,
host-local monitor heuristics, or product-local immersive-console policy.

## Authority hierarchy

Speaker deployment, fold-down, and monitoring-scene meaning has one authority
chain:

1. this contract defines deployment class, fold-down policy, monitoring-scene
   class, monitoring-scene authority, and monitoring outcome
2. `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
   remains the authority for canonical layout, channel-role, and custom-layout
   fallback meaning that deployment and fold-down semantics must anchor to
3. `docs/contracts/036-spatial-adapter-execution-contract.md` remains the
   authority for baseline spatial adapter class, execution mode, target
   environment, and fallback posture that deployment work must widen rather
   than replace
4. `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`
   remains the authority for bed/object/mix-policy meaning that deployment and
   fold-down policy must compose with rather than reopen
5. `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`
   remains the authority for immersive object-rendering posture, room-policy
   class, room-policy authority, and immersive room outcome that monitoring and
   deployment policy must layer on top of
6. `signal-graph` owns node-local graph identity and any later structural
   monitoring metadata needed to stage deployment-aware or monitor-scene-aware
   execution
7. `signal-runtime` must own the typed execution, observation, diagnostic, and
   render receipts that expose deployment and monitoring truth to hosts and
   supervisor consumers
8. hosts, renderers, adapters, and downstream products may contribute concrete
   speaker calibration, device inventory, or UX, but they must not redefine
   deployment or monitoring-scene meaning once runtime-owned receipts exist

If a deployment or monitoring claim cannot be explained through this contract,
the closed multichannel, spatial, richer-spatial, immersive room-policy, and
runtime-owned topology receipts, it is not yet part of the shared monitoring
boundary.

## Existing anchors

Batch 7.1 freezes this contract on top of the current bounded implementation
anchors instead of pretending deployment-aware immersive runtime depth already
exists:

- `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`
  - canonical layout and channel-role meaning that deployment semantics must
    remain anchored to
- `docs/contracts/036-spatial-adapter-execution-contract.md`
  - baseline spatial execution and fallback meaning that monitoring-scene
    policy must widen rather than replace
- `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`
  - bed, object, mix-policy, and render-scope meaning that fold-down policy
    must obey
- `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`
  - immersive room-policy meaning that deployment and monitoring work must
    layer on top of instead of replacing
- `crates/signal-runtime/src/interfaces.rs`
  - current richer-spatial and immersive room-policy receipts, observation
    export, and offline-preview seams that later batches must widen
- `crates/signal-runtime/src/runtime.rs`
  - current graph planning, richer-spatial fallback, and immersive room-policy
    projection seams that define the bounded baseline this contract expands
    from

This contract does not claim full speaker deployment or monitoring-scene depth
already exists. It freezes the meaning later runtime and renderer work must
obey.

## Shared vocabulary

### Deployment class

A `deployment class` is the runtime-owned category of the speaker or endpoint
deployment Signal is trying to satisfy.

Batch 7.1 freezes this bounded family:

- `SourceLayoutDeployment`
- `ReferenceSpeakerDeployment`
- `MonitoringSpeakerDeployment`
- `PortableFoldDownDeployment`
- `FallbackDeployment`

Deployment class is stronger than channel count. It says whether Signal is
trying to preserve a known speaker deployment, monitor against one, or fall
back from it.

### Fold-down policy

A `fold-down policy` is the runtime-owned rule for how richer deployment or
monitoring material should collapse when the active deployment cannot be
preserved directly.

Batch 7.1 freezes this bounded family:

- `PreserveDeclaredDeployment`
- `FoldDownToReferenceBed`
- `FoldDownToStereoMonitoring`
- `FoldDownToPortablePreview`
- `BypassDeploymentPolicy`

Fold-down policy must stay explicit because later monitoring work cannot
depend on hidden renderer or host assumptions about where immersive material
collapses.

### Monitoring-scene class

A `monitoring-scene class` is the runtime-owned category of the active
monitoring view Signal is trying to satisfy.

Batch 7.1 freezes this bounded family:

- `NoMonitoringScene`
- `ReferenceScene`
- `FoldDownScene`
- `ConfidenceScene`
- `FallbackScene`

Monitoring-scene class is not a product UI label. It describes reusable
runtime-owned meaning for how monitoring intent is grouped.

### Monitoring-scene authority

`monitoring-scene authority` means where the currently active monitoring-scene
meaning is allowed to come from.

Batch 7.1 freezes this bounded family:

- `RuntimeDefault`
- `RuntimeDeclared`
- `HostForwarded`
- `RendererAdvisory`

This keeps the shared ownership line explicit. Hosts and renderers may forward
detail, but the shared monitoring boundary must still reduce to one
runtime-owned authority answer.

### Monitoring outcome

A `monitoring outcome` is the runtime-owned result when deployment and
monitoring-scene intent is projected onto the currently available room-policy,
renderer, and endpoint substrate.

Batch 7.1 freezes this bounded family:

- `MonitorDeclaredDeployment`
- `MonitorFoldedDownScene`
- `MonitorPortablePreview`
- `BypassMonitoringScene`
- `TerminalMonitoringFailure`

This outcome is distinct from immersive room outcome. It explains what
happened at the monitoring and deployment boundary, not only at the room-policy
boundary.

## Rules

### Rule 1: deployment and monitoring meaning must stay runtime-owned

Deployment class, fold-down policy, monitoring-scene class, monitoring-scene
authority, and monitoring outcome belong to runtime-owned receipts, not to
renderer-private speaker maps, host-local endpoint heuristics, or product-local
monitoring UX.

### Rule 2: deployment semantics must compose with the closed immersive seam

This contract widens the closed multichannel, spatial, richer-spatial, and
immersive room-policy seams. It must not replace canonical layout, bed/object
role, room-policy meaning, or immersive fallback truth with a second monitoring
taxonomy.

### Rule 3: renderer-private speaker detail stays advisory

Concrete speaker positions, calibration payloads, endpoint names, and
renderer-native monitor-scene metadata may exist internally, but the shared
boundary must remain grounded in runtime-owned deployment class, fold-down
policy, monitoring-scene class, authority, and outcome.

### Rule 4: live, offline, and diagnostic surfaces must converge

Later deployment work may stage rollout, but it must not create one monitoring
model for live execution, another for offline render, and a third for
observation or supervisor export.

### Rule 5: product room and monitor UX stay out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze immersive-console UX, room editors, speaker-layout design tools, or
consumer-facing monitoring workflows.

## Deferred scope

Batch 7.1 intentionally leaves these out:

- public supervisor-boundary proof surfaces
- renderer-capability negotiation or immersive export packaging
- room calibration, headphone virtualization, or renderer-specific speaker
  schemas
- product-local immersive monitoring or room-design UX

## Batch 7.1 outcome

Batch 7.1 freezes the first reusable deployment and monitoring authority line
for Signal:

- deployment class, fold-down policy, monitoring-scene class,
  monitoring-scene authority, and monitoring outcome are now explicit
  Signal-owned vocabulary
- later runtime execution can widen on top of the closed multichannel,
  spatial, richer-spatial, and immersive room-policy seams instead of
  inventing a parallel deployment or monitor-scene policy model
- unsupported deployment-aware monitoring paths and renderer gaps are now
  required to explain themselves through bounded monitoring outcome meaning
  rather than host-local heuristics or renderer-private interpretation

## Batch 7.2 outcome

Batch 7.2 turns this contract into a real runtime seam:

- `signal-runtime` now owns typed deployment and monitoring posture through
  `RuntimeDeploymentMonitoringSummary` on the shared spatial execution surface
- topology and offline render preview now count deployment-aware, folded-down,
  and fallback-monitoring spatial work directly from runtime-owned receipts
  rather than consumer-side recounting
- the stable local and server host edges now export the same bounded
  deployment and monitoring answers as the runtime-owned spatial seam

The bounded baseline is intentionally conservative:

- stereo balance-only paths still report no deployment-monitoring receipt
- the current surround fallback path reports `FallbackDeployment`,
  `FoldDownToReferenceBed`, `FallbackScene`, `RuntimeDefault`, and
  `BypassMonitoringScene`
- Batch 7.3 still has to prove the widened seam through the shared supervisor
  consumer boundary before this milestone is complete

## Batch 7.3 outcome

Batch 7.3 closes this contract through the existing shared spatial boundary:

- `signal-supervisor-tools` now treats deployment and monitoring posture as
  part of `signal.runtime.spatial-boundary` rather than leaving supervisor
  proof anchored at the earlier immersive-only seam
- the machine-readable boundary now points at this contract and names
  deployment-aware, folded-down, and fallback-monitoring topology, stage, and
  render-preview anchors directly
- the existing repo-owned `acceptance:spatial-boundary` lane now proves the
  full runtime, supervisor, and stable host-edge deployment and monitoring seam
  without inventing a renderer-private monitoring descriptor

This contract is now complete for its bounded goal:

- deployment and monitoring truth is contract-shaped, runtime-owned, and
  publicly inspectable
- stable host-edge and supervisor proof now converge on the same bounded seam
- renderer-capability negotiation, immersive export packaging, and deeper
  renderer-backed monitoring breadth remain explicitly later `g08` work

## Next Task

Continue `g08.008` with Batch 8.1 by freezing the first runtime-owned
renderer-capability negotiation and immersive export contract on top of the
closed deployment, fold-down, and monitoring-scene seam.
