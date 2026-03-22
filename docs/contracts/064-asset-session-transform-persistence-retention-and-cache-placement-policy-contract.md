# 064 Asset/Session Transform Persistence, Retention, And Cache Placement Policy Contract

Status: complete
Owner: core-product
Updated: 2026-03-21
Related contracts: `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`, `docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md`, `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`, `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
Related architecture: `docs/architecture/graph-runtime-feature-reference.md`

## Purpose

Freeze the first runtime-owned asset/session transform persistence, retention,
and cache placement boundary so later preview and transform workflow depth
widen one shared Signal contract instead of reopening browser-local storage,
editor-local session ledgers, or host-private cache placement rules as the
authority.

## Authority hierarchy

Asset/session transform persistence, retention, and cache placement have one
authority chain:

1. `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
   remains the authority for:
   - media identity, indexing, invalidation, and preview readiness
   - the rule that transform persistence claims must stay grounded in
     runtime-owned media identity instead of browser-local storage ledgers
2. `docs/contracts/048-post-warp-render-cache-and-transform-artifact-contract.md`
   remains the authority for:
   - transform-artifact identity, readiness, invalidation, reuse, and
     degraded posture
   - the rule that later persistence and cache placement work must widen from
     one shared transform-artifact substrate instead of inventing a second
     cache shell
3. `docs/contracts/049-low-latency-audition-scrub-and-preview-transform-service-contract.md`
   remains the authority for:
   - preview-transform service class, readiness, degraded state, fallback, and
     bounded preview-facing transform behavior
   - the rule that persistence or retention policy must stay additive on the
     closed preview-transform seam instead of replacing it
4. `docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md`
   remains the authority for:
   - preview-browser queue, media audition orchestration, and transform-
     scheduling posture
   - the rule that persistence, retention, and cache placement must compose
     with the closed preview-workflow seam instead of introducing a second
     browser-local workflow ledger
5. `signal-runtime` must own the canonical consumer-visible meaning for:
   - asset/session transform persistence posture and scope
   - retention policy class, authority, and bounded retention outcome
   - cache placement posture, authority, and bounded cache-placement outcome
   - observation, supervisor, render-preview, and stable host-edge export
6. browsers, editors, host crates, and storage backends may broker raw cache
   locations, bounded storage hints, or session-trigger evidence into
   runtime-owned receipts, but they must not become the authority for:
   - a second transform-persistence taxonomy
   - host-local cache placement policy as the consumer boundary
   - browser-local storage or editor-local session ledgers as shared truth

If a transform persistence, retention, or cache placement claim cannot be
explained through the closed media-service, transform-artifact, preview-
transform, and preview-workflow seams plus runtime-owned receipts, it is not
yet part of the shared Signal contract.

## Existing anchors

Batch 13.1 freezes this contract on top of the currently closed transform and
preview seams:

- `RuntimeMediaPipelineSnapshot`
- `RuntimeMediaServiceSnapshot`
- `RuntimeTransformArtifactSnapshot`
- `RuntimePreviewTransformServiceSnapshot`
- `RuntimeOfflineRenderContractPreview`
- `RuntimeObservationReport`
- `RuntimeSupervisorReport`
- stable host-edge `supervisor_report()` export

Batch 13.1 does not claim these anchors already expose realized persistence,
retention, or cache-placement receipts. It freezes how later DTOs and proofs
must widen from them instead of inventing a separate browser-private or
host-private cache policy model.

## Shared vocabulary

### Asset/session transform persistence

`asset/session transform persistence` means the runtime-owned bounded answer
for how Signal is retaining or reusing transform-related state across media
assets, session scope, and bounded workflow lifetimes.

This is not a browser-local storage record, not an editor session save model,
and not a host-private cache manifest.

### Persistence posture

`persistence posture` means the bounded category of transform persistence
behavior Signal is currently using.

Batch 13.1 freezes the concept, not final implementation breadth, around:

- no transform persistence
- asset-scoped transform persistence
- session-scoped transform persistence
- guarded transform persistence
- unavailable transform persistence

### Retention policy

`retention policy` means the runtime-owned bounded answer for how long or
under what scope transform state is intended to remain reusable.

This policy is separate from transform readiness. It answers what reuse window
Signal intends to preserve at the persistence layer.

### Retention outcome

`retention outcome` means the runtime-owned result when persistence intent is
projected onto the currently available transform-artifact, media, and preview
workflow state.

