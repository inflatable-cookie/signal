# 019 - Multichannel, Linux, Time-Stretch, And Control-Surface Acceptance Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g07.006, g07.008, g07.010, g07.014, g07.018
Vision tags: `ACCEPTANCE`, `LINUX`, `STRETCH`

## Problem

The widened `g07` feature surface will not be trustworthy unless Signal proves
it coherently across multichannel, Linux, controller, and transform scenarios.

## Goals

- [ ] add integrated acceptance depth for the widened `g07` feature surface
- [ ] exercise multichannel, Linux, control-surface, and stretch behavior coherently
- [ ] gather useful runtime receipts instead of only pass/fail outcomes

## Non-Goals

- [ ] no final release gate here
- [ ] no app-local acceptance harnesses

## Execution Plan

### Batch 19.1 - Acceptance Scope

- [ ] map evidence needs across routing, Linux, control, and transform depth
- [ ] define the useful summaries and receipts needed for later promotion

### Batch 19.2 - Harness Depth

- [ ] implement focused integrated acceptance harnesses as needed
- [ ] keep outputs machine-readable and repo-owned

### Batch 19.3 - Readiness Proof

- [ ] confirm the widened feature surface has coherent downstream-ready evidence

## Acceptance Criteria

- [ ] integrated acceptance covers the most important `g07` feature fronts
- [ ] outputs are useful runtime evidence rather than isolated demos
- [ ] later closeout work can rely on the same acceptance substrate

## Risks And Mitigations

- Risk: acceptance work becomes expensive integration sprawl.
- Mitigation: keep scenarios bounded and tie them to the generation's core claims.

## Evidence Requirements

- [ ] log each meaningful acceptance tranche
- [ ] run focused integrated acceptance validation
- [ ] record any deferred acceptance lanes explicitly

## Next Task

Continue `g07.020` by turning the widened feature and acceptance surface into a
clear Loophole-facing feature-readiness closeout gate.

