# 066 Cross-Backend Device Protocol And Live Workflow Acceptance Contract

Status: active
Owner: core-product
Updated: 2026-03-22
Related contracts: `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`, `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`, `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`, `docs/contracts/060-advanced-control-surface-display-motor-and-haptic-transport-contract.md`, `docs/contracts/061-control-surface-scene-mapping-feedback-pages-and-safe-action-graph-contract.md`, `docs/contracts/065-live-external-midi-device-ownership-and-backend-parity-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared cross-backend device protocol and live workflow
acceptance contract for `g08.015` so Signal can prove the widened device and
workflow substrate through one bounded repo-owned evidence lane instead of
isolated boundary checks, backend-local endpoint policy, or product-local live
performance glue.

## Authority hierarchy

Cross-backend device protocol and live workflow acceptance have one authority
chain:

1. the closed device and workflow contracts define what Signal is allowed to
   claim about:
   - richer controller-expression and external MIDI meaning
   - control-surface transport, mapping, and feedback posture
   - advanced hardware and bounded feedback transport
   - scene mapping, feedback pages, and safe action graph posture
   - live external MIDI ownership, attach continuity, and backend parity
2. `signal-runtime` owns the typed receipts those claims must compose from:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeExternalMidiEndpointGraphSnapshot`
   - `RuntimeControlSurfaceSnapshot`
   - `RuntimeAdvancedHardwareSnapshot`
   - `RuntimeInterruptionSummary`
   - `RuntimeLinuxBackendSessionSnapshot`
3. shared host crates own bounded local and server export for the same receipt
   families, but they do not own acceptance meaning
4. `signal-supervisor-tools` must own the machine-readable descriptors that
   explain:
   - which cross-backend device and workflow families are part of the shared
     acceptance lane
   - which runtime, supervisor, and stable host-edge proofs are required
   - which broader backend-native or product-local depth remains advisory or
     deferred
5. Effigy tasks must own the runnable grouping policy for the shared lane:
   - which already-closed boundary tasks are required building blocks
   - which grouped checks become the mandatory `g08.015` acceptance path
   - which broader reruns or backend-native probes remain non-blocking
6. downstream consumers may archive or rerun the outputs, but they must not
   become the authority for what Signal considers the canonical cross-backend
   device workflow acceptance seam

If a cross-backend device protocol or live workflow claim cannot be explained
through the closed contracts above, typed runtime receipts, supervisor-tools
descriptors, and repo-owned Effigy tasks, it is not yet part of the shared
Signal acceptance boundary.

## Existing acceptance anchors

This contract builds on the currently closed bounded proof tasks and
descriptors:

- `effigy acceptance:external-midi-boundary`
- `effigy acceptance:controller-expression-boundary`
- `effigy acceptance:control-surface-boundary`
- `effigy acceptance:advanced-hardware-boundary`
- `cargo run -p signal-supervisor-tools -- --describe-external-midi-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-controller-expression-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-control-surface-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-advanced-hardware-boundary --format=json`

Batch 15.1 does not claim these tasks already form one grouped acceptance
lane. It freezes how they must be composed and widened in later `g08.015`
batches.

## Shared vocabulary

### Cross-backend device protocol acceptance

`cross-backend device protocol acceptance` means one repo-owned, machine-
readable evidence lane that proves Signal's device-protocol and live workflow
receipts remain consumable across the bounded backend families already modeled
by Signal.

It is not an exhaustive environment matrix, not a session-manager
certification program, and not a product-local rehearsal workflow.

### Live workflow acceptance

`live workflow acceptance` means the bounded proof that live endpoint
ownership, controller expression, control-surface posture, and advanced
hardware workflow receipts can be consumed together without backend-local
reclassification or host-private glue.

### Required acceptance evidence

`required acceptance evidence` means evidence that must remain green for
Signal to claim the shared `g08.015` acceptance lane exists.

### Advisory acceptance evidence

`advisory acceptance evidence` means broader shared device or workflow checks
that improve confidence but do not yet block the bounded lane.

### Deferred acceptance evidence

`deferred acceptance evidence` means known and useful scenario depth that
remains outside the bounded lane because it is not yet stable enough, portable
enough, or appropriately Signal-owned.

## Integrated scenario families

Batch 15.1 freezes four scenario families for later implementation.

### Family 1: Live endpoint ownership and protocol continuity

