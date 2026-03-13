# 019 - Fault-Injection Harnesses And Multi-Backend Acceptance Depth

Status: planned
Owner: core-product
Created: 2026-03-13
Depends on: g06.005, g06.011, g06.016, g06.018
Vision tags: `ACCEPTANCE`, `RECOVERY`, `BACKENDS`

## Problem

`g06` will widen both runtime hardening and actual feature breadth. The
generation therefore needs a stronger reusable acceptance surface that can
inject realistic faults and prove behavior across multiple adapters and
runtime-service lanes.

## Goals

- [ ] define reusable fault-injection and integrated acceptance scenarios for
  the widened `g06` surface
- [ ] cover recovery, adapter breadth, hardware, and media-service behavior in
  one stronger acceptance lane
- [ ] keep the evidence machine-readable and repo-owned

## Non-Goals

- [ ] no product-specific acceptance dashboards
- [ ] no full certification matrix across every environment

## Execution Plan

### Batch 19.1 - Harness Scope Contract

- [ ] define the key fault-injection and multi-backend acceptance scenarios
- [ ] separate required acceptance depth from optional longer-running soak paths

### Batch 19.2 - Harness Implementation

- [ ] implement reusable fault-injection fixtures and acceptance tasks
- [ ] keep outputs typed through supervisor tools and Effigy surfaces

### Batch 19.3 - Integrated Evidence Proof

- [ ] add focused proofs that the widened runtime and adapter surface now has
  meaningful integrated evidence rather than only milestone-local checks

## Acceptance Criteria

- [ ] Signal has reusable fault-injection and integrated acceptance depth
- [ ] multi-backend, hardware, and media-service behavior have cross-cutting evidence
- [ ] later closeout and downstream consumers can rely on typed acceptance receipts

## Risks And Mitigations

- Risk: acceptance breadth becomes vague integration sprawl.
- Mitigation: freeze a bounded scenario set with required versus optional depth.
- Risk: harnesses depend on private scripts or local operator steps.
- Mitigation: keep outputs repo-owned, typed, and runnable through Effigy/tasks.

## Evidence Requirements

- [ ] log each meaningful fault-injection tranche
- [ ] run focused integrated acceptance validation
- [ ] record explicit deferred soak depth that remains optional

## Next Task

Continue `g06.020` by combining the widened runtime, feature, and acceptance
evidence into one Loophole-facing readiness closeout.
