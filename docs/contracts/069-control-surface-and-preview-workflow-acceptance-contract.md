# 069 Control-Surface And Preview Workflow Acceptance Contract

Status: complete
Owner: core-product
Updated: 2026-03-22
Related contracts: `docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`, `docs/contracts/060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`, `docs/contracts/062-preview-output-routing-audition-sink-and-low-latency-device-policy-contract.md`, `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared control-surface and preview workflow acceptance
contract for `g08.018` so Signal can prove bounded controller workflow and
preview workflow behavior through one repo-owned evidence lane instead of
isolated advanced-hardware and preview boundary proofs, device-private
workflow shells, or browser-local queue policy.

## Authority hierarchy

Control-surface and preview workflow acceptance have one authority chain:

1. the closed workflow contracts define what Signal is allowed to claim about:
   - control-surface scene mapping, feedback-page posture, and safe action
     graph behavior
   - advanced display, motor, and haptic feedback posture
   - preview-output routing, audition-sink ownership, and low-latency device
     policy
   - preview-browser queue, audition orchestration, and transform scheduling
2. `signal-runtime` owns the typed receipts those claims must compose from:
   - `RuntimeAdvancedHardwareSnapshot`
   - `RuntimePreviewTransformServiceSnapshot`
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
3. shared host crates own bounded local and server export for the same
   workflow receipt families, but they do not own workflow acceptance meaning
4. `signal-supervisor-tools` must own the machine-readable descriptors that
   explain:
   - which control-surface and preview workflow families are part of the
     shared acceptance lane
   - which runtime, supervisor, and stable host-edge proofs are required
   - which broader device-native or browser-native workflow depth remains
     advisory or deferred
5. Effigy tasks must own the runnable grouping policy for the shared lane:
   - which already-closed control-surface and preview boundary tasks are
     required building blocks
   - which grouped checks become the mandatory `g08.018` acceptance path
   - which broader workflow reruns or integrated UI depth remain non-blocking
6. downstream consumers may archive or rerun the outputs, but they must not
   become the authority for what Signal considers the canonical shared
   control-surface and preview workflow acceptance seam

If a control-surface or preview workflow acceptance claim cannot be explained
through the closed contracts above, typed runtime receipts, supervisor-tools
descriptors, and repo-owned Effigy tasks, it is not yet part of the shared
Signal acceptance boundary.

## Existing acceptance anchors

This contract builds on the currently closed bounded proof tasks and
descriptors:

- `effigy acceptance:advanced-hardware-boundary`
- `effigy acceptance:preview-transform-boundary`
- `cargo run -p signal-supervisor-tools -- --describe-advanced-hardware-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-preview-transform-boundary --format=json`
- the runtime-owned `scene_mapping`, `feedback_pages`, `safe_action_graph`,
  `preview_device_policy`, and `preview_workflow` seams already carried on the
  shared advanced-hardware and preview-transform boundaries

Batch 18.1 does not claim these tasks already form one grouped acceptance
lane. It freezes how they must be composed and widened in later `g08.018`
batches.

## Shared vocabulary

### Control-surface and preview workflow acceptance

`control-surface and preview workflow acceptance` means one repo-owned,
machine-readable evidence lane that proves Signal's bounded controller
workflow and preview workflow receipts remain consumable together across the
shared advanced-hardware and preview-transform consumer seams already modeled
by Signal.

It is not a device certification program, not a browser queue editor promise,
and not a product-local controller page workflow.

### Required acceptance evidence

`required acceptance evidence` means evidence that must remain green for
Signal to claim the shared `g08.018` acceptance lane exists.

### Advisory acceptance evidence

`advisory acceptance evidence` means broader device-native reruns,
preview-browser confidence passes, or integrated workflow checks that improve
confidence but do not yet block the bounded lane.

### Deferred acceptance evidence

`deferred acceptance evidence` means known and useful workflow depth that
remains outside the bounded lane because it is not yet stable enough, portable
enough, or appropriately Signal-owned.

## Integrated scenario families

Batch 18.1 freezes four scenario families for later implementation.

### Family 1: Control-surface workflow coherence

This family proves the shared lane can surface coherent controller workflow
truth across:

- scene-mapping posture
- feedback-page posture and bounded page class
- safe action graph posture, authority, and outcome
- advanced display, motor, and haptic readiness where already modeled by the
  shared advanced-hardware seam

### Family 2: Preview workflow coherence

This family proves the shared lane spans the widened preview seam:

- preview-output routing and audition-sink ownership
- low-latency preview-device policy
- preview-browser queue posture
- media audition orchestration continuity
- transform-scheduling posture, authority, and outcome

### Family 3: Cross-surface workflow coherence

This family proves the shared lane can surface one bounded truth across:

- public runtime receipts
- supervisor export or descriptors
- stable local host-edge export
- stable server host-edge export

### Family 4: Shared grouped workflow acceptance export

This family proves the bounded lane can expose one machine-readable grouped
descriptor or acceptance task that spans controller workflow and preview
workflow together instead of only re-listing isolated boundary-local tasks.

## Required versus advisory versus deferred policy

Batch 18.1 freezes a three-tier policy.

### Required

The later `g08.018` shared lane must require:

- the already-closed advanced-hardware and preview-transform boundary proofs
  as building blocks
- at least one grouped descriptor or acceptance task that spans controller
  workflow and preview workflow receipts together
- proof through public runtime, supervisor, and both stable host edges

### Advisory

The later lane may report but not block on:

- broader device-native display, motor, or haptic reruns
- richer browser-native queue or preview UX passes
- repeated-run workflow depth that stays useful but is not yet bounded

### Deferred

The shared lane must keep explicitly deferred:

- exhaustive device-vendor workflow certification matrices
- product-local page editors, browser queue editors, or controller scripting UX
- broader cross-generation integrated acceptance that belongs to later `g08`
  milestones
- device-native or browser-native workflow tooling that does not yet collapse
  cleanly into Signal-owned receipts

## Rules

### Rule 1: the lane stays additive over closed workflow contracts

Later grouped acceptance may combine controller workflow and preview workflow
surfaces, but it must stay a proof over already-closed Signal-owned contracts
instead of inventing a second workflow authority.

### Rule 2: the shared lane must stay machine-readable

The acceptance seam must not degrade into prose-only tranche logs or human
memory. Later batches should expose descriptors, supervisor JSON, or explicit
Effigy grouping that explains what the lane covers and why it passes.

### Rule 3: device-private and browser-local glue remain out of bounds

Device-private page logic, browser queue policy, and host-private workflow
helpers may inform scenario setup later, but they must not become the shared
acceptance surface.

### Rule 4: required, advisory, and deferred depth stay explicit

Signal must not hide unstable or expensive workflow depth inside the bounded
lane. If a scenario blocks the shared claim, it must be marked `required`. If
it is useful but non-blocking, it must stay `advisory`. If it is known but not
yet bounded or stable, it must stay `deferred`.

### Rule 5: runtime and stable host-edge truth must align

The shared lane must prove that public runtime receipts, supervisor export,
and both stable host edges tell the same bounded controller and preview
workflow story instead of allowing one device path or one preview path to
define a special case.

## Deferred scope

Batch 18.1 intentionally leaves these out:

- the concrete grouped descriptor and Effigy task names for the later lane
- exact device-native or browser-native reruns for advisory depth
- integrated live/device/preview acceptance that belongs to later `g08`
  milestones
- product-local page-editing, browser queue editing, or device scripting
  policy

## Batch 18.1 outcome

Batch 18.1 freezes the shared acceptance policy shape for control-surface and
preview workflow depth:

- Signal now has one explicit authority line for grouped control-surface and
  preview workflow acceptance instead of relying on isolated advanced-hardware
  and preview-transform boundary proof only
- the later grouped lane is now required to compose through public runtime
  receipts, supervisor export, and both stable host edges rather than device-
  private or browser-local workflow glue
- grouped descriptor, Effigy acceptance lane, and broader advisory or deferred
  workflow depth remain explicitly deferred until later `g08.018` batches

## Batch 18.2 outcome

Batch 18.2 materializes the first grouped acceptance surface for this seam:

- `signal-supervisor-tools` now owns one machine-readable
  `signal.runtime.control-preview-workflow-acceptance-lane` descriptor instead
  of leaving grouped controller and preview workflow proof spread across the
  isolated advanced-hardware and preview-transform boundaries
- Effigy now owns one runnable
  `effigy acceptance:control-preview-workflow-acceptance-lane` task that
  composes the bounded proof spine into one shared lane while keeping
  device-native and browser-native workflow reruns explicitly non-blocking
- the remaining work is now the final grouped consumer-proof closure rather
  than more policy setup

## Batch 18.3 outcome

Batch 18.3 closes the bounded consumer seam for this milestone:

- one repo-owned supervisor export proof now demonstrates that control-surface
  workflow, advanced-feedback, preview-device policy, and preview-workflow
  receipts are consumable together instead of only through the grouped
  descriptor and the isolated boundary tasks
- `effigy acceptance:control-preview-workflow-acceptance-lane` now composes
  the grouped descriptor, grouped export proof, and the existing advanced-
  hardware and preview-transform proof spine into one reusable shared
  acceptance lane
- `g08.018` is now complete, and the next `g08` queue is integrated
  live-ownership and workflow acceptance depth

## Next Task

Continue `g08.019` with Batch 19.1 by freezing the shared integrated live-
ownership and workflow acceptance contract on top of the closed Linux live,
device workflow, immersive, and control-preview workflow acceptance seams.
