# 007 - Speaker Deployment, Fold-Down, And Monitoring Scene Depth

Status: complete
Owner: core-product
Created: 2026-03-19
Depends on: g08.006
Vision tags: `IMMERSIVE`, `MONITORING`, `DEPLOYMENT`

## Problem

`g08.006` closes the bounded immersive room-policy seam, but speaker
deployment, fold-down, and monitoring-scene truth still sits below the shared
runtime surface. Without a runtime-owned contract here, monitoring and
deployment behavior will drift back into renderer-private speaker maps,
host-local monitor policy, or product-local immersive-console logic.

## Goals

- [ ] freeze one runtime-owned authority line for speaker deployment, fold-down, and monitoring scenes
- [ ] keep deployment and monitoring meaning composable with the closed immersive room-policy substrate
- [ ] avoid renderer-private speaker or monitor policy becoming shared truth

## Non-Goals

- [ ] no product-local monitoring mixer, room editor, or speaker layout UX
- [ ] no final renderer-capability negotiation or immersive export packaging in this milestone

## Execution Plan

### Batch 7.1 - Deployment And Monitoring Contract

- [x] freeze runtime-owned speaker deployment, fold-down, and monitoring-scene meaning
- [x] define shared runtime versus renderer-private authority explicitly

### Batch 7.2 - Runtime Monitoring Baseline

- [x] materialize the first runtime-owned deployment, fold-down, and monitoring-scene receipts
- [x] align stable host-edge export with the same bounded model

### Batch 7.3 - Consumer Proof

- [x] prove the widened deployment and monitoring seam through shared runtime,
      supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] speaker deployment, fold-down, and monitoring-scene posture is runtime-owned and inspectable
- [x] renderer-private and host-local monitor detail stays bounded and typed
- [x] later immersive export and monitoring work can build on one explicit deployment authority line

## Risks And Mitigations

- Risk: monitoring and deployment depth drifts into renderer-private speaker maps or host-local monitor policy.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after the runtime baseline lands
- [x] record the next milestone step explicitly

## Batch 7.1 Outcome

Batch 7.1 freezes the first reusable deployment and monitoring contract in
`docs/contracts/058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`.

That contract layers deployment class, fold-down policy, monitoring-scene
class, monitoring-scene authority, and monitoring outcome on top of the closed
multichannel, spatial, richer-spatial, and immersive room-policy seams instead
of letting monitoring meaning drift into renderer-private speaker maps or
host-local endpoint policy.

It now makes the authority line explicit:

- `032` remains the canonical layout and channel-role authority instead of
  being reopened as a generic speaker deployment surface
- `036`, `037`, and `057` remain the spatial, richer-spatial, and immersive
  room-policy authorities, so deployment and monitoring semantics must compose
  with those seams instead of replacing them
- Batch 7.2 now has one bounded contract target for runtime-owned deployment,
  fold-down, and monitoring-scene receipts before consumer proof widens in
  Batch 7.3

## Batch 7.2 Outcome

Batch 7.2 lands the first runtime-owned deployment and monitoring receipt seam
 inside `signal-runtime` instead of leaving deployment class, fold-down
 posture, and monitoring-scene truth as contract-only meaning.

What now exists on the shared runtime surface:

- `RuntimeSpatialExecutionSummary` carries bounded
  `deployment_monitoring` truth alongside the already-closed immersive
  room-policy summary
- `RuntimeExecutionTopologySummary` and
  `RuntimeOfflineRenderChainDependencyPreview` now count deployment-aware,
  folded-down, and fallback-monitoring spatial nodes or stages directly from
  runtime-owned receipts
- the focused public runtime and stable host-edge proofs now assert the same
  fallback deployment and monitoring answers, so consumers do not need
  renderer-private or host-local monitor policy reconstruction

This keeps Batch 7.2 meaningful but bounded:

- the shared runtime seam now exposes deployment and monitoring posture for the
  current fallback surround path
- stable host-edge export is aligned to that same bounded model
- consumer-facing supervisor proof still belongs to Batch 7.3

## Batch 7.3 Outcome

Batch 7.3 closes `g08.007` by widening the existing shared spatial consumer
boundary instead of creating a second monitoring-only acceptance shell.

What now closes through one reusable seam:

- `signal-supervisor-tools` `spatial-boundary` now points at contract `058`
  instead of stopping at the earlier immersive room-policy contract
- the machine-readable supervisor boundary now names deployment-aware,
  folded-down, and fallback-monitoring topology and render-preview anchors
  alongside `deployment_monitoring` on node and stage receipts
- the existing `acceptance:spatial-boundary` lane now proves the full runtime,
  supervisor, and stable host-edge deployment and monitoring seam without
  introducing renderer-private monitoring policy as shared truth

This leaves the milestone cleanly closed:

- deployment class, fold-down policy, monitoring-scene class and authority,
  and monitoring outcome are now contract-shaped, runtime-owned, publicly
  inspectable, and supervisor-described
- renderer-capability negotiation and immersive export packaging remain the
  next milestone instead of being left implicit inside `g08.007`

## Next Task

Continue `g08.008` with Batch 8.1 by freezing the first runtime-owned
renderer-capability negotiation and immersive export contract on top of the
closed deployment, fold-down, and monitoring-scene seam.
