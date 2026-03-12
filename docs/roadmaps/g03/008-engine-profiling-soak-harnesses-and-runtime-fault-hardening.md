# 008 - Engine Profiling, Soak Harnesses, And Runtime Fault Hardening

Status: planned
Owner: core-product
Created: 2026-03-12
Depends on: g03.001, g03.002, g03.003, g03.004, g03.005, g03.006, g03.007
Vision tags: `ENGINE`, `HARDENING`, `PERFORMANCE`

## Problem

Once the major engine surfaces exist, Signal needs profiling, soak, and fault
hardening that measures the real graph/runtime/plugin/render substrate rather
than protecting only narrow demos. Otherwise regressions will land exactly
where downstream products depend on long-running stability.

## Goals

- [ ] add reusable profiling and soak coverage around the deepened engine substrate
- [ ] harden runtime fault visibility and recovery across the major engine paths
- [ ] freeze a stronger acceptance spine for future product integrations

## Non-Goals

- [ ] no product-end workflow QA planning here
- [ ] no distributed deployment orchestration

## Execution Plan

### Batch 8.1 - Profiling And Soak Harnesses

- [ ] extend supervisor/runtime tools or dedicated harnesses for long-running routed engine cases
- [ ] surface performance counters that are actionable for engine work rather than only host glue

### Batch 8.2 - Fault Hardening

- [ ] validate recovery and degraded-state behavior across routing, plugin-chain, and offline-render paths
- [ ] pin acceptance reports strong enough to guard future engine changes

## Acceptance Criteria

- [ ] the deepened engine substrate has reusable profiling and soak coverage
- [ ] runtime fault and degraded-state reporting stays explicit across major engine paths
- [ ] future Signal work can build on a hardened acceptance spine

## Risks and Mitigations

- Risk: hardening work discovers missing foundation too late.
- Mitigation: use this milestone only after the earlier engine surfaces are explicitly landed.

## Evidence Requirements

- [ ] log each meaningful profiling or hardening tranche
- [ ] run focused soak/profiling/fault validation against the major engine paths
- [ ] record residual risk clearly if environment limits block full-duration runs

## Next Task

Close `g03` only after profiling and hardening prove the engine-oriented queue
is durable enough to hand off as a stable reusable substrate.
