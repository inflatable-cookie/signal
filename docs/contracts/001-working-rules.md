# 001 - Working Rules

Status: active
Owner: core-product
Updated: 2026-04-09
Depends on: docs/architecture/system-architecture.md
Authority owners: core-product
Affects: docs, crates

## Problem

Signal now needs a lane-first strict Northstar surface for the active `g09`
queue so longer-running runtime and host work can stay inside explicit
guardrails instead of depending only on roadmap prose and ad hoc thread memory.

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

- The first strict lane is attached to the live `g09` queue.
- `g09.011` is the current strict-execution milestone.
- The immediate follow-on boundary is `g09.012`.
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
- the work no longer matches the active `g09` milestone or its governing
  contracts
- validation fails in a way that changes the plan
- the current strict card is exhausted and no next ready card exists

## Validation

- `effigy health`
- `effigy qa:docs`

## Next Task

Use this contract with the active `g09.011` strict lane while
`docs/specs/batch-cards/020-g09-011-demo-launch-and-evidence-conventions.md`
governs the active batch.
