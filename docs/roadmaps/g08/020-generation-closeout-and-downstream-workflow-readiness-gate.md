# 020 - Generation Closeout And Downstream Workflow Readiness Gate

Status: complete
Owner: core-product
Created: 2026-03-22
Depends on: g08.019
Vision tags: `CLOSEOUT`, `READINESS`, `WORKFLOW`

## Problem

`g08.019` closes the shared integrated live-ownership and workflow acceptance
seam, but the generation still lacks one explicit closeout and downstream
workflow readiness gate.

Without that final gate, `g08` would end as a pile of completed acceptance
lanes without one repo-owned verdict about what later downstream work may now
assume safely and what remains intentionally deferred.

## Goals

- [x] define the generation-closeout and downstream workflow readiness gate
      for `g08`
- [x] tie readiness claims to the completed `g08.019` integrated seam
- [x] keep deferred workflow, environment, and product-local depth explicit

## Non-Goals

- [x] no new feature expansion here
- [x] no product-launch or distribution verdict detached from runtime evidence

## Execution Plan

### Batch 20.1 - Closeout Scope

- [x] freeze the shared generation closeout and downstream workflow readiness
      contract
- [x] define the required integrated acceptance base explicitly

### Batch 20.2 - Readiness Gate

- [x] wire the first machine-readable closeout descriptor and repo-owned gate
- [x] keep broader rerun and environment-specific depth advisory or deferred

### Batch 20.3 - Closeout Output

- [x] record the final `g08` closeout verdict and the next queue cleanly

## Acceptance Criteria

- [x] `g08` closes with one explicit downstream-workflow readiness gate
- [x] readiness claims stay tied to concrete runtime, supervisor, and host-edge
      evidence
- [x] deferred work is named cleanly instead of left ambiguous

## Risks And Mitigations

- Risk: closeout claims outrun the actual integrated acceptance evidence.
- Mitigation: bind all readiness claims to the completed `g08.019` seam before
  wiring the final gate.

## Evidence Requirements

- [x] log each meaningful tranche
- [x] run focused closeout validation after gate changes land
- [x] record the next queue or backlog posture explicitly

## Batch 20.1 Outcome

Batch 20.1 freezes the closeout policy in
`docs/contracts/071-generation-closeout-and-downstream-workflow-readiness-gate-contract.md`.
That contract locks the authority chain for the final `g08` closeout gate,
downstream workflow readiness meaning, and explicit deferred posture on top of
the now-closed `g08.019` integrated acceptance lane instead of leaving the
generation finish line as an open-ended review.

It also makes the closeout posture explicit:

- the integrated `g08` acceptance lane is the required fast-path base of the
  final gate
- one machine-readable closeout descriptor and Effigy gate task will be
  required once implemented
- broader repeated-run confidence and environment-specific workflow mixes stay
  advisory rather than quietly becoming blockers
- richer product-local controller, browser, immersive, certification, and
  downstream launch workflows remain deferred

That gives Batch 20.2 one fixed target for the actual closeout descriptor and
repo-owned gate task instead of reopening closeout policy while wiring the
final surface.

## Batch 20.2 Outcome

Batch 20.2 turns that frozen policy into a runnable shared surface. The
machine-readable `g08` closeout descriptor now lives in
`crates/signal-supervisor-tools/src/main.rs`, and Effigy now owns the matching
repo gate as `acceptance:g08-closeout` in `effigy.toml`.

This batch intentionally stops short of the final generation verdict. The
descriptor now reports:

- `g08`-specific contract and roadmap anchors instead of the older `g07`
  promotion record
- the required grouped integrated acceptance base from `g08.019`
- a runnable closeout-gate status and provisional downstream workflow
  readiness areas tied back to the closed Linux live, immersive, device
  workflow, and preview workflow seams
- explicit residual risk and next-queue posture that keeps the final
  closeout-or-backlog decision reserved for Batch 20.3

That gives Batch 20.3 one repo-owned closeout gate and one typed readiness
record to review, rather than forcing the final verdict to invent its own
surface or collapse back into prose-only judgment.

## Batch 20.3 Outcome

Batch 20.3 turns that provisional review state into the final `g08` verdict.
The machine-readable `g08` closeout descriptor now records one completed
closeout decision, points at an explicit post-`g08` backlog item, and makes it
clear that no new generation is active yet.

This final batch closes `g08` on one bounded repo-owned answer:

- the closed integrated acceptance seam from `g08.019` is sufficient for
  bounded downstream workflow readiness claims inside Signal
- broader repeated-run confidence and environment-specific matrices remain
  useful, but they are explicit post-`g08` backlog work instead of shadow
  blockers
- product-local controller, browser, immersive-console, certification, and
  downstream launch workflows remain deferred rather than being smuggled into
  the `g08` closeout bar
- the next likely queue is now recorded in
  `docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`
  instead of being left implicit

That closes `g08` cleanly: one final closeout verdict, one explicit deferred
queue, and no fake active generation left behind.

## Next Task

COMPLETE. `g08.020` closed on 2026-03-22 after the final `g08` closeout verdict
and explicit post-`g08` backlog handoff landed. Promote
`docs/roadmaps/backlog/post-g08-repeated-run-environment-matrices-and-downstream-workflow-depth.md`
only when maintainers choose to open the post-`g08` generation.
