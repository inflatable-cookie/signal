# 002 - Sidechain Routing And Secondary-Input Execution Depth

Status: complete
Owner: core-product
Created: 2026-03-13
Depends on: g07.001
Vision tags: `ROUTING`, `PLUGINS`, `EXECUTION`

## Problem

Chorus already treats sidechain routing as first-class authority intent, but
Signal still needs one runtime-owned meaning for secondary inputs, routing
identity, and failure or fallback behavior.

## Goals

- [ ] define runtime-owned sidechain and secondary-input semantics
- [ ] support sidechain-capable routing without host-local reconstruction
- [ ] keep plugin, graph, and render surfaces aligned on one secondary-input model

## Non-Goals

- [ ] no product-specific sidechain UX yet
- [ ] no broad multi-bus expansion beyond what this contract needs

## Execution Plan

### Batch 2.1 - Secondary-Input Contract

- [x] define sidechain source, target, and fallback meaning
- [x] align authority routing intent with runtime execution surfaces

### Batch 2.2 - Runtime Execution Depth

- [x] implement secondary-input execution and runtime observation depth
- [x] keep plugin and render paths aligned with the new sidechain meaning

### Batch 2.3 - Focused Proof

- [x] add focused proofs for sidechain routing, fallback, and failure behavior

## Acceptance Criteria

- [x] Signal has explicit secondary-input and sidechain execution semantics
- [ ] later complex plugin-I/O and spatial work can rely on the same routing truth
- [x] hosts observe sidechain state without inventing separate models

## Risks And Mitigations

- Risk: sidechain meaning forks between live and offline execution.
- Mitigation: require one runtime-owned contract across both paths.

## Evidence Requirements

- [ ] log each meaningful sidechain tranche
- [ ] run focused routing and execution validation
- [ ] record deferred sidechain breadth explicitly

## Batch 2.1 Outcome

Batch 2.1 freezes the first reusable sidechain and secondary-input routing
contract in `docs/contracts/033-sidechain-routing-and-secondary-input-execution-contract.md`.
That contract makes sidechain source, target, attachment policy, and fallback
outcome Signal-owned routing meaning instead of leaving later live, render,
multi-bus, or plugin-format work to infer sidechain behavior from host-local
patch wiring.

It also gives Batch 2.2 one fixed target:

- secondary-input identity must now stay explicit across graph, runtime, and
  later render surfaces
- sidechain source and target are now bounded typed vocabulary rather than
  informal topology hints
- fallback outcomes are now runtime-owned policy, not host convenience logic
- live and offline sidechain paths are now required to converge on one shared
  model instead of forking later

This keeps `g07.002` on the broad routing goal and prevents runtime work from
reopening the meaning question.

## Batch 2.2 Outcome

Batch 2.2 turns the frozen sidechain contract into runtime-owned receipts
instead of leaving it as routing prose.

`signal-runtime` now carries:

- typed secondary-input contract projection through
  `GraphNodeBufferContractProjection.secondary_input`
- runtime-planned and execution-topology sidechain route summaries with
  explicit source, target, attachment policy, and fallback outcome
- plugin-chain stage summaries that keep the same sidechain meaning while
  rebasing the target onto the plugin-facing execution boundary
- offline render chain-dependency previews that carry the same secondary-input
  route family as render-target receipts rather than reconstructing sidechain
  needs from later host patching

Focused runtime proof now covers both live and offline depth:

- live topology and plugin-chain sidechain routing plus fallback receipts
- offline render contract preview carrying aligned sidechain dependency receipts
- regression coverage that existing send-return and earlier offline preview
  surfaces still behave after the widened routing family landed

This keeps the work broad and avoids churn:

- one secondary-input receipt family now spans planning, execution topology,
  plugin stages, and offline dependency preview
- broader public runtime, supervisor, and stable host-edge proof is still
  intentionally deferred to Batch 2.3 rather than partially claimed here
- richer multi-bus, spatial, or format-specific sidechain breadth is still
  outside this milestone

## Batch 2.3 Outcome

Batch 2.3 closes the shared sidechain consumer seam instead of leaving the new
receipt family as runtime-internal depth only.

The proof spine now includes:

- public runtime proof that sidechain source, target, attachment policy, and
  fallback meaning remain consumable from shared runtime reports
- stable local and server host-edge proof that `supervisor_report()` forwards
  the same secondary-input routing and plugin-stage receipts without host-local
  reinterpretation
- a machine-readable `signal.runtime.sidechain-boundary` descriptor in
  `signal-supervisor-tools`
- a repo-owned `effigy acceptance:sidechain-boundary` task that keeps the
  runtime, host-edge, and descriptor proof surfaces aligned

This closes `g07.002` on a real shared boundary:

- sidechain truth is now Signal-owned from graph contract through runtime,
  supervisor, and stable host-edge consumption
- later multi-bus, complex plugin-I/O, and spatial milestones can extend the
  same routing family instead of reopening secondary-input semantics
- broader routing breadth is still deferred explicitly rather than hidden

## Next Task

Continue `g07.003` with Batch 3.1 by freezing the runtime-owned multi-bus
graph execution and auxiliary-topology contract on top of the now-closed
multichannel and sidechain routing boundaries.
