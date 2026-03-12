# 006 - Consumer Conformance, Export Stability, And Release Packaging

Status: complete
Owner: core-product
Created: 2026-03-12
Depends on: g04.001, g04.004, g04.005
Vision tags: `CONFORMANCE`, `RELEASE`, `CONSUMERS`

## Problem

Once Signal’s public/runtime boundaries, scheduling model, deferred services,
hardware contracts, and plugin boundaries are explicit, the repo still needs a
final generation-level milestone that proves consumers can rely on them and
that Signal can be packaged and released as a stable shared project.

## Goals

- [ ] add conformance surfaces for downstream consumers such as Loophole and Finch
- [ ] freeze export/report stability expectations at the consumer boundary
- [ ] define the first credible release-packaging and versioning workflow for Signal

## Non-Goals

- [ ] no downstream consumer release orchestration
- [ ] no crates.io publication obligation unless the repo is ready for it

## Execution Plan

### Batch 6.1 - Consumer Conformance Matrix

- [x] define which fixtures, exports, or examples prove consumer-facing stability
- [x] make the chosen matrix runnable without reading private implementation detail

### Batch 6.2 - Release Packaging Baseline

- [x] define versioning, changelog, packaging, and artifact expectations for the
  stabilized Signal boundary
- [x] record what remains intentionally unstable after the first packaging pass

### Batch 6.3 - Generation Closeout Proof

- [x] validate the conformance and release boundary with focused evidence
- [x] capture residual risks and the next likely post-`g04` queue

## Progress Notes

- 2026-03-12: completed Batch 6.1 by defining the first runnable consumer
  conformance matrix around the stabilized runtime/export/plugin boundary,
  surfacing it through `signal-supervisor-tools --describe-conformance-matrix`,
  and wiring the same matrix into `effigy acceptance:conformance` so consumers
  can inspect and execute the shared proof set without private implementation
  detail.
- 2026-03-12: completed Batch 6.2 by defining the first host-free release
  boundary baseline in `signal-supervisor-tools --describe-release-boundary`,
  wiring the same baseline into `effigy acceptance:release-boundary`, and
  making the release version source, changelog path, required boundary
  descriptions, validation steps, and intentionally unstable scopes explicit.
- 2026-03-12: completed Batch 6.3 by adding
  `signal-supervisor-tools --describe-generation-closeout`, wiring the
  combined closeout proof into `effigy acceptance:g04-closeout`, and recording
  the post-`g04` queue explicitly in
  `docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md`
  instead of leaving the next queue implicit.

## Acceptance Criteria

- [x] Signal has explicit consumer conformance coverage
- [x] release and packaging policy is no longer implicit
- [x] the repo can open a later queue from a stabilized shared-project boundary

## Risks and Mitigations

- Risk: release planning arrives before the underlying boundaries are mature.
- Mitigation: depend on the earlier `g04` milestones and keep this as closeout work.
- Risk: consumer conformance becomes downstream-specific integration sprawl.
- Mitigation: keep the matrix to shared boundary proofs only.

## Evidence Requirements

- [x] log each meaningful conformance or packaging tranche
- [x] run focused validation against the chosen consumer-facing boundary
- [x] record the next-generation candidate queue explicitly

## Next Task

COMPLETE. `g04.006` closed on 2026-03-12 after the combined conformance,
release-boundary, and generation-closeout proof landed. The next likely queue
is recorded in
`docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md`.
