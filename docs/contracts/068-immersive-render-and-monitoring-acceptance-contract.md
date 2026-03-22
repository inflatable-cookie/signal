# 068 Immersive Render And Monitoring Acceptance Contract

Status: complete
Owner: core-product
Updated: 2026-03-22
Related contracts: `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`, `docs/contracts/058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`, `docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared immersive render and monitoring acceptance contract for
`g08.017` so Signal can prove immersive room-policy, deployment-monitoring,
and renderer-export behavior through one repo-owned evidence lane instead of
isolated spatial boundary proofs, renderer-private capability shells, or
product-local monitoring workflows.

## Authority hierarchy

Immersive render and monitoring acceptance has one authority chain:

1. the closed immersive contracts define what Signal is allowed to claim about:
   - immersive room-policy posture and room outcome
   - deployment class, fold-down policy, and monitoring outcome
   - renderer-capability negotiation and immersive export posture
2. `signal-runtime` owns the typed receipts those claims must compose from:
   - `RuntimeSpatialExecutionSummary`
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeExecutionTopologySummary`
   - `RuntimeOfflineRenderChainDependencyPreview`
3. shared host crates own bounded local and server export for the same spatial
   receipt families, but they do not own immersive acceptance meaning
4. `signal-supervisor-tools` must own the machine-readable descriptors that
   explain:
   - which immersive render and monitoring families are part of the shared
     acceptance lane
   - which runtime, supervisor, and stable host-edge proofs are required
   - which broader renderer-native or workflow-native depth remains advisory or
     deferred
5. Effigy tasks must own the runnable grouping policy for the shared lane:
   - which already-closed immersive and spatial boundary tasks are required
   - which grouped checks become the mandatory `g08.017` acceptance path
   - which broader immersive rerun or renderer-native depth remains non-blocking
6. downstream consumers may archive or rerun the outputs, but they must not
   become the authority for what Signal considers the canonical immersive
   render and monitoring acceptance seam

If an immersive render or monitoring acceptance claim cannot be explained
through the closed contracts above, typed runtime receipts, supervisor-tools
descriptors, and repo-owned Effigy tasks, it is not yet part of the shared
Signal acceptance boundary.

## Existing acceptance anchors

This contract builds on the currently closed bounded proof tasks and
descriptors:

- `effigy acceptance:spatial-boundary`
- `cargo run -p signal-supervisor-tools -- --describe-spatial-boundary --format=json`
- the runtime-owned `immersive_room_policy`, `deployment_monitoring`, and
  `renderer_export` seams already carried on the shared spatial boundary

Batch 17.1 does not claim these already form one grouped acceptance lane. It
freezes how they must be composed and widened in later `g08.017` batches.

## Shared vocabulary

### Immersive render acceptance

`immersive render acceptance` means one repo-owned, machine-readable evidence
lane that proves Signal's immersive room-policy, deployment-monitoring, and
renderer-export receipts remain consumable together across the bounded spatial
consumer seam already modeled by Signal.

It is not a renderer certification program, not a monitor-scene UX promise,
and not a product-local authoring workflow.

### Monitoring acceptance

`monitoring acceptance` means the bounded proof that deployment-aware,
folded-down, and fallback monitoring outcomes remain consumable through shared
runtime, supervisor, and stable host-edge receipts when immersive render
intent is projected onto the current runtime-owned spatial substrate.

### Required acceptance evidence

`required acceptance evidence` means evidence that must remain green for
Signal to claim the shared `g08.017` acceptance lane exists.

### Advisory acceptance evidence

`advisory acceptance evidence` means broader immersive reruns, renderer-native
depth, or richer monitoring variants that improve confidence but do not yet
block the bounded lane.

### Deferred acceptance evidence

`deferred acceptance evidence` means known and useful immersive scenario depth
that remains outside the bounded lane because it is not yet stable enough,
portable enough, or appropriately Signal-owned.

## Integrated scenario families

Batch 17.1 freezes four scenario families for later implementation.

### Family 1: Room-policy and immersive render continuity

This family proves the shared lane can surface coherent immersive truth across:

- room-policy posture and outcome
- object-rendering and fallback posture
- renderer-export interaction with the same bounded spatial seam

### Family 2: Deployment, fold-down, and monitoring coherence

This family proves the shared lane spans the widened deployment seam:

- deployment class and fold-down policy
- monitoring-scene authority and outcome
- bounded fallback-monitoring and portable-preview answers

### Family 3: Cross-surface immersive coherence

This family proves the shared lane can surface one bounded truth across:

- public runtime receipts
- supervisor export or descriptors
- stable local host-edge export
- stable server host-edge export

### Family 4: Shared grouped immersive acceptance export

