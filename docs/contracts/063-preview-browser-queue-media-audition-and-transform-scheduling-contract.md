# 063 Preview-Browser Queue, Media Audition, And Transform Scheduling Contract

Status: complete
Owner: core-product
Updated: 2026-03-21
Related contracts: `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`, `docs/contracts/062-preview-output-routing-audition-sink-and-low-latency-device-policy-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned preview-browser queue, media audition
orchestration, and transform-scheduling boundary so later preview workflow
depth widens one shared Signal contract instead of reopening browser-local
preview queues, editor-local audition schedulers, or app-specific transform
timing shells as the authority.

## Authority hierarchy

Preview-browser queueing, media audition orchestration, and transform
scheduling have one authority chain:

1. `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
   remains the authority for:
   - media identity, preview readiness, waveform readiness, and analysis-ready
     service meaning
   - the rule that queued preview claims must stay grounded in runtime-owned
     media-service truth instead of browser-local media ledgers
2. `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`
   remains the authority for:
   - preview-transform service class, readiness, degraded state, fallback, and
     artifact alignment
   - the rule that transform scheduling must widen from one shared preview
     vocabulary instead of inventing a second preview engine or editor-local
     transform queue
3. `docs/contracts/062-preview-output-routing-audition-sink-and-low-latency-device-policy-contract.md`
   remains the authority for:
   - preview-output routing posture, bounded audition-sink ownership, and
     low-latency device-policy class or outcome
   - the rule that preview-browser queue and audition scheduling depth must
     compose with the closed preview-device seam instead of replacing it
4. `signal-runtime` must own the canonical consumer-visible meaning for:
   - preview-browser queue posture, queue class, and bounded queue outcome
   - media audition orchestration posture, authority, and continuity outcome
   - transform-scheduling posture, authority, and bounded scheduling outcome
   - observation, supervisor, render-preview, and stable host-edge export
5. browsers, editors, and host crates may broker raw queue requests, transport
   hints, and bounded timing evidence into runtime-owned receipts, but they
   must not become the authority for:
   - a second preview queue taxonomy
   - app-local audition schedulers or browser-private playback ledgers as the
     consumer boundary
   - editor-local transform timing policy as shared truth

If a preview queue, audition orchestration, or transform-scheduling claim
cannot be explained through the closed media-service, preview-transform, and
preview-device seams plus runtime-owned receipts, it is not yet part of the
shared Signal contract.

## Existing anchors

Batch 12.1 freezes this contract on top of the currently closed preview and
workflow seams:

- `RuntimeMediaServiceSnapshot`
- `RuntimePreviewTransformServiceSnapshot`
- `RuntimeOfflineRenderContractPreview`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 12.1 does not claim these anchors already expose realized preview queue
or scheduling receipts. It freezes how later DTOs and proofs must widen from
them instead of inventing a separate browser-private or host-private preview
workflow model.

## Shared vocabulary

### Preview-browser queue

`preview-browser queue` means the runtime-owned bounded answer for how Signal
is tracking queued preview or audition requests that originate from browser,
media-browser, library, or bounded workflow surfaces.

This is not a browser widget queue, not a product-local playlist model, and
not an editor timeline arrangement.

### Queue posture

`queue posture` means the bounded category of preview queue behavior Signal is
currently using.

Batch 12.1 freezes the concept, not final implementation breadth, around:

- no runtime preview queue
- single-item preview queue
- ordered preview queue
- guarded preview queue
- unavailable preview queue

### Queue outcome

`queue outcome` means the runtime-owned result when queued preview intent is
projected onto the currently available preview-transform, media-service, and
preview-device state.

This outcome is distinct from preview-device routing outcome. It answers what
happened at the preview-workflow layer before delivery reaches the device seam.

### Media audition orchestration

`media audition orchestration` means the runtime-owned bounded answer for how
Signal is sequencing, advancing, guarding, or resuming media audition work
across preview-ready assets and currently active preview services.

This is not an editor-local transport model, not a browser-local playback
controller, and not app-specific audition glue.

### Audition orchestration authority

`audition orchestration authority` means where Signal is allowed to source the
currently active audition sequencing decision.

Batch 12.1 freezes the ownership line conceptually as:

- runtime default
- runtime declared
- workflow forwarded
- guarded runtime override

### Transform scheduling

`transform scheduling` means the runtime-owned bounded answer for how preview
transform work is being sequenced relative to queued preview demand and media
audition intent.

This policy is distinct from transform readiness. It explains how currently
available transform work is being scheduled at the preview-workflow layer.

### Scheduling outcome

`scheduling outcome` means the runtime-owned result when queued preview demand
is projected onto currently available transform service capacity and readiness.

This outcome is separate from transform fallback. It answers what happened at
the workflow-scheduling layer once queued preview intent was assessed against
available runtime transform capacity.

## Rules

### Rule 1: preview queue and audition scheduling meaning must stay runtime-owned

Browsers, editors, and products must not define their own queue, audition, or
transform-scheduling taxonomy for shared consumers.

### Rule 2: preview-browser depth must compose with media-service and preview-transform truth

Later preview-browser work must widen from the closed media-service and
preview-transform seams. It must not invent a second media preview scheduler
or editor-local transform queue.

### Rule 3: preview-device policy remains additive

Preview queueing and audition orchestration may influence preview-device use,
but they must still reduce to one shared preview-device answer through the
closed `062` seam instead of replacing it.

### Rule 4: browser and app-local workflow stay advisory

Browsers and products may provide queue requests, bounded queue ordering
hints, or user-trigger evidence, but the shared consumer-facing queue,
audition, and scheduling answers must stay typed and runtime-owned.

### Rule 5: UX semantics stay out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze browser panel design, playlist UX, editor timeline semantics, or
end-user queue controls.

## Deferred scope

Batch 12.1 intentionally leaves these out:

- realized runtime receipts for preview queue, audition orchestration, and
  transform scheduling depth
- public runtime, supervisor, and host-edge proof surfaces
- full browser-side queue editing, reordering, or session persistence UX
- richer remote audition transport, collaborative queue ownership, or cloud
  workflow semantics
- deeper transform prioritization, cancellation, or cost policy beyond the
  bounded scheduling vocabulary

## Batch 12.1 outcome

Batch 12.1 freezes the first reusable preview-workflow contract for Signal:

- preview-browser queueing, media audition orchestration, and transform
  scheduling now have one explicit Signal-owned authority line
- later runtime realization is forced to compose with the closed media-
  service, preview-transform, and preview-device seams instead of reopening
  browser-local queues, editor-local audition schedulers, or app-specific
  transform timing shells
- Batch 12.2 can now focus on materializing the first bounded receipt family
  instead of reopening which preview-workflow semantics belong to Signal

## Batch 12.2 outcome

Batch 12.2 materializes the first runtime-owned preview-workflow receipt
family on the existing preview-transform seam:

- `signal-runtime` now exposes bounded preview-browser queue, media audition
  orchestration, and transform-scheduling truth on
  `RuntimePreviewTransformServiceSnapshot`
- the same preview-workflow truth now flows through public runtime surfaces
  and stable local or server host-edge export without a browser-local queue or
  app-local audition scheduler shell
- the widened receipt family stays additive on top of the closed preview-
  transform and preview-device contracts instead of opening a second preview
  workflow report model

This still keeps richer browser queue editing, remote preview workflow, and
deeper transform prioritization out of scope, but it turns the bounded
preview-workflow contract into typed runtime evidence that Batch 12.3 can now
prove at the supervisor boundary.

## Batch 12.3 outcome

Batch 12.3 closes the bounded consumer seam on top of the Batch 12.2 runtime
receipt family:

- the existing `signal.runtime.preview-transform-boundary` now points at this
  preview-workflow contract instead of the narrower preview-device contract
  alone
- the machine-readable supervisor boundary explicitly describes runtime-owned
  preview-workflow queue and scheduling posture on observation, supervisor,
  render-preview, and offline-preview surfaces
- the repo-owned acceptance lane continues to reuse the focused public runtime
  and stable host-edge proofs, but now closes the preview-browser queue and
  transform-scheduling seam without a second preview-workflow acceptance shell

## Next Task

Open `g08.013` with Batch 13.1 by freezing the first runtime-owned
asset/session transform persistence, retention, and cache placement policy
contract on top of the closed preview-workflow seam.
