# 067 Live Linux Backend Acceptance And Failure-Injection Contract

Status: complete
Owner: core-product
Updated: 2026-03-22
Related contracts: `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`, `docs/contracts/053-jack-transport-graph-and-backend-native-coordination-contract.md`, `docs/contracts/054-pipewire-and-alsa-session-role-device-claim-and-stream-policy-parity-contract.md`, `docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared live Linux backend acceptance and failure-injection
contract for `g08.016` so Signal can prove live backend ownership, guarded
continuity, backend parity, and bounded failure behavior through one repo-
owned evidence lane instead of isolated boundary proofs, backend-native daemon
policy, or ad hoc recovery reruns.

## Authority hierarchy

Live Linux backend acceptance and failure-injection depth have one authority
chain:

1. the closed Linux contracts define what Signal is allowed to claim about:
   - live backend ownership, session role, and guarded ownership fallback
   - JACK transport, graph coordination, and bounded backend-native posture
   - PipeWire and ALSA session-role, device-claim, and stream-policy parity
   - Linux clocking, duplex, and endpoint-topology parity
2. `signal-runtime` owns the typed receipts those claims must compose from:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeLinuxBackendSessionSnapshot`
   - `RuntimeJackCoordinationSnapshot`
   - `RuntimePipeWireAlsaParitySnapshot`
   - `RuntimeHostClockingSummary`
   - `RuntimeFaultStatusSnapshot`
   - `RuntimeInterruptionSummary`
3. shared host crates own bounded local and server export for the same Linux
   receipt families, but they do not own acceptance meaning
4. `signal-supervisor-tools` must own the machine-readable descriptors that
   explain:
   - which Linux live-backend families are part of the shared acceptance lane
   - which runtime, supervisor, and stable host-edge proofs are required
   - which broader daemon-native or backend-specific depth remains advisory or
     deferred
5. Effigy tasks must own the runnable grouping policy for the shared lane:
   - which already-closed Linux boundary tasks are required building blocks
   - which grouped checks become the mandatory `g08.016` acceptance path
   - which broader failure-injection or repeated-run depth remains non-blocking
6. downstream consumers may archive or rerun the outputs, but they must not
   become the authority for what Signal considers the canonical live Linux
   backend acceptance seam

If a live Linux backend or failure-injection claim cannot be explained through
the closed contracts above, typed runtime receipts, supervisor-tools
descriptors, and repo-owned Effigy tasks, it is not yet part of the shared
Signal acceptance boundary.

## Existing acceptance anchors

This contract builds on the currently closed bounded proof tasks and
descriptors:

- `effigy acceptance:linux-live-ownership-boundary`
- `effigy acceptance:jack-coordination-boundary`
- `effigy acceptance:pipewire-alsa-parity-boundary`
- `effigy acceptance:linux-backend-clock-topology-boundary`
- `cargo run -p signal-supervisor-tools -- --describe-linux-live-ownership-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-jack-coordination-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-pipewire-alsa-parity-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-linux-backend-clock-topology-boundary --format=json`

Batch 16.1 does not claim these tasks already form one grouped acceptance
lane. It freezes how they must be composed and widened in later `g08.016`
batches.

## Shared vocabulary

### Live Linux backend acceptance

`live Linux backend acceptance` means one repo-owned, machine-readable evidence
lane that proves Signal's live ALSA, JACK, and PipeWire ownership receipts
remain consumable across the bounded Linux backend families already modeled by
Signal.

It is not a distro certification program, not a daemon packaging guarantee,
and not a product-local repair workflow.

### Failure-injection acceptance

`failure-injection acceptance` means the bounded proof that guarded detach,
restart, parity degradation, and clock-topology impact remain consumable
through shared runtime, supervisor, and stable host-edge receipts when Linux
backend paths are stressed or interrupted.

### Required acceptance evidence

`required acceptance evidence` means evidence that must remain green for
Signal to claim the shared `g08.016` acceptance lane exists.

### Advisory acceptance evidence

`advisory acceptance evidence` means broader Linux recovery, repeated-run, or
daemon-native checks that improve confidence but do not yet block the bounded
lane.

### Deferred acceptance evidence

`deferred acceptance evidence` means known and useful Linux scenario depth
that remains outside the bounded lane because it is not yet stable enough,
portable enough, or appropriately Signal-owned.

## Integrated scenario families

Batch 16.1 freezes four scenario families for later implementation.

### Family 1: Live ownership and guarded continuity

This family proves the shared lane can surface coherent Linux live-backend
truth across:

- live backend ownership posture
- session lifecycle and guarded ownership fallback
- interruption and restart continuity
- bounded fault or degraded outcomes

### Family 2: Backend-native coordination and parity

This family proves the shared lane spans the widened backend-native seam:

- JACK transport and graph coordination posture
- PipeWire and ALSA session-role, device-claim, and stream-policy parity
- clocking, duplex, and endpoint-topology parity
- bounded guarded or unavailable backend answers

### Family 3: Cross-backend host-edge coherence

This family proves the shared lane can surface one bounded truth across:

