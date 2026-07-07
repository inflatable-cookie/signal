# 025 - Stretch Product Workflow Contract Checkpoint

Status: deferred
Owner: core-product
Created: 2026-07-07
Depends on: g10.021, g10.022, g10.023, g10.024
Vision tags: `STRETCH`, `CONTRACTS`, `INTEGRATION`

## Problem

Signal owns stretch DSP and render/cache/export mechanics. Loophole product
integration should not drive Signal internals prematurely, but product
workflows will eventually need a narrow contract for tier selection, ratio,
pitch, markers, projection, latency, cache behavior, and artifact receipts.

## Goals

- [ ] define product-visible tier selection: Repitch, RealtimePreview,
  OfflineHighQuality
- [ ] define ratio, pitch, marker, projection, latency, and cache behavior
  exposed to consumers
- [ ] map export/freeze/cache workflows to Signal-owned artifact receipts
- [ ] record Loophole integration planning in Chorus when the product workflow
  is ready

## Execution Plan

### Batch 25.1 - Signal Contract Surface

- [ ] publish only the consumer contract needed by product workflows
- [ ] keep DSP policy and implementation inside Signal

### Batch 25.2 - Loophole Integration Planning

- [ ] record Chorus roadmap/spec work for integration only after the product
  workflow is ready

## Acceptance Criteria

- [ ] Signal remains the DSP owner
- [ ] Pulse/Aura changes are narrow contract additions, not duplicate DSP
  policy
- [ ] Chorus planning describes integration only, not Signal internals as a
  blocker

## Churn Guardrails

- [ ] do not add more receipt, fixture, or promotion-evidence shapes unless a
  real report artifact, cache consumer, or product workflow needs them
- [ ] do not spend another batch tightening accepted/rejected policy tests
  unless behavior changes
- [ ] do not treat the current synthetic report as Rubber Band-class evidence;
  it is a fast local gate only
- [ ] prefer DSP quality, real corpus evidence, bounded-memory rendering, or
  RealtimePreview over docs or fixture polish

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`
- focused Signal contract tests once a product-facing contract changes

## Progress

- 2026-07-07: created as a deferred checkpoint so product integration planning
  has an explicit home without blocking Signal DSP work or forcing Chorus work
  too early.

## Next Task

Keep this deferred until a Loophole product workflow consumes the Signal-owned
stretch contract.
