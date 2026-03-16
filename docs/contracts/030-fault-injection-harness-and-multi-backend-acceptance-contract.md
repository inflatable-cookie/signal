# 030 Fault-Injection Harness And Multi-Backend Acceptance Contract

Status: complete
Owner: core-product
Updated: 2026-03-16
Related contracts: `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`, `docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md`, `docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md`, `docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md`, `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`, `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first shared fault-injection and multi-backend acceptance contract
for `g06.019` so Signal can prove the widened `g06` runtime surface through a
bounded integrated evidence lane instead of only milestone-local checks,
product-local harnesses, or ad hoc manual scenario runs.

## Authority hierarchy

Integrated acceptance depth has one authority chain:

1. the closed `g06` contracts define what Signal is allowed to claim about:
   - interruption, recovery, render continuity, and fault attribution
   - deferred-work policy, timing pressure, and critical-path instrumentation
   - VST3, AU, and cross-adapter portability breadth
   - device supervision, clock topology, external-I/O, media-service, and
     analysis-metadata boundaries
2. `signal-runtime` owns the typed receipts those claims must compose from:
   - `RuntimeObservationReport`
   - `RuntimeSupervisorReport`
   - `RuntimeFaultDiagnosticReceipt`
   - `RuntimePerformanceSnapshot`
   - `RuntimePerformanceTraceReceipt`
   - `RuntimeDeviceSupervisionSnapshot`
   - `RuntimeExternalIoSnapshot`
   - `RuntimeMediaServiceSnapshot`
   - `RuntimeMediaLibraryServiceSnapshot`
3. shared host crates own scenario brokering and bounded backend evidence for:
   - local and server watchdog or recovery scenarios
   - backend-specific plugin scan/load coverage
   - hardware, monitoring, and external-I/O availability evidence
4. `signal-supervisor-tools` owns the machine-readable integrated acceptance
   descriptors that explain:
   - which scenario bundles are part of the shared acceptance lane
   - which typed receipts those bundles are expected to surface
   - which checks are required, advisory, or deferred
5. Effigy tasks own the runnable grouping policy for integrated acceptance:
   - which already-closed boundary tasks are required building blocks
   - which new integrated harnesses belong in the bounded fast lane
   - which broader soak or backend depth remains advisory or deferred
6. downstream consumers may run or archive the acceptance outputs, but they
   must not become the authority for what Signal considers the canonical
   integrated acceptance lane

If an integrated acceptance claim cannot be explained through closed `g06`
contracts, typed runtime receipts, `signal-supervisor-tools` descriptors, and
repo-owned Effigy tasks, it is not yet part of the shared acceptance boundary.

## Existing acceptance anchors

This contract builds on the currently closed bounded proof tasks and
descriptors:

- `effigy acceptance:interruption-boundary`
- `effigy acceptance:recording-continuity`
- `effigy acceptance:offline-render-continuity`
- `effigy acceptance:fault-diagnostic-boundary`
- `effigy acceptance:block-timing-boundary`
- `effigy acceptance:critical-path-boundary`
- `effigy acceptance:deferred-work-policy-boundary`
- `effigy acceptance:plugin-continuity`
- `effigy acceptance:vst3-boundary`
- `effigy acceptance:au-boundary`
- `effigy acceptance:cross-adapter-parity-boundary`
- `effigy acceptance:generic-event-boundary`
- `effigy acceptance:recall-portability-boundary`
- `effigy acceptance:device-supervision-boundary`
- `effigy acceptance:clock-topology-boundary`
- `effigy acceptance:external-io-boundary`
- `effigy acceptance:media-service-boundary`
- `effigy acceptance:analysis-metadata-boundary`
- `cargo run -p signal-supervisor-tools -- --describe-vst3-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-au-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-cross-adapter-parity-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-device-supervision-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-external-io-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-media-service-boundary --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-analysis-metadata-boundary --format=json`

Batch 19.1 does not claim these tasks already form one integrated harness. It
freezes how they should be grouped and widened in later batches.

## Shared vocabulary

### Fault-injection harness

`fault-injection harness` means one repo-owned, repeatable scenario bundle that
deliberately forces a bounded class of runtime stress, degradation, device
loss, plugin failure, unavailability, or invalidation so Signal can prove its
typed receipts stay coherent through that disturbance.

It is not an app-local QA checklist, private shell script, or free-form manual
operator playbook.

### Integrated acceptance lane

`integrated acceptance lane` means the bounded shared acceptance bundle that
combines multiple already-closed boundary seams into one runnable Signal-owned
proof path for the widened `g06` surface.

It is broader than a milestone-local boundary proof, but still narrower than a
full environment-certification matrix.

### Required acceptance evidence

`required acceptance evidence` means a repo-owned acceptance check that must
stay green for Signal to claim the current integrated `g06` acceptance lane.

### Advisory acceptance evidence

`advisory acceptance evidence` means a shared and runnable check that improves
confidence but does not yet block the bounded integrated lane.

### Deferred acceptance evidence

`deferred acceptance evidence` means a known and useful scenario that remains
outside the integrated lane because it is not yet stable enough, portable
enough, or bounded enough to promote into the shared path.

### Multi-backend acceptance

`multi-backend acceptance` means the integrated lane explicitly exercises more
than one plugin or host-path backend family already modeled by Signal, rather
than collapsing back to one adapter-specific happy path.

Batch 19.1 freezes the requirement that the shared lane span the widened
adapter and hardware/media seams, not that every environment or plugin format
must be exercised identically.

## Integrated scenario families

Batch 19.1 freezes five scenario families for later implementation.

### Family 1: Recovery and fault attribution

This family proves the integrated lane can surface typed interruption,
continuity, and fault-cause truth across:

- runtime interruption and resumability
- recording and offline-render continuity
- runtime fault-cause attribution
- device supervision and restart-state transitions

### Family 2: Scheduling and execution pressure

This family proves the integrated lane can surface bounded timing and policy
pressure through:

- block timing and deadline pressure
- critical-path and hot-lane attribution
- deferred-work priority, backpressure, starvation, and cancellation state

### Family 3: Adapter and portability breadth

This family proves the integrated lane spans the widened adapter surface:

- shared plugin continuity
- VST3 boundary breadth
- AU boundary breadth
- cross-adapter parity receipts
- generic event and recall-portability depth where they affect adapter claims

### Family 4: Hardware and external-I/O continuity

This family proves the integrated lane covers the widened hardware seam:

- device supervision
- clock topology and duplex mismatch
- external-I/O, monitoring, and loopback state

### Family 5: Media-service and library-service continuity

This family proves the integrated lane spans the reusable media substrate:

- media-service readiness and invalidation
- analysis-metadata and library-service descriptors

## Required versus advisory policy

Batch 19.1 freezes a three-tier policy:

- `required`
  - bounded integrated acceptance checks that must remain green for the shared
    `g06` acceptance lane claim
- `advisory`
  - broader shared scenarios that improve confidence but do not yet block the
    bounded lane
- `deferred`
  - known scenario depth that is intentionally excluded until it stabilizes

The required tier for later `g06.019` implementation must:

- compose the already-closed boundary tasks instead of replacing them
- add at least one grouped integrated harness descriptor or task that spans
  multiple families above
- stay bounded enough for maintainers to run deliberately without collapsing
  into the later long-session soak lane of `g06.020`

The advisory tier may include:

- broader mixed watchdog paths
- wider integrated local-host scenario bundles
- repeated-run confidence passes over the required lane

The deferred tier currently includes:

- long-session or repeated soak policy that belongs to `g06.020`
- unstable server-host integrated recovery-overlap scenarios that still trip
  the current attach-limit constraint
- exhaustive environment matrices across every backend or platform
- product-local manual QA and browser/editor workflows

## Rules

### Rule 1: integrated acceptance stays additive over closed boundaries

Later harnesses may combine recovery, adapter, hardware, and media-service
surfaces, but they must stay proofs over already-closed Signal-owned boundary
contracts instead of inventing a new semantic authority.

### Rule 2: the shared lane must stay machine-readable

Integrated acceptance must not degrade into prose-only logs or human memory.
Later batches should expose typed descriptors, supervisor JSON, or explicit
Effigy grouping that explains what the integrated lane covers and why it
passes.

### Rule 3: required, advisory, and deferred depth must stay explicit

Signal must not hide longer-running soak or unstable backend depth inside the
bounded integrated lane. If a scenario blocks the shared acceptance claim, it
must be marked `required`. If it is useful but non-blocking, it must stay
`advisory`. If it is known but not yet stable or bounded, it must stay
`deferred`.

### Rule 4: integrated acceptance is repo-owned, not product-owned

Signal may support Loophole and other consumers, but the canonical harnesses,
descriptors, and tasks must remain in shared Signal crates and Effigy surfaces
rather than private product scripts or CI-only glue.

### Rule 5: multi-backend breadth must be explicit, not aspirational

The integrated lane must explicitly exercise widened adapter or host-path
breadth already closed in `g06`, instead of collapsing back to one plugin
format or one host path while still claiming cross-adapter confidence.

## Deferred scope

Batch 19.1 intentionally keeps the following outside the contract:

- exact task names and descriptor flags for the later integrated lane
- long-session soak thresholds, rerun counts, or promotion gates
- remote or farm-based scenario orchestration
- exhaustive platform certification across every host/backend combination
- product-local launch or release readiness beyond the reusable Signal surface

These may later gain additive Signal-owned surfaces, but they are not promised
by Batch 19.1.

## Batch 19.1 outcome

Batch 19.1 freezes the first reusable integrated acceptance policy for `g06`:

- Signal now has one authority line for fault-injection harness meaning and
  multi-backend integrated acceptance policy
- the bounded lane is explicitly split into `required`, `advisory`, and
  `deferred` depth instead of burying soak or unstable scenarios inside one
  vague acceptance claim
- the integrated lane is required to compose the already-closed recovery,
  adapter, hardware, media-service, and metadata boundary tasks rather than
  replace them with product-local harnesses
- later `g06.019` work can now implement one bounded integrated harness and
  descriptor family before `g06.020` widens into longer-session soak and
  promotion policy

## Batch 19.2 outcome

Batch 19.2 materializes the first runnable shared lane on top of this policy:

- `signal-supervisor-tools` now exposes the machine-readable
  `signal.runtime.integrated-acceptance-lane` descriptor
- Effigy now owns `effigy acceptance:integrated-acceptance-lane`, which groups
  the required cross-family path from the already-closed interruption,
  diagnostics, scheduling, plugin, portability, hardware, external-I/O,
  media-service, and analysis-metadata boundaries
- advisory depth is now explicit and inspectable instead of hidden:
  recording continuity, offline render continuity, VST3, AU, generic-event,
  and recall-portability remain visible but non-blocking
- the first grouped lane also flushed out and repaired stale watchdog-restart
  expectations in the public and internal interruption proofs, keeping the
  required lane aligned with the current runtime restart-threshold semantics
- this contract now has one real descriptor plus one runnable grouped task to
  build on before Batch 19.3 proves the integrated evidence is genuinely
  cross-family and not only a repackaged checklist

## Batch 19.3 outcome

Batch 19.3 turns the grouped lane into a true integrated-evidence surface:

- `signal-supervisor-tools` now proves one `signal.supervisor.export` artifact
  can carry recovery, deferred-work, adapter breadth, hardware, and media or
  analysis-library receipts together
- the integrated acceptance descriptor now names that cross-family export proof
  explicitly as part of the canonical validation spine
- `effigy acceptance:integrated-acceptance-lane` now runs the same proof,
  keeping the required lane anchored to combined receipts rather than a
  descriptor that only re-lists already-closed boundary tasks
- this closes the bounded integrated acceptance contract for `g06` and hands
  the next queue to `g06.020`, where the remaining problem becomes long-session
  soak, promotion policy, and Loophole-facing closeout depth

## Next Task

Continue `g06.020` with Batch 20.1 by freezing the bounded long-session soak,
promotion-gate, and Loophole-readiness policy on top of the now-closed
integrated acceptance lane.