This family proves the shared lane can surface coherent live device truth
across:

- external MIDI graph, route, and capability posture
- live external MIDI ownership and attach continuity
- backend parity and guarded parity outcome
- controller-expression relevance where live protocol breadth matters

### Family 2: Control-surface and advanced hardware workflow coherence

This family proves the shared lane spans the widened device workflow surface:

- control-surface transport, mapping, and bounded feedback posture
- advanced display, motor, and haptic transport posture
- scene-mapping, feedback-page, and safe-action workflow posture
- bounded guarded or degraded workflow outcomes

### Family 3: Cross-backend parity and host-edge export coherence

This family proves the shared lane can surface one bounded truth across:

- public runtime receipts
- supervisor export or descriptors
- stable local host-edge export
- stable server host-edge export

### Family 4: Shared grouped acceptance export

This family proves the bounded lane can expose one machine-readable grouped
descriptor or acceptance task that spans more than one family above instead of
only re-listing isolated boundary-local tasks.

## Required versus advisory versus deferred policy

Batch 15.1 freezes a three-tier policy.

### Required

The later `g08.015` shared lane must require:

- the already-closed external MIDI, controller-expression, control-surface,
  and advanced-hardware boundary proofs as building blocks
- at least one grouped descriptor or acceptance task that spans external MIDI
  live ownership and device-workflow receipts together
- proof through public runtime, supervisor, and both stable host edges

### Advisory

The later lane may report but not block on:

- broader repeated-run confidence passes
- wider permutations across backend-native transport detail
- richer device-matrix coverage that stays useful but is not yet bounded

### Deferred

The shared lane must keep explicitly deferred:

- exhaustive ALSA, JACK, PipeWire, and session-manager certification matrices
- product-local controller pages, live-performance scenes, or show-control UX
- backend-native patchbay, reservation, or routing-console depth
- remote collaboration or cloud device-handoff workflows

## Rules

### Rule 1: the lane stays additive over closed device contracts

Later grouped acceptance may combine external MIDI, controller, and advanced
hardware surfaces, but it must stay a proof over already-closed Signal-owned
contracts instead of inventing a second semantic authority.

### Rule 2: the shared lane must stay machine-readable

The acceptance seam must not degrade into prose-only tranche logs or human
memory. Later batches should expose descriptors, supervisor JSON, or explicit
Effigy grouping that explains what the lane covers and why it passes.

### Rule 3: backend-local and product-local glue remain out of bounds

Backend-native endpoint policy, host-private controller helpers, and
product-local workflow shells may inform scenario setup later, but they must
not become the shared acceptance surface.

### Rule 4: required, advisory, and deferred depth stay explicit

Signal must not hide unstable or expensive device-depth checks inside the
bounded lane. If a scenario blocks the shared claim, it must be marked
`required`. If it is useful but non-blocking, it must stay `advisory`. If it
is known but not yet bounded or stable, it must stay `deferred`.

### Rule 5: runtime and stable host-edge truth must align

The shared lane must prove that public runtime receipts, supervisor export,
and both stable host edges tell the same bounded device-workflow story instead
of allowing one host path or one backend to define a special case.

## Deferred scope

Batch 15.1 intentionally leaves these out:

- the concrete grouped descriptor and Effigy task names for the later lane
- exact backend permutations for repeated or advisory reruns
- Linux live backend failure-injection depth, which belongs to later `g08.016`
- immersive, preview, or cross-family integrated acceptance that belongs to
  later `g08` milestones

## Batch 15.1 outcome

Batch 15.1 freezes the shared acceptance policy shape for cross-backend device
protocol and live workflow depth:

- Signal now has one explicit authority line for grouped device-protocol and
  live workflow acceptance instead of relying on isolated boundary-local
  proofs
- later `g08.015` implementation is forced to build on the closed external
  MIDI, controller-expression, control-surface, advanced-hardware, and live
  ownership seams instead of backend-local endpoint policy or product-local
  workflow glue
- Batch 15.2 can now focus on materializing one grouped descriptor and task
  instead of reopening what the shared device workflow acceptance lane means

## Next Task

Continue `g08.015` with Batch 15.2 by wiring the first repo-owned descriptor
and acceptance lane for the shared cross-backend device protocol and live
workflow seam while keeping backend-specific depth explicit and non-blocking.
