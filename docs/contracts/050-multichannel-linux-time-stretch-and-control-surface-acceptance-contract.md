# 050 Multichannel, Linux, Time-Stretch, And Control-Surface Acceptance Contract

Status: complete
Owner: core-product
Updated: 2026-03-19
Related contracts: `docs/contracts/032-canonical-multichannel-layout-and-channel-role-contract.md`, `docs/contracts/034-multi-bus-graph-execution-and-auxiliary-topology-contract.md`, `docs/contracts/037-surround-bed-object-and-mix-policy-expansion-contract.md`, `docs/contracts/039-linux-cross-adapter-plugin-parity-and-sandbox-policy-contract.md`, `docs/contracts/040-linux-audio-backend-portability-across-alsa-jack-and-pipewire-contract.md`, `docs/contracts/041-linux-backend-clocking-duplex-and-endpoint-topology-parity-contract.md`, `docs/contracts/042-external-midi-endpoint-graph-and-device-identity-contract.md`, `docs/contracts/044-control-surface-transport-mapping-and-feedback-contract.md`, `docs/contracts/045-advanced-hardware-extensibility-and-scripting-safe-device-policy-contract.md`, `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`, `docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md`, `docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md`, `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first integrated acceptance contract for `g07.019` so Signal can
prove the widened `g07` feature surface through one bounded shared evidence
lane instead of isolated milestone-local checks, product-local harnesses, or
ad hoc manual scenario bundles.

## Authority hierarchy

Integrated `g07` acceptance depth has one authority chain:

1. the closed `g07` routing, Linux, control-surface, and stretch contracts
   define what Signal is allowed to claim about:
   - multichannel, sidechain, multi-bus, complex plugin-I/O, and spatial
     routing meaning
   - Linux plugin parity, Linux backend portability, and Linux backend
     clock-topology parity
   - external MIDI, controller-expression, control-surface, and advanced
     hardware policy posture
   - stretch-engine, marker-analysis, transform-artifact, and preview-service
     readiness
2. `signal-runtime` owns the typed receipts those claims must compose from:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeExecutionTopologySummary`
   - `RuntimeExternalIoSnapshot`
   - `RuntimeExternalMidiSnapshot`
   - `RuntimeControlSurfaceSnapshot`
   - `RuntimeAdvancedHardwareSnapshot`
   - `RuntimeStretchEngineSnapshot`
   - `RuntimeMarkerAnalysisSnapshot`
   - `RuntimeTransformArtifactSnapshot`
   - `RuntimePreviewTransformServiceSnapshot`
3. shared host crates own bounded scenario brokering and runtime-owned export
   for the same receipt families across local and server host edges
4. `signal-supervisor-tools` must own the machine-readable integrated
   acceptance descriptors that explain:
   - which widened `g07` families are part of the bounded lane
   - which runtime receipts the lane is expected to surface together
   - which checks are required, advisory, or deferred
5. Effigy tasks must own the runnable grouping policy for integrated `g07`
   acceptance:
   - which closed boundary tasks are required building blocks
   - which grouped integrated checks become the bounded shared lane
   - which broader scenario depth remains advisory or deferred
6. downstream consumers may rerun or archive the acceptance outputs, but they
   must not become the authority for what Signal considers the canonical
   integrated `g07` acceptance lane

If an integrated acceptance claim cannot be explained through closed `g07`
contracts, typed runtime receipts, `signal-supervisor-tools` descriptors, and
repo-owned Effigy tasks, it is not yet part of the shared acceptance boundary.

## Existing acceptance anchors

This contract builds on the currently closed bounded proof tasks and
descriptors:

