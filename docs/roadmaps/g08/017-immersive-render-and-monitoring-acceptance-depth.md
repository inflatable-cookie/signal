# 017 - Immersive Render And Monitoring Acceptance Depth

Status: complete
Owner: core-product
Created: 2026-03-22
Depends on: g08.016
Vision tags: `IMMERSIVE`, `MONITORING`, `ACCEPTANCE`

## Problem

`g08.016` closes the bounded Linux live acceptance seam, but the shared
consumer proof for immersive room policy, deployment monitoring, and renderer
export behavior is still fragmented across earlier spatial, immersive,
deployment, and renderer boundaries.

Without one explicit acceptance milestone here, later immersive work risks
drifting into renderer-private capability shells, monitoring-scene UX policy,
or ad hoc rerun coverage that shared consumers cannot rely on.

## Goals

- [ ] freeze one shared acceptance target for immersive render and monitoring
      behavior
- [ ] keep the acceptance seam grounded in existing runtime-owned spatial,
      immersive room-policy, deployment-monitoring, and renderer-export
      receipts
- [ ] avoid renderer-private or product-local immersive workflow glue becoming
      the shared proof surface

## Non-Goals

- [ ] no product-local immersive authoring UI or monitoring-scene editor
- [ ] no vendor-specific renderer packaging or distribution workflow as the
      shared acceptance surface

## Execution Plan

### Batch 17.1 - Immersive Acceptance Contract

- [x] freeze the shared immersive render and monitoring acceptance contract
- [x] define the mandatory runtime, supervisor, and stable host-edge proof
      spine explicitly

### Batch 17.2 - Acceptance Descriptor And Task

- [x] wire the first repo-owned descriptor and acceptance lane for the shared
      immersive seam
- [x] keep optional renderer-native depth explicit rather than folding it into
      the mandatory shared contract

### Batch 17.3 - Consumer Proof Closure

- [x] prove the widened immersive render and monitoring acceptance seam through
      shared runtime, supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] immersive render and monitoring acceptance are repo-owned and inspectable
- [x] renderer-private and product-local immersive detail stays bounded and
      typed
- [x] later preview, device, and integrated acceptance work can build on one
      explicit immersive acceptance seam

## Risks And Mitigations

- Risk: immersive acceptance drifts into renderer-private capability policy or
  product-specific monitoring UX.
- Mitigation: freeze one shared acceptance contract before widening broader
  immersive rerun or export depth.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused validation after descriptor/task changes land
- [x] record the next milestone step explicitly

## Batch 17.1 Outcome

- `g08` now has a frozen shared immersive render and monitoring acceptance
  contract in
  `docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md`
  instead of leaving grouped immersive proof fragmented across earlier spatial,
  room-policy, deployment-monitoring, and renderer-export seams
- the shared acceptance lane is now required to compose through public runtime
  receipts, supervisor export, and both stable host edges rather than
  renderer-private capability shells or product-local monitoring workflows
- the grouped descriptor, Effigy acceptance lane, and broader advisory versus
  deferred immersive-depth policy remain explicitly deferred until Batch 17.2
  and Batch 17.3

## Batch 17.2 Outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.immersive-acceptance-lane` descriptor so the shared
  immersive render and monitoring seam is inspectable without reading the
  broader spatial boundary by hand
- Effigy now owns one runnable `effigy acceptance:immersive-acceptance-lane`
  task that composes the already-closed spatial boundary proof with the new
  grouped immersive descriptor into one bounded shared lane
- broader renderer-native reruns, richer monitoring-scene variants, and
  workflow-native immersive depth remain explicitly advisory or deferred
  instead of being smuggled into the required path

## Batch 17.3 Outcome

- the shared immersive acceptance lane now has one grouped consumer-facing
  supervisor export proof instead of only a grouped descriptor, so immersive
  room-policy, deployment-monitoring, and renderer-export truth are proven
  consumable together on one shared path
- `effigy acceptance:immersive-acceptance-lane` now composes the existing
  spatial boundary proof, the grouped export proof, and the machine-readable
  descriptor into one reusable acceptance lane
- `g08.017` is now complete, and the next `g08` queue is control-surface and
  preview workflow acceptance depth

## Completion

`g08.017` is complete. The bounded immersive render and monitoring acceptance
seam is now frozen, grouped, proved through one shared consumer path, and
ready for later workflow and integrated acceptance work to build on.

## Next Task

Continue `g08.018` with Batch 18.1 by freezing the shared control-surface and
preview workflow acceptance contract on top of the closed advanced-hardware,
workflow, preview-transform, and preview-device consumer seams.
