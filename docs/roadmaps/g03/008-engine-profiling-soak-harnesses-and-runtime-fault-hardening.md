# 008 - Engine Profiling, Soak Harnesses, And Runtime Fault Hardening

Status: complete
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

- [x] add reusable profiling and soak coverage around the deepened engine substrate
- [x] harden runtime fault visibility and recovery across the major engine paths
- [x] freeze a stronger acceptance spine for future product integrations

## Non-Goals

- [ ] no product-end workflow QA planning here
- [ ] no distributed deployment orchestration

## Execution Plan

### Batch 8.1 - Profiling And Soak Harnesses

- [x] extend supervisor/runtime tools or dedicated harnesses for long-running routed engine cases
- [x] surface performance counters that are actionable for engine work rather than only host glue

### Batch 8.2 - Fault Hardening

- [x] validate recovery and degraded-state behavior across routing, plugin-chain, and offline-render paths
- [x] pin acceptance reports strong enough to guard future engine changes

## Progress Notes

- 2026-03-12: opened `g03.008` and completed Batch 8.1 by adding
  runtime-owned profiling and soak receipts derived from supervisor/host
  observation surfaces, wiring `signal-host-local` soak coverage to assert
  those receipts directly, and extending `signal-supervisor-tools` export so
  long-running routed scenarios can carry typed profiling/soak counters
  without rebuilding benchmark state in tool-local code.
- 2026-03-12: completed Batch 8.2 and closed `g03.008` by extending the live
  profiling/soak receipts with routing and plugin-chain degradation boundary
  counts, adding typed offline-render profiling/soak receipts, and proving
  routing gate, quarantined plugin-chain, and delegated-unavailable offline
  render cases through those runtime-owned receipt surfaces instead of raw
  snapshot-only assertions.

## Acceptance Criteria

- [x] the deepened engine substrate has reusable profiling and soak coverage
- [x] runtime fault and degraded-state reporting stays explicit across major engine paths
- [x] future Signal work can build on a hardened acceptance spine

## Risks and Mitigations

- Risk: hardening work discovers missing foundation too late.
- Mitigation: use this milestone only after the earlier engine surfaces are explicitly landed.

## Evidence Requirements

- [x] log each meaningful profiling or hardening tranche
- [x] run focused soak/profiling/fault validation against the major engine paths
- [x] record residual risk clearly if environment limits block full-duration runs

## Next Task

COMPLETE. `g03.008` closed on 2026-03-12 after the runtime-owned profiling,
soak, and offline-render hardening receipts were proven across the major engine
fault paths. Open the next generation only when maintainers want the next
engine queue.