- `effigy acceptance:multichannel-boundary`
- `effigy acceptance:sidechain-boundary`
- `effigy acceptance:multi-bus-boundary`
- `effigy acceptance:complex-io-boundary`
- `effigy acceptance:spatial-boundary`
- `effigy acceptance:lv2-boundary`
- `effigy acceptance:linux-plugin-parity-boundary`
- `effigy acceptance:linux-audio-backend-boundary`
- `effigy acceptance:linux-backend-clock-topology-boundary`
- `effigy acceptance:external-midi-boundary`
- `effigy acceptance:controller-expression-boundary`
- `effigy acceptance:control-surface-boundary`
- `effigy acceptance:advanced-hardware-boundary`
- `effigy acceptance:stretch-boundary`
- `effigy acceptance:marker-analysis-boundary`
- `effigy acceptance:transform-artifact-boundary`
- `effigy acceptance:preview-transform-boundary`
- `cargo run -p signal-supervisor-tools -- --describe-multichannel-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-linux-plugin-parity-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-linux-audio-backend-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-linux-backend-clock-topology-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-control-surface-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-advanced-hardware-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-stretch-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-marker-analysis-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-transform-artifact-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-preview-transform-boundary --format=json`

Batch 19.1 does not claim these tasks already form one integrated lane. It
freezes how later `g07.019` work should group and widen them.

## Shared vocabulary

### Integrated acceptance lane

`integrated acceptance lane` means the bounded shared acceptance bundle that
combines multiple already-closed `g07` boundary seams into one runnable
Signal-owned proof path for the widened feature surface.

It is broader than a milestone-local boundary proof, but narrower than a full
environment-certification matrix or product-level launch gate.

### Required acceptance evidence

`required acceptance evidence` means a repo-owned acceptance check that must
stay green for Signal to claim the current bounded integrated `g07` lane.

### Advisory acceptance evidence

`advisory acceptance evidence` means a shared and runnable check that improves
confidence but does not yet block the bounded lane.

### Deferred acceptance evidence

`deferred acceptance evidence` means known and useful scenario depth that
remains outside the lane because it is not yet stable enough, bounded enough,
or worth promoting into the shared fast path.

### Cross-family runtime evidence

`cross-family runtime evidence` means one machine-readable export or grouped
acceptance result that proves routing, Linux, controller, and stretch receipts
can be consumed together rather than only through separate boundary-local
descriptors.

## Integrated scenario families

Batch 19.1 freezes four required scenario families and one cross-family proof
family for later implementation.

### Family 1: Routing and multichannel coherence

This family proves the integrated lane can surface coherent routing truth
across:

- multichannel layout and channel-role substrate
- sidechain and secondary-input posture
- multi-bus and auxiliary-topology identity
- complex plugin-I/O and bounded spatial execution posture

### Family 2: Linux plugin and backend continuity

This family proves the integrated lane spans the widened Linux-native seam:

- LV2 baseline and Linux-native plugin lifecycle depth
- Linux cross-adapter plugin parity and sandbox policy
- Linux backend portability across ALSA, JACK, and PipeWire
- Linux backend clocking, duplex, and endpoint-topology parity

### Family 3: External control and advanced hardware coherence

This family proves the integrated lane spans the widened controller and device
surface:

- external MIDI endpoint and device-identity baseline
- richer controller-expression posture
- control-surface transport, mapping, and feedback baseline
- advanced hardware extensibility and scripting-safe device policy

### Family 4: Stretch, analysis, artifact, and preview continuity

This family proves the integrated lane spans the widened sample-domain media
surface:

- stretch-engine readiness and fallback
- marker-analysis posture
- transform-artifact readiness, invalidation, and reuse
- preview-transform readiness, degraded-state, and fallback

### Family 5: Cross-family integrated export

This family proves the bounded lane can surface one machine-readable export or
descriptor that carries useful receipts from more than one family above at the
same time instead of only re-listing the component boundary tasks.

## Required versus advisory policy

Batch 19.1 freezes a three-tier policy:

- `required`
  - bounded integrated acceptance checks that must remain green for the shared
    `g07` lane claim
- `advisory`
  - broader shared scenarios that improve confidence but do not yet block the
    bounded lane
- `deferred`
  - known scenario depth intentionally excluded until it is stable enough or
    belongs to later closeout work

The required tier for later `g07.019` implementation must:

- compose already-closed boundary tasks instead of replacing them
- add at least one grouped integrated descriptor or task that spans multiple
  families above
