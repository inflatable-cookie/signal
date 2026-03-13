# 020 - Long-Session Soak, Promotion Gate, And Loophole-Readiness Closeout

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.019
Vision tags: `ACCEPTANCE`, `SOAK`, `CLOSEOUT`

## Problem

`g06` will only be worth the planning cost if it closes with stronger
long-session evidence and a clear answer for whether the widened Signal runtime
actually moved Loophole forward on both hardening and feature breadth.

## Goals

- [ ] define the final `g06` soak and promotion gate
- [ ] combine runtime recovery, profiling, plugin breadth, hardware, and media
  evidence into one closeout surface
- [ ] make Loophole-facing readiness explicit rather than implicit

## Non-Goals

- [ ] no product launch-readiness review outside Signal's reusable boundary
- [ ] no remote/distributed profile generation closeout yet

## Execution Plan

### Batch 20.1 - Soak And Promotion Scope

- [ ] define the bounded long-session soak expectations and promotion criteria
- [ ] decide which `g06` evidence is required, advisory, or deferred

### Batch 20.2 - Closeout Surface

- [ ] implement the combined `g06` closeout descriptor, task, and receipts
- [ ] keep the outputs machine-readable and downstream-consumable

### Batch 20.3 - Readiness Review

- [ ] review the generation against Loophole-facing runtime and feature-depth needs
- [ ] record the next backlog or generation handoff clearly

## Acceptance Criteria

- [ ] `g06` has bounded long-session soak evidence
- [ ] the widened runtime and feature surface is summarized through one closeout gate
- [ ] Loophole-facing readiness is explicit enough to guide the next Signal generation

## Risks And Mitigations

- Risk: closeout becomes a vague summary instead of a gate.
- Mitigation: require typed receipts and required-versus-advisory policy.
- Risk: generation claims outrun actual reusable evidence.
- Mitigation: tie the final gate to the integrated acceptance and soak receipts only.

## Evidence Requirements

- [ ] log each meaningful closeout tranche
- [ ] run the final closeout and soak validation tasks actually used for promotion
- [ ] record the next backlog or generation handoff explicitly

## Next Task

COMPLETE `g06` only when the combined soak and readiness gate is real, then
either promote the next generation or hand the remaining deferred scope back
into backlog explicitly.
