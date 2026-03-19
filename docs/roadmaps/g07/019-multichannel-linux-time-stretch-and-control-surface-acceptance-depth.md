# 019 - Multichannel, Linux, Time-Stretch, And Control-Surface Acceptance Depth

Status: complete
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

- [x] map evidence needs across routing, Linux, control, and transform depth
- [x] define the useful summaries and receipts needed for later promotion

### Batch 19.2 - Harness Depth

- [x] implement focused integrated acceptance harnesses as needed
- [x] keep outputs machine-readable and repo-owned

### Batch 19.3 - Readiness Proof

- [x] confirm the widened feature surface has coherent downstream-ready evidence

## Acceptance Criteria

- [x] integrated acceptance covers the most important `g07` feature fronts
- [x] outputs are useful runtime evidence rather than isolated demos
- [x] later closeout work can rely on the same acceptance substrate

## Risks And Mitigations

- Risk: acceptance work becomes expensive integration sprawl.
- Mitigation: keep scenarios bounded and tie them to the generation's core claims.

## Evidence Requirements

- [ ] log each meaningful acceptance tranche
- [ ] run focused integrated acceptance validation
- [ ] record any deferred acceptance lanes explicitly

## Batch 19.1 Outcome

- Signal now has a bounded integrated acceptance contract for the widened
  multichannel, Linux, controller, and stretch seams instead of a loose set of
  milestone-local proofs with no shared grouping policy.
- the required, advisory, and deferred acceptance tiers are now explicit, which
  keeps later harness work from quietly promoting unstable matrices,
  product-local flows, or broader closeout policy into the shared lane.
- Batch 19.2 can now materialize the first grouped descriptor and repo-owned
  acceptance lane without reopening which feature families belong in the shared
  `g07` acceptance surface.

## Batch 19.2 Outcome

- `signal-supervisor-tools` now owns the first grouped machine-readable
  `g07` acceptance lane descriptor instead of leaving widened acceptance depth
  as a prose-only contract or a loose list of boundary tasks.
- Effigy now owns `acceptance:g07-integrated-acceptance-lane` as the repo-owned
  grouped rerun lane across routing, Linux, controller, and stretch families.
- Batch 19.3 can now focus on proving that the grouped lane carries meaningful
  cross-family evidence rather than only re-listing the component boundary
  tasks.

## Batch 19.3 Outcome

- the grouped `g07` acceptance lane now proves one machine-readable supervisor
  export can carry routing, Linux backend, control-surface or advanced-hardware,
  and stretch or transform receipts together instead of only replaying the
  component boundary tasks.
- `signal-supervisor-tools` and Effigy now keep that cross-family export proof
  inside the repo-owned `g07` lane, which makes later closeout work rely on one
  coherent acceptance substrate instead of separate milestone-local checks.
- `g07.019` is now complete, and Batch 20.1 can freeze the final closeout scope
  and Loophole-facing readiness gate without reopening grouped acceptance depth.

## Next Task

Continue `g08.001` with Batch 1.1 by freezing the runtime-owned live Linux
audio backend ownership and session-lifecycle contract before deeper ALSA,
JACK, and PipeWire runtime realization widens.