- stay bounded enough for maintainers to rerun deliberately without collapsing
  into the `g07.020` closeout gate

The advisory tier may include:

- repeated-run confidence passes over the required lane
- broader local and server host permutations
- richer multichannel, controller, or preview scenario mixes that are useful
  but not yet worth blocking on

The deferred tier currently includes:

- exhaustive adapter, plugin, and environment matrices
- live backend-native ownership breadth beyond the bounded Linux proof seam
- browser, editor, or workflow-local preview and controller scenarios
- Loophole-facing closeout and promotion verdicts that belong to `g07.020`

## Rules

### Rule 1: integrated acceptance stays additive over closed `g07` boundaries

Later harnesses may combine routing, Linux, controller, and stretch surfaces,
but they must stay proofs over already-closed Signal-owned contracts instead of
inventing a new semantic authority.

### Rule 2: the shared lane must stay machine-readable

Integrated acceptance must not degrade into prose-only tranche logs or human
memory. Later batches should expose descriptors, supervisor JSON, or explicit
Effigy grouping that explains what the lane covers and why it passes.

### Rule 3: required, advisory, and deferred depth must stay explicit

Signal must not hide longer-running or broader scenario depth inside the
bounded lane. If a scenario blocks the shared acceptance claim, it must be
`required`. If it is useful but non-blocking, it must be `advisory`. If it is
known but not yet bounded or stable, it must stay `deferred`.

### Rule 4: the lane must prove cross-family receipt coherence

`g07.019` must not stop at re-listing milestone-local tasks. Later batches
must show that useful runtime-owned receipts from more than one family can be
consumed together through one shared export or descriptor.

### Rule 5: integrated acceptance is repo-owned, not product-owned

Signal may support Loophole and other downstream consumers, but the canonical
harnesses, descriptors, and tasks must remain in shared Signal crates and
Effigy surfaces rather than product-local scripts or CI-only glue.

## Deferred scope

Batch 19.1 intentionally keeps the following outside the contract:

- exact CLI flags and task names for the later integrated lane
- final grouped acceptance descriptor schema
- exact host/scenario permutations for later harness depth
- the Loophole-facing generation closeout verdict, which belongs to `g07.020`

## Batch 19.1 outcome

Batch 19.1 freezes the first bounded integrated acceptance policy for `g07`:

- Signal now has one explicit contract for how routing, Linux, controller, and
  stretch proof depth should be grouped into a bounded acceptance lane instead
  of remaining a loose pile of milestone-local descriptors
- the required, advisory, and deferred tiers are now explicit, which prevents
  later acceptance work from quietly promoting unstable or product-local depth
- Batch 19.2 can now focus on materializing the first grouped descriptor and
  runnable lane instead of reopening what acceptance breadth belongs to Signal

## Batch 19.2 outcome

Batch 19.2 materializes the first grouped `g07` acceptance lane:

- `signal-supervisor-tools` now exposes a machine-readable grouped descriptor
  for the bounded `g07` lane instead of leaving the contract as prose-only
  grouping policy
- Effigy now owns one repo-owned grouped rerun lane across the required
  routing, Linux, controller, and stretch families
- Batch 19.3 can now prove whether that grouped lane yields meaningful
  cross-family runtime evidence rather than only restating the component
  boundary tasks

## Batch 19.3 outcome

Batch 19.3 closes the bounded integrated `g07` acceptance contract:

- `signal-supervisor-tools` now proves one machine-readable supervisor export
  can carry routing, Linux backend, control-surface or advanced-hardware, and
  stretch or preview receipts together instead of only enumerating component
  boundary tasks
- Effigy now reruns that cross-family export proof inside the repo-owned
  `acceptance:g07-integrated-acceptance-lane` task
- the grouped lane is now meaningful downstream-ready runtime evidence rather
  than a descriptor-only wrapper over already-closed milestone seams
- the remaining Loophole-facing feature-readiness and generation closeout
  verdict now belongs to `g07.020`

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