- public runtime receipts
- supervisor export or descriptors
- stable local host-edge export
- stable server host-edge export

### Family 4: Shared grouped acceptance export

This family proves the bounded lane can expose one machine-readable grouped
descriptor or acceptance task that spans more than one family above instead of
only re-listing isolated Linux boundary-local tasks.

## Required versus advisory versus deferred policy

Batch 16.1 freezes a three-tier policy.

### Required

The later `g08.016` shared lane must require:

- the already-closed Linux live ownership, JACK coordination, PipeWire/ALSA
  parity, and clock-topology boundary proofs as building blocks
- at least one grouped descriptor or acceptance task that spans live
  ownership, backend-native coordination, and parity receipts together
- proof through public runtime, supervisor, and both stable host edges

### Advisory

The later lane may report but not block on:

- broader repeated-run confidence passes
- wider daemon-native or backend-native recovery permutations
- richer Linux environment mixes that stay useful but are not yet bounded

### Deferred

The shared lane must keep explicitly deferred:

- exhaustive distro, daemon, session-manager, and packaging certification
  matrices
- product-local repair, device browser, or patchbay workflows
- broader cross-generation integrated acceptance that belongs to later `g08`
  milestones
- backend-native failure tooling that does not yet collapse cleanly into
  Signal-owned receipts

## Rules

### Rule 1: the lane stays additive over closed Linux contracts

Later grouped acceptance may combine Linux live ownership, JACK, PipeWire/ALSA,
and clock-topology surfaces, but it must stay a proof over already-closed
Signal-owned contracts instead of inventing a second semantic authority.

### Rule 2: the shared lane must stay machine-readable

The acceptance seam must not degrade into prose-only tranche logs or human
memory. Later batches should expose descriptors, supervisor JSON, or explicit
Effigy grouping that explains what the lane covers and why it passes.

### Rule 3: backend-native and daemon-local glue remain out of bounds

Daemon policy, session-manager glue, and host-private repair helpers may
inform scenario setup later, but they must not become the shared acceptance
surface.

### Rule 4: required, advisory, and deferred depth stay explicit

Signal must not hide unstable or expensive Linux failure depth inside the
bounded lane. If a scenario blocks the shared claim, it must be marked
`required`. If it is useful but non-blocking, it must stay `advisory`. If it
is known but not yet bounded or stable, it must stay `deferred`.

### Rule 5: runtime and stable host-edge truth must align

The shared lane must prove that public runtime receipts, supervisor export,
and both stable host edges tell the same bounded Linux live-backend story
instead of allowing one backend or one host path to define a special case.

## Deferred scope

Batch 16.1 intentionally leaves these out:

- the concrete grouped descriptor and Effigy task names for the later lane
- exact failure-injection permutations for repeated or advisory reruns
- immersive, preview, device-workflow, or generation-level integrated
  acceptance that belongs to later `g08` milestones
- distro or daemon packaging policy beyond the shared Signal-owned receipt seam

## Batch 16.1 outcome

Batch 16.1 freezes the shared acceptance policy shape for live Linux backend
ownership and failure-injection depth:

- Signal now has one explicit authority line for grouped Linux live-backend
  acceptance instead of relying on isolated boundary-local proofs
- later `g08.016` implementation is forced to build on the closed live
  ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology
  seams instead of daemon-local policy or backend-specific recovery glue
- Batch 16.2 can now focus on materializing one grouped descriptor and task
  instead of reopening what the shared Linux live acceptance lane means

## Batch 16.2 outcome

Batch 16.2 materializes the first repo-owned grouped acceptance seam for live
Linux backend ownership and failure-injection depth:

- `signal-supervisor-tools` now exposes one machine-readable
  `signal.runtime.linux-live-acceptance-lane` descriptor instead of leaving
  grouped Linux live proof implicit across isolated Linux boundary descriptors
- Effigy now owns one runnable `effigy acceptance:linux-live-acceptance-lane`
  task that composes the already-closed Linux live ownership, JACK
  coordination, PipeWire/ALSA parity, and clock-topology proof spine
- advisory and deferred backend-native failure depth remain explicit rather
  than collapsing into the mandatory shared lane

## Batch 16.3 outcome

Batch 16.3 closes the widened Linux live acceptance seam through one grouped
consumer-facing supervisor export proof on top of the repo-owned grouped lane.

- `signal-supervisor-tools` now proves one supervisor export can carry Linux
  live ownership, JACK coordination, PipeWire/ALSA parity, and clock-topology
  truth together instead of only composing isolated Linux boundary descriptors
- `effigy acceptance:linux-live-acceptance-lane` now runs the grouped export
  proof, the machine-readable descriptor, and the already-closed Linux
  boundary proofs as one reusable acceptance lane
- the shared claim stays additive over the closed Linux contracts and typed
  runtime receipts instead of opening a daemon-local recovery shell or a
  backend-private failure-injection model

## Next Task

Continue `g08.017` with Batch 17.1 by freezing the shared immersive render and
monitoring acceptance contract on top of the closed immersive room-policy,
deployment-monitoring, renderer-export, and spatial consumer seams.
