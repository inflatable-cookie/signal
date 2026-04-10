# 001 - g09 Lane-First Strict Adoption

Status: complete
Owner: core-product
Updated: 2026-04-10
Vision refs: docs/vision/001-signal-vision.md
Promotion targets: docs/architecture/product-guardrails.md, docs/contracts/001-working-rules.md
Roadmap refs: g09.014

## Problem

Signal's active `g09` work is contract-rich and implementation-heavy, but until
now it has relied on roadmap prose and logs alone. That makes it too easy for a
thread to drift, over-assume the next step, or lose the exact active boundary.

## Target Operating Model

Signal should keep the broader repo in a healthy baseline posture while using a
bounded strict surface on the live `g09` queue:

- explicit product guardrails
- explicit working rules
- one active strict-lane spec
- one bounded ready card only while the active `g09` seam is truly ready

## Current Posture

- current phase: `lane-first stricter adoption complete`
- scope: closed `g09` lane
- active milestone: none; `g09` is closed at the end of `g09.014`
- immediate follow-on boundary: next-generation planning boundary

## Goals

- install the minimum strict docs pack around the live `g09` lane
- bind each active `g09` batch to one explicit ready card while the generation
  is open
- return the lane to explicit next-generation planning once `g09` closes

## Non-Goals

- full repo-wide strict conversion
- historical backfill for older generations
- changing the substance of the active product roadmap

## Lane Plan

### Phase 1

- install `product-guardrails`, `001-working-rules`, and `docs/specs/`
- refresh the front doors to point at the strict lane

### Phase 2

- bind the reopened `g09.014` batch to one ready card
- keep the reopened production-readiness gate explicit until the generation can
  close honestly

## Open Questions

- none; the current migration tranche is intentionally bounded

## Promotion Plan

- durable cross-lane guardrails live in `docs/architecture/product-guardrails.md`
- durable execution policy for the strict lane lives in
  `docs/contracts/001-working-rules.md`
- do not let this spec become the only authority once those surfaces are in
  place

## Validation Strategy

- `effigy health`
- `effigy qa:docs`

## Stop Conditions

- the strict lane broadens into repo-wide migration
- active implementation no longer matches the live `g09` milestone

## Next Task

Re-enter planning at the next-generation boundary before promoting another
strict execution lane.
