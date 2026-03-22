# 013 - Asset/Session Transform Persistence, Retention, And Cache Placement Policy

Status: complete
Owner: core-product
Created: 2026-03-21
Depends on: g08.012
Vision tags: `PREVIEW`, `PERSISTENCE`, `CACHE`

## Problem

`g08.012` closes the bounded preview-workflow seam, but asset/session
transform persistence, retention, and cache placement policy are still at risk
of drifting into browser-local storage, editor-local session ledgers, or
host-private cache placement rules.

Without a runtime-owned contract here, later transform workflow depth will
either reopen cache and retention policy outside Signal-owned receipts or
split persistence truth across browser, host, and transform services.

## Goals

- [ ] freeze one runtime-owned authority line for asset/session transform
      persistence, retention, and cache placement
- [ ] keep persistence policy composable with the closed transform-artifact,
      preview-workflow, media-service, and deferred-work seams
- [ ] avoid browser-local storage or host-local cache policy becoming shared
      truth

## Non-Goals

- [ ] no product-local browser UX, editor timeline persistence UX, or end-user cache controls
- [ ] no arbitrary browser-side storage policy or host-local cache scripts as the shared contract

## Execution Plan

### Batch 13.1 - Persistence Policy Contract

- [x] freeze runtime-owned asset/session transform persistence, retention, and cache-placement meaning
- [x] define shared runtime versus browser-local or host-local authority explicitly

### Batch 13.2 - Runtime Persistence Policy Baseline

- [x] materialize the first runtime-owned persistence, retention, and cache-placement receipts
- [x] align stable host-edge export with the same bounded model

### Batch 13.3 - Consumer Proof

- [x] prove the widened persistence-policy seam through shared runtime, supervisor, and stable host-edge surfaces

## Acceptance Criteria

- [x] asset/session transform persistence, retention, and cache placement are runtime-owned and inspectable
- [x] browser-local or host-local persistence detail stays bounded and typed
- [x] later transform and preview workflow work can build on one explicit persistence and cache-policy authority line

## Risks And Mitigations

- Risk: transform persistence and cache placement drift into browser-local storage, host-private cache policy, or app-specific retention glue.
- Mitigation: freeze one runtime-owned contract before widening runtime realization.

## Evidence Requirements

- [ ] log each meaningful tranche
- [ ] run focused validation after the runtime baseline lands
- [ ] record the next milestone step explicitly

## Batch 13.1 Outcome

- `g08` now has a frozen transform-persistence contract in
  `docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
  instead of leaving asset/session transform persistence, retention, and cache
  placement as deferred prose under the older transform-artifact and preview
  seams
- asset/session transform persistence, retention, and cache placement are now
  required to compose through the closed media-service, transform-artifact,
  preview-transform, and preview-workflow seams rather than browser-local
  storage, editor-local session ledgers, or host-private cache policy
- runtime receipts, supervisor proof, and stable host-edge export remain
  explicitly deferred until Batch 13.2 and Batch 13.3

## Batch 13.2 Outcome

- `signal-runtime` now widens the existing transform-artifact seam with
  bounded asset/session transform persistence, retention, and cache-placement
  truth instead of opening a second cache-policy report family
- `RuntimeTransformArtifactSnapshot` now carries a typed
  `transform_persistence` summary covering persistence posture, retention
  policy class, retention authority and outcome, plus cache-placement posture,
  authority, and outcome
- the same transform-persistence truth now flows through public runtime
  surfaces and stable local or server host-edge export without a browser-local
  storage ledger or host-local cache policy shell

## Batch 13.3 Outcome

- `signal-supervisor-tools` now widens the existing
  `signal.runtime.transform-artifact-boundary` so it proves the bounded
  transform-persistence seam on the same shared supervisor descriptor instead
  of opening a second persistence-only acceptance lane
- the shared boundary now points at
  `docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md`
  and explicitly describes `transform_persistence`, `persistence_posture`,
  `retention_outcome`, and `cache_placement_outcome` alongside the earlier
  transform-artifact anchors
- `g08.013` is now complete, and the next queue is live external MIDI device
  ownership and backend parity depth

## Next Task

Continue `g08.014` with Batch 14.1 by freezing the first runtime-owned live
external MIDI device ownership and backend parity contract on top of the
closed transform-persistence seam.
