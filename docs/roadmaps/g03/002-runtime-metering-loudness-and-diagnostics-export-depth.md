# 002 - Runtime Metering, Loudness, And Diagnostics Export Depth

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g03.001
Vision tags: `ENGINE`, `DIAGNOSTICS`, `METERING`

## Problem

Signal has runtime observation and `g02` has reusable loudness analysis, but
the engine does not yet expose a deliberate metering and loudness pipeline tied
to routed mixer topology. Without that, products will keep inventing their own
meter summaries and diagnostics views around the same engine state.

## Goals

- [x] project routed peak/RMS and loudness-oriented meter state from the runtime
- [x] reuse Signal-owned loudness primitives instead of duplicating metering logic in hosts
- [x] make diagnostics export strong enough for both local monitoring and soak tooling

## Non-Goals

- [x] no final product-specific meter UX breadth
- [x] no mastering-grade offline report suite yet

## Execution Plan

### Batch 2.1 - Metering Contract

- [x] define reusable meter and loudness snapshot surfaces on top of the mixer graph summary
- [x] separate realtime-safe meter accumulation from heavier report/export preparation

### Batch 2.2 - Runtime And Export Proof

- [x] thread meter snapshots through `signal-runtime` and supervisor-facing exports
- [x] validate that routed track, return, and main-output paths expose coherent diagnostic summaries

## Acceptance Criteria

- [x] meter state and loudness-oriented summaries are reusable Signal exports
- [x] runtime and supervisor tools can observe the same routed meter vocabulary
- [x] hosts no longer need to infer meter ownership from unrelated engine fields

## Risks and Mitigations

- Risk: diagnostics and metering drift into separate incompatible vocabularies.
- Mitigation: treat meter export as one typed runtime-owned contract.

## Evidence Requirements

- [x] log the metering/export tranche
- [x] run focused runtime and supervisor-tool validation for routed meter cases
- [x] note any deferred high-cost offline reporting scope explicitly

## Next Task

Execute `g03.003` by deepening automation playback semantics across routed
engine targets and proving deterministic multi-block control playback through
`signal-graph` and `signal-runtime`.
