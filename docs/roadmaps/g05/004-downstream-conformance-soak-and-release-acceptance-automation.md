# 004 - Downstream Conformance Soak And Release-Acceptance Automation

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g05.001, g05.002, g05.003
Vision tags: `CONFORMANCE`, `AUTOMATION`, `ACCEPTANCE`

## Problem

`g04` closed with a focused runnable conformance matrix, but the next stage of
consumer confidence requires broader and longer-running acceptance automation
that still stays shared, repo-owned, and contract-driven.

Without a dedicated downstream automation milestone:

- consumer confidence will rely on one narrow matrix plus ad hoc downstream
  testing
- packaging and release claims will not be exercised under longer-running or
  broader consumer scenarios
- backend breadth and host-edge decisions will not get a shared acceptance
  spine
- future regressions will be caught only after downstream breakage

## Goals

- [ ] define the next layer of downstream conformance and release automation
- [ ] keep soak and acceptance automation repo-owned rather than app-local
- [ ] exercise widened backend, host-edge, and release claims together
- [ ] avoid turning Signal into a downstream orchestration repository

## Non-Goals

- [ ] no consumer-specific CI ownership
- [ ] no app-local workflow automation living in Signal
- [ ] no indefinite benchmark farm or fleet orchestration

## Execution Plan

### Batch 4.1 - Automation Contract

- [x] define which longer-running conformance and release checks belong in the
  shared automation boundary
- [x] separate mandatory release automation from optional soak depth

### Batch 4.2 - Shared Automation Fixtures

- [x] implement the first broader repo-owned conformance and release fixtures
- [x] keep outputs typed and inspectable rather than log-scraping only

### Batch 4.3 - Failure-Gate Policy

- [x] define the first credible fail-gate policy for widened consumer/release
  automation
- [x] keep deferred or expensive checks explicit when they stay out of the fast
  path

## Progress Notes

- 2026-03-12: seeded `g05.004` so downstream confidence grows as shared Signal
  automation rather than consumer-specific test sprawl.
- 2026-03-13: activated `g05.004` after `g05.003` closed with explicit
  backend-neutral breadth, shared host-edge, and publication packaging
  boundaries plus runnable consumer-facing proofs for each.
- 2026-03-13: completed Batch 4.1 by freezing contract `011`, separating the
  bounded mandatory release-acceptance fast path from optional soak/confidence
  depth, and explicitly anchoring both tiers to existing Signal-owned receipts,
  descriptors, and Effigy tasks.
- 2026-03-13: completed Batch 4.2 by adding
  `signal-supervisor-tools --describe-downstream-automation`, wiring bounded
  `effigy acceptance:downstream-release`, optional typed-depth
  `effigy acceptance:downstream-depth`, and the combined
  `effigy acceptance:downstream-automation` task so broader shared
  automation now produces machine-readable boundary output and typed scenario
  export rather than log-only review.
- 2026-03-13: completed Batch 4.3 by adding
  `signal-supervisor-tools --describe-downstream-fail-gates`, wiring
  `effigy acceptance:downstream-gate`, and making the first required,
  advisory, and deferred downstream automation states explicit, including the
  currently deferred `server soak` fixture.

## Acceptance Criteria

- [x] Signal has a broader shared conformance and release automation boundary
- [x] widened backend, host-edge, and packaging claims are exercised together
- [x] optional versus mandatory automation depth is explicit

## Risks And Mitigations

- Risk: downstream automation becomes expensive sprawl.
- Mitigation: keep one shared acceptance vocabulary and explicit fail-gate rules.
- Risk: soak work hides contract drift instead of surfacing it.
- Mitigation: require typed receipts or descriptors for the broader checks.

## Evidence Requirements

- [x] log each meaningful automation tranche
- [x] run focused validation for widened acceptance or soak fixtures
- [x] record deferred automation depth that still remains out of scope

## Next Task

COMPLETE. `g05.004` closed on 2026-03-13 after the downstream automation
contract, typed fixtures, and fail-gate policy landed. Continue with `g05.005`
Batch 5.1.
