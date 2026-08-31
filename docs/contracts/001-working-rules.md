# 001 - Working Rules

Status: active
Owner: core-product
Updated: 2026-08-17
Depends on: docs/architecture/system-architecture.md
Authority owners: core-product
Affects: docs, crates

## Problem

Signal needed a lane-first strict Northstar surface so longer-running runtime
and host work could stay inside explicit guardrails instead of depending only
on roadmap prose and ad hoc thread memory.

## Contract

### Delivery grammar

- Material active-lane work should follow:
  `vision -> architecture + contracts -> strict-lane spec -> roadmap milestone -> batch card -> evidence -> closeout`.
- Signal is currently in lane-first stricter adoption, not full strict
  compliance across every active lane.
- The fuller continuation and ready-state model applies where `docs/specs/`
  and batch cards are explicitly installed.
- Durable structure belongs in architecture and durable behavior or policy
  belongs in contracts; specs are provisional and should not become shadow
  authority.

### Lane-first strict posture

- The first strict lane was attached to the reopened `g09` queue and closed
  with `g09.015`.
- `g10` ran mostly baseline-routed; the stretch audit completed without an
  active strict-execution milestone.
- There is currently no active strict-execution milestone. `g11.003` is a
  baseline-routed explicit audit maintenance lane, not a reopened strict spec.
- Closed generations and unrelated active work should not be backfilled just to
  make the stricter surface look symmetrical.

### Ready-state rubric

- A batch card is `ready` only when:
  - the objective is bounded enough to execute without fresh planning decisions
  - the governing refs point at current roadmap and contract surfaces
  - scope boundaries, acceptance criteria, validation, evidence requirements,
    and stop conditions are explicit
  - no unresolved planning gap still governs the card's scope
- A short continuation chain is valid only when each transition is already
  explicit in file state and still inside the active strict lane.
- In the strict lane, a bare `continue` should resolve through the previous
  closeout's `Next Task`, which should normally point at the current ready card
  or an explicit stop/reassessment step.

### Definition of done

- Work is not done unless the claimed runtime or host behavior exists for real,
  not as scaffold, placeholder, or synthetic stand-in.
- Relevant roadmap, spec, card, and log surfaces must all reflect the current
  truth.
- Validation actually run must be recorded.
- Remaining limits or deferred seams must be stated explicitly rather than
  implied away.
- The next task must be explicit enough that a later bare `continue` does not
  need a recap prompt to find the correct next move.

### Closeout pattern

- For a meaningful batch closeout:
  - update the current batch card first
  - update the active roadmap milestone if progress or the next batch changed
  - refresh any front-door or currentness surfaces that name the active lane,
    current ready card, or recent evidence chain
  - write the batch log with evidence and validation actually run
  - update handoff state only if another thread truly needs to continue
  - leave one explicit next task in the highest-authority active surface

### Strict-lane autonomy

- The paused thread may continue inside the active strict lane only while the
  current card remains `ready` and the governing refs still match live Signal
  state.
- If the lane is healthy, a later bare `continue` should normally be enough
  because the previous closeout already named the next task and the current
  ready card.
- If active code or docs drift beyond the card boundary, stop and re-enter
  planning before resuming.

### Stop conditions

- a batch needs fresh design or planning judgment not already captured in the
  strict lane
- the work no longer matches the active generation milestone or its governing
  contracts
- validation fails in a way that changes the plan
- the current strict card is exhausted and no next ready card exists

## Generation Rollover Rule

Treat roadmap generations as substantial sequencing eras, not tiny buckets. In a long-running repo, expect roughly 20 to 40 roadmap files in one generation before rollover is even worth discussing.

Treat rollover as full closeout:

- every roadmap in the old generation must be explicitly closed, paused, superseded, or moved to backlog
- the roadmap front doors must reflect that closed state before the next generation opens
- stale specs and batch cards from the closing generation must be archived or removed from `docs/specs/`

If those closeout conditions are not satisfied, repair the current generation instead of opening a new one.

## Validation

- `effigy health`
- `effigy qa:docs`

## Next Task

Signal is baseline-routed with `g11.001` and `g11.002` complete. Execute the
bounded `g11.003` instruction and Rust quality audit through card `008`, then
stop at its PR for orchestrator review. Do not start a follow-on generation or
infer a product backlog pull. Reopen this contract when a future generation
installs a new strict lane.