This family proves the bounded lane can expose one machine-readable grouped
descriptor or acceptance task that spans more than one family above instead of
only re-listing isolated spatial boundary-local tasks.

## Required versus advisory versus deferred policy

Batch 17.1 freezes a three-tier policy.

### Required

The later `g08.017` shared lane must require:

- the already-closed immersive room-policy, deployment-monitoring, and
  renderer-export proof spine as building blocks
- at least one grouped descriptor or acceptance task that spans immersive
  render, monitoring, and renderer-export receipts together
- proof through public runtime, supervisor, and both stable host edges

### Advisory

The later lane may report but not block on:

- broader renderer-native or monitor-scene reruns
- richer immersive authoring or export-adjacent confidence passes
- repeated-run depth that stays useful but is not yet bounded

### Deferred

The shared lane must keep explicitly deferred:

- exhaustive renderer vendor certification or package-schema matrices
- product-local monitor-scene editors, immersive consoles, or authoring UX
- broader cross-generation integrated acceptance that belongs to later `g08`
  milestones
- renderer-native failure tooling that does not yet collapse cleanly into
  Signal-owned receipts

## Rules

### Rule 1: the lane stays additive over closed immersive contracts

Later grouped acceptance may combine room-policy, deployment-monitoring, and
renderer-export surfaces, but it must stay a proof over already-closed
Signal-owned contracts instead of inventing a second immersive authority.

### Rule 2: the shared lane must stay machine-readable

The acceptance seam must not degrade into prose-only tranche logs or human
memory. Later batches should expose descriptors, supervisor JSON, or explicit
Effigy grouping that explains what the lane covers and why it passes.

### Rule 3: renderer-private and product-local glue remain out of bounds

Renderer-private capability shells, monitor-scene UX glue, and host-private
immersive helpers may inform scenario setup later, but they must not become
the shared acceptance surface.

### Rule 4: required, advisory, and deferred depth stay explicit

Signal must not hide unstable or expensive immersive depth inside the bounded
lane. If a scenario blocks the shared claim, it must be marked `required`. If
it is useful but non-blocking, it must stay `advisory`. If it is known but not
yet bounded or stable, it must stay `deferred`.

### Rule 5: runtime and stable host-edge truth must align

The shared lane must prove that public runtime receipts, supervisor export, and
both stable host edges tell the same bounded immersive render and monitoring
story instead of allowing one host path or one renderer posture to define a
special case.

## Deferred scope

Batch 17.1 intentionally leaves these out:

- the concrete grouped descriptor and Effigy task names for the later lane
- exact immersive rerun or renderer-native permutations for advisory depth
- preview, device-workflow, Linux live, or generation-level integrated
  acceptance that belongs to later `g08` milestones
- renderer package publication or product-local monitoring workflow policy

## Batch 17.1 outcome

Batch 17.1 freezes the shared acceptance policy shape for immersive render and
monitoring depth:

- Signal now has one explicit authority line for grouped immersive acceptance
  instead of relying on isolated spatial-boundary proof only
- later `g08.017` implementation is forced to build on the closed room-policy,
  deployment-monitoring, renderer-export, and spatial consumer seams instead of
  renderer-private capability shells or product-local monitoring UX
- Batch 17.2 can now focus on materializing one grouped descriptor and task
  instead of reopening what the shared immersive acceptance lane means

## Batch 17.2 outcome

Batch 17.2 materializes the first repo-owned grouped acceptance seam for
immersive render and monitoring depth:

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.immersive-acceptance-lane` descriptor instead of leaving
  grouped immersive proof implicit under the older shared spatial boundary
- Effigy now owns one runnable `effigy acceptance:immersive-acceptance-lane`
  task that composes the already-closed spatial boundary proof with the grouped
  immersive descriptor
- advisory and deferred renderer-native or workflow-native depth remain
  explicit rather than collapsing into the mandatory shared lane

## Batch 17.3 outcome

Batch 17.3 closes the widened immersive render and monitoring acceptance seam
through one grouped consumer-facing supervisor export proof on top of the
repo-owned immersive lane.

- `signal-supervisor-tools` now proves one supervisor export can carry
  immersive room-policy, deployment-monitoring, and renderer-export truth
  together instead of only composing the broader spatial boundary descriptor
- `effigy acceptance:immersive-acceptance-lane` now runs the grouped export
  proof, the machine-readable descriptor, and the already-closed spatial
  boundary proof as one reusable acceptance lane
- the shared claim stays additive over the closed immersive contracts and
  typed runtime receipts instead of opening a renderer-private acceptance shell
  or a workflow-local monitoring model

## Next Task

Continue `g08.018` with Batch 18.1 by freezing the shared control-surface and
preview workflow acceptance contract on top of the closed advanced-hardware,
workflow, preview-transform, and preview-device consumer seams.
