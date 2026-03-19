# 005 - Spatial Adapter Execution Baseline

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.001, g07.003
Vision tags: `SPATIAL`, `MULTICHANNEL`, `EXECUTION`

## Problem

Chorus already defines spatial adapter intent, but Signal still needs a first
credible runtime-owned execution baseline for those adapters.

## Goals

- [ ] implement the first real spatial adapter execution baseline in Signal
- [ ] keep spatial behavior aligned with multichannel and routing substrate
- [ ] expose host-visible runtime spatial state without host-local reinterpretation

## Non-Goals

- [ ] no product-specific spatial UI variants
- [ ] no full object-audio ecosystem breadth yet

## Execution Plan

### Batch 5.1 - Spatial Execution Contract

- [x] align existing spatial-adapter semantics with runtime execution meaning
- [x] define fallback behavior for unsupported layouts and adapters

### Batch 5.2 - Runtime Baseline

- [x] implement the first credible spatial adapter path
- [x] expose runtime-owned spatial observation and diagnostics

### Batch 5.3 - Focused Proof

- [x] add focused proofs for spatial execution and fallback behavior

## Acceptance Criteria

- [x] Signal has a real spatial adapter execution baseline
- [x] spatial behavior stays aligned with multichannel and routing truth
- [x] hosts observe spatial state through one reusable runtime vocabulary

## Risks And Mitigations

- Risk: spatial semantics stay model-only and never become executable.
- Mitigation: require runtime proof and fallback behavior before expansion.

## Evidence Requirements

- [x] log each meaningful spatial tranche
- [x] run focused spatial execution validation
- [ ] record deferred spatial breadth explicitly

## Batch 5.1 Outcome

Batch 5.1 froze the first reusable Signal-owned spatial execution boundary in
`docs/contracts/036-spatial-adapter-execution-contract.md`.

Signal now has one bounded shared vocabulary for:

- spatial adapter class
- spatial execution mode
- target environment
- control family
- activation policy
- fallback outcome

That gives the next runtime batch one fixed target for node-owned spatial
execution on top of the already-closed multichannel, sidechain, multi-bus, and
complex plugin-I/O routing seams instead of letting spatial behavior drift back
into product-local pan policy or adapter-private renderer logic.

## Batch 5.2 Outcome

Batch 5.2 turned that frozen contract into real runtime-owned receipts across
execution, render, and observation surfaces.

Signal now carries one typed spatial receipt family through:

- planned-node and execution-topology summaries
- plugin-chain stage snapshots used by live runtime observation
- offline-render chain dependency preview and stage receipts
- host-visible shared supervisor reports and JSON export paths that now expose
  active versus bypassed spatial state directly

The first executable baseline is intentionally narrow but real: stereo balance
is now a bounded spatial adapter path, while non-stereo layouts surface
explicit runtime-owned `BypassSpatialProcessing` fallback instead of pretending
surround or renderer depth already exists.

## Batch 5.3 Outcome

Batch 5.3 closes the public consumer seam for the bounded spatial baseline.

Spatial execution and fallback receipts are now proven through:

- public runtime observation and offline-render preview surfaces
- stable local and server host-edge supervisor reports
- a machine-readable `signal.runtime.spatial-boundary` descriptor and repo-owned
  `effigy acceptance:spatial-boundary` task

This means downstream consumers can now inspect active versus bypassed spatial
execution, target environment, and explicit `BypassSpatialProcessing` fallback
without reconstructing speaker policy from adapter-local or host-local state.

## Next Task

Continue `g07.006` with Batch 6.2 by materializing runtime-owned surround-bed,
object-role, mix-policy, render-scope, and expanded-fallback receipts across
execution, render, and observation surfaces without reopening host-local or
renderer-local spatial ownership.
