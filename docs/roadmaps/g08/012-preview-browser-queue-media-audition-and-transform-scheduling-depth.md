# 012 - Preview-Browser Queue, Media Audition, And Transform Scheduling Depth

Status: active
Owner: core-product
Created: 2026-03-21
Depends on: g08.011
Vision tags: `PREVIEW`, `WORKFLOW`, `SCHEDULING`

## Problem

`g08.011` closes the bounded preview-device seam, but preview-browser queue
ownership, media-audition orchestration, and transform-scheduling depth are
still at risk of drifting into browser-local queues, editor-local transport,
or app-specific preview orchestration shells.

Without a runtime-owned contract here, later preview workflow depth will
either reopen media audition scheduling outside Signal-owned receipts or split
preview queue truth across browser, host, and transform services.

## Goals

- [ ] freeze one runtime-owned authority line for preview-browser queueing,
      media audition orchestration, and transform scheduling
- [ ] keep preview workflow composable with the closed preview-transform,
      preview-device, media-service, and deferred-work seams
- [ ] avoid browser-local preview queues or app-local audition schedulers
      becoming shared truth

## Non-Goals

- [ ] no product-local browser UX, editor timeline semantics, or end-user queue design
- [ ] no arbitrary browser-side scheduling or host-local preview workflow scripts as the shared contract

## Execution Plan

### Batch 12.1 - Preview Queue Contract

- [x] freeze runtime-owned preview-browser queue, media audition orchestration, and transform-scheduling meaning
- [x] define shared runtime versus browser-local authority explicitly

### Batch 12.2 - Runtime Preview Queue Baseline

- [x] materialize the first runtime-owned preview-browser queue, media audition orchestration, and transform-scheduling receipts
- [x] align stable host-edge export with the same bounded model

### Batch 12.3 - Consumer Proof

- [x] prove the widened preview-workflow seam through shared runtime, supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [ ] preview-browser queue, media audition orchestration, and transform scheduling are runtime-owned and inspectable
- [ ] browser-local or app-local preview workflow detail stays bounded and typed
- [ ] later preview and audition workflow work can build on one explicit queue and scheduling authority line

## Risks And Mitigations

- Risk: preview queueing drifts into browser-local state, app-specific audition schedulers, or host-private orchestration glue.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Batch 12.1 Outcome

- `g08` now has a frozen preview-workflow contract in
  `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
  instead of leaving preview-browser queueing, media audition orchestration,
  and transform scheduling as deferred prose under the older preview-transform
  and preview-device seams
- preview-browser queueing, media audition orchestration, and transform
  scheduling are now required to compose through the closed media-service,
  preview-transform, and preview-device seams rather than browser-local
  queues, editor-local audition schedulers, or app-specific transform timing
  shells
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until Batch 12.2 and Batch 12.3

## Batch 12.2 Outcome

- `signal-runtime` now widens the existing preview-transform seam with bounded
  preview-browser queue, media audition orchestration, and transform-
  scheduling truth instead of opening a second preview-workflow report family
- `RuntimePreviewTransformServiceSnapshot` now carries a typed
  `preview_workflow` summary covering queue posture, queue class, queue
  outcome, audition posture, audition authority, continuity outcome, and
  transform-scheduling posture, authority, and outcome
- the same preview-workflow truth now flows through public runtime surfaces
  and stable local or server host-edge export without a browser-local queue,
  editor-local transport scheduler, or host-local preview workflow shell

## Batch 12.3 Outcome

- the existing `signal.runtime.preview-transform-boundary` now points at
  `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
  and explicitly describes bounded preview-workflow queue and scheduling truth
  on observation, supervisor, render-preview, and offline-preview surfaces
- the machine-readable supervisor boundary now closes the bounded preview-
  workflow consumer seam through the existing public runtime and stable
  host-edge proof spine instead of introducing a preview-queue-only acceptance
  lane
- `g08.012` is now complete, and the next `g08` queue is asset/session
  transform persistence, retention, and cache placement policy

## Next Task

Open `g08.013` with Batch 13.1 by freezing the first runtime-owned
asset/session transform persistence, retention, and cache placement policy
contract on top of the closed preview-workflow seam.
