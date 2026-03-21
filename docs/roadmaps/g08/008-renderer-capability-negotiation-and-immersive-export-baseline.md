# 008 - Renderer-Capability Negotiation And Immersive Export Baseline

Status: complete
Owner: core-product
Created: 2026-03-19
Depends on: g08.007
Vision tags: `IMMERSIVE`, `EXPORT`, `RENDERER`

## Problem

`g08.007` closes the bounded deployment, fold-down, and monitoring-scene seam,
but renderer-capability negotiation and immersive export packaging are still
explicitly deferred. Without a runtime-owned contract here, later export and
renderer capability work will drift back into renderer-private negotiation
tables, host-local export glue, or product-local immersive packaging rules.

## Goals

- [ ] freeze one runtime-owned authority line for renderer capability and immersive export
- [ ] keep renderer negotiation and export meaning composable with the closed spatial, room-policy, and monitoring seams
- [ ] avoid renderer-private capability tables or host-local packaging becoming shared truth

## Non-Goals

- [ ] no product-local immersive export UX or release workflow
- [ ] no final downstream distribution or publication lane in this milestone

## Execution Plan

### Batch 8.1 - Renderer Capability And Export Contract

- [x] freeze runtime-owned renderer-capability negotiation and immersive export meaning
- [x] define shared runtime versus renderer-private authority explicitly

### Batch 8.2 - Runtime Capability And Export Baseline

- [x] materialize the first runtime-owned renderer-capability and immersive export receipts
- [x] align stable host-edge export with the same bounded model

### Batch 8.3 - Consumer Proof

- [x] prove the widened renderer-capability and immersive export seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] renderer capability and immersive export posture is runtime-owned and inspectable
- [x] renderer-private and host-local export detail stays bounded and typed
- [x] later immersive acceptance and export work can build on one explicit renderer capability authority line

## Risks And Mitigations

- Risk: renderer capability and immersive export depth drifts into renderer-private negotiation tables or host-local export glue.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after the runtime baseline lands
- [x] record the next milestone step explicitly

## Batch 8.1 Outcome

Batch 8.1 freezes the first reusable renderer-capability negotiation and
immersive export contract in
`docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md`.

That contract layers renderer-capability posture, capability authority,
immersive export class, export authority, and export outcome on top of the
closed spatial, room-policy, and deployment-monitoring seams instead of
letting immersive export meaning drift into renderer-private capability tables
or host-local packaging rules.

It now makes the authority line explicit:

- `036`, `037`, `057`, and `058` remain the spatial, richer-spatial,
  immersive room-policy, and deployment-monitoring authorities, so renderer
  negotiation and export semantics must compose with those seams instead of
  replacing them
- Batch 8.2 now has one bounded contract target for runtime-owned renderer
  capability and immersive export receipts before consumer proof widens in
  Batch 8.3

## Batch 8.2 Outcome

Batch 8.2 lands the first runtime-owned renderer capability and immersive
export receipt seam inside `signal-runtime` instead of leaving renderer
compatibility and export posture at the contract layer only.

What now exists on the shared runtime surface:

- `RuntimeSpatialExecutionSummary` carries bounded `renderer_export` truth
  alongside the already-closed immersive room-policy and deployment-monitoring
  summaries
- `RuntimeExecutionTopologySummary` and
  `RuntimeOfflineRenderChainDependencyPreview` now count renderer-capability,
  negotiated-renderer, immersive-export, and fallback-export spatial work
  directly from runtime-owned receipts
- the focused public runtime and stable host-edge proofs now assert the same
  fallback renderer negotiation and immersive export answers, so consumers do
  not need renderer-private capability tables or host-local export
  reconstruction

This keeps Batch 8.2 meaningful but bounded:

- the shared runtime seam now exposes renderer capability and export posture
  for the current fallback surround path
- stable host-edge export is aligned to that same bounded model
- consumer-facing supervisor proof still belongs to Batch 8.3

## Batch 8.3 Outcome

Batch 8.3 closes the widened renderer-capability and immersive export seam
through the existing shared `signal.runtime.spatial-boundary` consumer surface
instead of opening a second renderer-only acceptance lane.

What now exists on the shared consumer boundary:

- `signal-supervisor-tools` points the spatial boundary at contract `059`
  and explicitly describes renderer-capability, negotiated-renderer,
  immersive-export, and fallback-export topology plus render-preview anchors
- the machine-readable supervisor descriptor now carries
  `spatial_execution.renderer_export` as part of the same runtime-owned spatial
  seam that already covers room-policy and deployment-monitoring truth
- the existing `effigy acceptance:spatial-boundary` lane now closes the
  bounded renderer/export consumer seam through public runtime proof, stable
  local host-edge proof, stable server host-edge proof, and the shared
  supervisor descriptor

This keeps the closure meaningful but still bounded:

- the closed seam now proves one explicit runtime-owned renderer capability
  authority line for the current fallback surround path
- consumers do not need renderer-private capability tables or host-local
  export shells to inspect immersive export posture
- deeper renderer-backed execution, vendor package schemas, and publication
  workflows remain later `g08` work

## Completion

`g08.008` is complete. The bounded renderer-capability negotiation and
immersive export seam is now frozen, runtime-owned, proved through the shared
consumer boundary, and ready for later renderer or export-depth milestones to
build on.

## Next Task

Continue `g08.009` with Batch 9.1 by freezing the first runtime-owned advanced
control-surface display, motor, and haptic transport contract on top of the
closed controller-expression, control-surface, advanced-hardware, and richer
workflow seams.
