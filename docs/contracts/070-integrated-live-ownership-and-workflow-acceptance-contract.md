# 070 Integrated Live-Ownership And Workflow Acceptance Contract

Status: complete
Owner: core-product
Updated: 2026-03-22
Related contracts: `docs/contracts/066-cross-backend-device-protocol-and-live-workflow-acceptance-contract.md`, `docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md`, `docs/contracts/068-immersive-render-and-monitoring-acceptance-contract.md`, `docs/contracts/069-control-surface-and-preview-workflow-acceptance-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared integrated live-ownership and workflow acceptance
contract for `g08.019` so Signal can prove its widened Linux live,
device-workflow, immersive, and control-preview workflow substrate through one
repo-owned evidence lane instead of parallel acceptance checklists, host-local
coordination shells, or product-local workflow glue.

## Authority hierarchy

Integrated live-ownership and workflow acceptance have one authority chain:

1. the closed grouped acceptance contracts define what Signal is allowed to
   claim about:
   - Linux live ownership, backend-native coordination, parity, and guarded
     failure posture
   - device workflow, controller-expression breadth, and advanced hardware
     workflow posture
   - immersive room-policy, deployment-monitoring, and renderer-export posture
   - control-surface workflow, preview-device policy, and preview-workflow
     posture
2. `signal-runtime` owns the typed receipts those grouped claims must still
   compose from:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - the runtime-owned receipt families already frozen by the grouped lanes
3. shared host crates own bounded local and server export for the same receipt
   families, but they do not own integrated acceptance meaning
4. `signal-supervisor-tools` must own the machine-readable descriptors that
   explain:
   - which grouped acceptance lanes are part of the integrated seam
   - which runtime, supervisor, and stable host-edge proofs are required
   - which broader repeated-run, environment-specific, or closeout-adjacent
     depth remains advisory or deferred
5. Effigy tasks must own the runnable grouping policy for the integrated lane:
   - which already-closed grouped acceptance tasks are required building blocks
   - which integrated checks become the mandatory `g08.019` acceptance path
   - which broader repeated-run or environment-specific depth remains
     non-blocking
6. downstream consumers may archive or rerun the outputs, but they must not
   become the authority for what Signal considers the canonical integrated
   live-ownership and workflow acceptance seam

If an integrated live-ownership or workflow acceptance claim cannot be
explained through the closed grouped contracts above, typed runtime receipts,
supervisor-tools descriptors, and repo-owned Effigy tasks, it is not yet part
of the shared Signal acceptance boundary.

## Existing acceptance anchors

This contract builds on the currently closed grouped proof tasks and
descriptors:

- `effigy acceptance:linux-live-acceptance-lane`
- `effigy acceptance:device-workflow-acceptance-lane`
- `effigy acceptance:immersive-acceptance-lane`
- `effigy acceptance:control-preview-workflow-acceptance-lane`
- `cargo run -p signal-supervisor-tools -- --describe-linux-live-acceptance-lane --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-device-workflow-acceptance-lane --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-immersive-acceptance-lane --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-control-preview-workflow-acceptance-lane --format=json`

Batch 19.1 does not claim these tasks already form one integrated acceptance
lane. It freezes how they must be composed and widened in later `g08.019`
batches.

## Shared vocabulary

### Integrated live-ownership and workflow acceptance

`integrated live-ownership and workflow acceptance` means one repo-owned,
machine-readable evidence lane that proves the widened Linux live, device
workflow, immersive, and control-preview workflow receipts remain consumable
together across the grouped `g08` acceptance seams already modeled by Signal.

It is not a distro certification program, not a renderer-vendor certification
program, and not a product-local controller, browser, or immersive workflow.

### Required acceptance evidence

`required acceptance evidence` means evidence that must remain green for
Signal to claim the shared `g08.019` integrated lane exists.

### Advisory acceptance evidence

`advisory acceptance evidence` means broader repeated-run, environment-
specific, or closeout-adjacent checks that improve confidence but do not yet
block the bounded integrated lane.

### Deferred acceptance evidence

`deferred acceptance evidence` means known and useful integrated scenario
depth that remains outside the bounded lane because it is not yet stable
enough, portable enough, or appropriately Signal-owned.

## Integrated scenario families

Batch 19.1 freezes four scenario families for later implementation.

### Family 1: Linux live and device workflow continuity

This family proves the integrated lane can surface one coherent live endpoint
and workflow story across:

- Linux live ownership and guarded continuity
- backend-native coordination and parity
- external MIDI live ownership and controller-expression breadth
- advanced hardware and control-surface workflow posture

### Family 2: Immersive and preview workflow continuity

This family proves the integrated lane spans the widened workflow substrate:

- immersive room-policy, deployment-monitoring, and renderer-export posture
- control-surface workflow posture that informs bounded device interaction
- preview-device policy and preview-workflow posture
- bounded continuity from live ownership into preview or monitoring-adjacent
  workflow answers

### Family 3: Cross-surface integrated coherence

This family proves the integrated lane can surface one bounded truth across:

- public runtime receipts
- supervisor export or descriptors
- stable local host-edge export
- stable server host-edge export

### Family 4: Shared grouped integrated acceptance export

This family proves the bounded lane can expose one machine-readable grouped
descriptor or acceptance task that spans the grouped Linux live, device
workflow, immersive, and control-preview workflow lanes instead of only
re-listing them as unrelated tasks.

## Required versus advisory versus deferred policy

Batch 19.1 freezes a three-tier policy.

### Required

The later `g08.019` shared lane must require:

- the already-closed Linux live, device workflow, immersive, and control-
  preview workflow grouped lanes as building blocks
- at least one integrated descriptor or acceptance task that spans more than
  one grouped lane together
- proof through public runtime, supervisor, and both stable host edges

### Advisory

The later lane may report but not block on:

- broader repeated-run confidence passes
- richer environment-specific or host-profile-specific mixes
- closer-to-closeout integrated reruns that stay useful but are not yet bounded

### Deferred

The shared lane must keep explicitly deferred:

- exhaustive distro, daemon, device-vendor, renderer-vendor, and browser UX
  certification matrices
- product-local controller pages, browser queue editors, immersive consoles,
  or repair workflows
- generation-closeout policy that belongs to later `g08.020`
- environment-specific tooling that does not yet collapse cleanly into
  Signal-owned receipts

## Rules

### Rule 1: the lane stays additive over closed grouped contracts

Later integrated acceptance may combine the grouped Linux live, device,
immersive, and control-preview workflow lanes, but it must stay a proof over
already-closed Signal-owned grouped contracts instead of inventing a second
semantic authority.

### Rule 2: the shared lane must stay machine-readable

The acceptance seam must not degrade into prose-only tranche logs or human
memory. Later batches should expose descriptors, supervisor JSON, or explicit
Effigy grouping that explains what the integrated lane covers and why it
passes.

### Rule 3: local workflow glue remains out of bounds

Backend-local, device-private, renderer-private, browser-local, and host-
private workflow glue may inform scenario setup later, but they must not
become the shared integrated acceptance surface.

### Rule 4: required, advisory, and deferred depth stay explicit

Signal must not hide unstable or expensive integrated depth inside the bounded
lane. If a scenario blocks the shared claim, it must be marked `required`. If
it is useful but non-blocking, it must stay `advisory`. If it is known but not
yet bounded or stable, it must stay `deferred`.

### Rule 5: runtime and stable host-edge truth must align

The shared lane must prove that public runtime receipts, supervisor export,
and both stable host edges tell the same bounded integrated live-ownership and
workflow story instead of allowing one grouped lane or one host path to define
a special case.

## Deferred scope

Batch 19.1 intentionally leaves these out:

- the concrete integrated descriptor and Effigy task names for the later lane
- exact repeated-run and environment-specific permutations for advisory depth
- generation-closeout gate policy that belongs to `g08.020`

## Batch 19.2 outcome

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.integrated-live-ownership-and-workflow-acceptance-lane`
  descriptor instead of leaving the broader `g08` integrated claim as four
  separate grouped lane descriptors only
- Effigy now owns one runnable
  `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
  task that composes the closed Linux live, device workflow, immersive, and
  control-preview workflow lanes into one repo-owned integrated acceptance
  seam while keeping repeated-run and environment-specific depth explicit and
  non-blocking
- the final grouped runtime, supervisor, and stable host-edge consumer proof
  remains intentionally deferred to Batch 19.3 rather than being implied by
  the grouped task alone

## Batch 19.3 outcome

- one repo-owned supervisor export proof now demonstrates that Linux live
  ownership, device workflow, immersive render and monitoring, and
  control-preview workflow receipts are consumable together instead of only
  through the grouped descriptor and grouped Effigy lane
- `effigy acceptance:integrated-live-ownership-and-workflow-acceptance-lane`
  now composes the four grouped lanes, the integrated descriptor proof, the
  integrated descriptor itself, and the grouped export proof into one reusable
  shared integrated acceptance seam
- this completes the bounded `g08.019` integrated acceptance contract and
  leaves `g08.020` as the next explicit queue for generation closeout and
  downstream workflow readiness

## Next task

Continue `g08.020` with Batch 20.1 by freezing the shared generation closeout
and downstream workflow readiness contract on top of the closed `g08.019`
integrated acceptance seam.