This outcome is distinct from transform-artifact reuse state. It answers what
happened at the persistence or retention layer once Signal assessed whether
cached transform state should survive or be discarded.

### Cache placement

`cache placement` means the runtime-owned bounded answer for where Signal is
allowed to keep reusable transform state relative to runtime-owned cache and
session scope.

This is not a host filesystem policy, not a browser storage API decision, and
not a product-local cache browser.

### Cache-placement outcome

`cache-placement outcome` means the runtime-owned result when persistence and
retention intent are projected onto the currently available cache substrate.

This outcome is separate from preview-workflow queueing. It answers what
happened at the placement layer once Signal determined where reusable
transform state is allowed to live.

## Rules

### Rule 1: persistence and cache-placement meaning must stay runtime-owned

Browsers, editors, hosts, and products must not define their own persistence,
retention, or cache-placement taxonomy for shared consumers.

### Rule 2: persistence depth must compose with transform-artifact truth

Later persistence work must widen from the closed transform-artifact and
preview-transform seams. It must not invent a second transform store or
host-private cache authority.

### Rule 3: preview-workflow policy remains additive

Persistence, retention, and cache placement may influence preview workflow,
but they must still reduce to one shared preview-workflow answer through the
closed `063` seam instead of replacing it.

### Rule 4: browser and host storage hints stay advisory

Browsers, editors, and hosts may provide bounded storage hints, cache roots,
or session-trigger evidence, but the shared consumer-facing persistence,
retention, and placement answers must stay typed and runtime-owned.

### Rule 5: UX and storage-backend detail stay out of scope

This contract freezes reusable runtime and adapter meaning only. It does not
freeze browser persistence UX, editor save workflow, filesystem layout, cloud
sync policy, or end-user cache controls.

## Deferred scope

Batch 13.1 intentionally leaves these out:

- realized runtime receipts for transform persistence, retention, and cache
  placement depth
- public runtime, supervisor, and host-edge proof surfaces
- full browser-side session persistence UX or product-local save workflow
- richer cloud or collaborative storage policy
- deeper eviction, archival, or quota policy beyond the bounded vocabulary

## Batch 13.1 outcome

Batch 13.1 freezes the first reusable transform-persistence contract for
Signal:

- asset/session transform persistence, retention, and cache placement now have
  one explicit Signal-owned authority line
- later runtime realization is forced to compose with the closed media-
  service, transform-artifact, preview-transform, and preview-workflow seams
  instead of reopening browser-local storage, editor-local session ledgers, or
  host-private cache placement rules
- Batch 13.2 can now focus on materializing the first bounded receipt family
  instead of reopening which persistence and cache semantics belong to Signal

## Batch 13.2 outcome

Batch 13.2 materializes the first runtime-owned transform-persistence receipt
family on the existing transform-artifact seam:

- `signal-runtime` now exposes bounded persistence posture, retention policy,
  and cache-placement truth on `RuntimeTransformArtifactSnapshot`
- the same transform-persistence truth now flows through public runtime
  surfaces and stable local or server host-edge export without a browser-local
  storage ledger or host-local cache policy shell
- the widened receipt family stays additive on top of the closed transform-
  artifact and preview-workflow contracts instead of opening a second cache
  policy report model

This still keeps richer session persistence UX, cloud sync policy, and deeper
eviction or quota policy out of scope, but it turns the bounded persistence
contract into typed runtime evidence that Batch 13.3 can now prove at the
supervisor boundary.

## Batch 13.3 outcome

Batch 13.3 closes the bounded transform-persistence consumer seam by widening
the existing shared transform-artifact boundary instead of opening a second
persistence-only acceptance lane:

- `signal-supervisor-tools` now points
  `signal.runtime.transform-artifact-boundary` at this `064` contract instead
  of the older `048` contract
- the machine-readable boundary explicitly describes
  `transform_persistence`, `persistence_posture`, `retention_outcome`, and
  `cache_placement_outcome` alongside the earlier transform-artifact anchors
- the repo-owned proof path remains
  `effigy acceptance:transform-artifact-boundary`, so runtime, supervisor,
  clip-render, offline preview, and both stable host edges continue to close
  on one shared seam without a browser-local storage ledger or host-local
  cache-policy shell

## Next Task

Continue `g08.014` with Batch 14.1 by freezing the first runtime-owned live
external MIDI device ownership and backend parity contract on top of the
closed transform-persistence seam.
