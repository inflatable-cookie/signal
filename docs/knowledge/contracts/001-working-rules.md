# 001 - Working Rules

Status: active
Owner: core-product
Updated: 2026-09-09
Depends on: docs/knowledge/architecture/system-architecture.md
Authority owners: core-product
Affects: docs, crates

## Problem

Signal needed a lane-first strict Northstar surface so longer-running runtime
and host work could stay inside explicit guardrails instead of depending only
on roadmap prose and ad hoc thread memory.

## Contract

### Delivery grammar

- Material active-lane work should follow:
  `vision -> architecture + contracts -> roadmap task -> evidence -> closeout`,
  with a strict-lane spec inserted after contracts only where `docs/specs/`
  explicitly installs one.
- Signal is currently baseline-routed, not in lane-first strict adoption:
  there is no open strict lane and no ready task.
- The fuller continuation and ready-state model below applies only where a
  strict lane is explicitly installed.
- Durable structure belongs in architecture and durable behavior or policy
  belongs in contracts; specs are provisional and should not become shadow
  authority.

- The first strict lane was attached to the reopened `g09` queue and closed
  with `g09.015`.
- `g10` ran mostly baseline-routed; the stretch audit completed without an
  active strict-execution task.
- `g11.003` was a baseline-routed explicit audit maintenance task, not a
  reopened strict spec; it is complete and merged through PR `#18`.
- There is currently no active strict lane and no ready task.
- Closed generations and unrelated active work should not be backfilled just to
  make the stricter surface look symmetrical.

### Ready-state rubric

- A roadmap task is `ready` only when:
  - the objective is bounded enough to execute without fresh planning decisions
  - the governing refs point at current architecture and contract surfaces
  - scope boundaries, acceptance criteria, validation, evidence requirements,
    and stop conditions are explicit
  - no unresolved planning gap still governs the task's scope
- A short continuation chain is valid only when each transition is already
  explicit in file state and still inside the active lane.
- In a strict lane, a bare `continue` should resolve through the previous
  closeout's `Next Task`, which should normally point at the current ready task
  or an explicit stop/reassessment step.

### Definition of done

- Work is not done unless the claimed runtime or host behavior exists for real,
  not as scaffold, placeholder, or synthetic stand-in.
- Relevant roadmap task and log surfaces must all reflect the current
  truth (plus the spec surface while a strict lane is open).
- Validation actually run must be recorded.
- Remaining limits or deferred seams must be stated explicitly rather than
  implied away.
- The next task must be explicit enough that a later bare `continue` does not
  need a recap prompt to find the correct next move.

### Closeout pattern

- For a meaningful task closeout:
  - update the current task file first
  - refresh any front-door or currentness surfaces that name the active lane,
    current ready task, or recent evidence chain
  - write the closeout log with evidence and validation actually run
  - update handoff state only if another thread truly needs to continue
  - leave one explicit next task in the highest-authority active surface

### Strict-lane autonomy

- The paused thread may continue inside the active strict lane only while the
  current task remains `ready` and the governing refs still match live Signal
  state.
- If the lane is healthy, a later bare `continue` should normally be enough
  because the previous closeout already named the next task and the current
  ready task.
- If active code or docs drift beyond the task boundary, stop and re-enter
  planning before resuming.

### Stop conditions

- a task needs fresh design or planning judgment not already captured in the
  strict lane
- the work no longer matches the active generation task or its governing
  contracts
- validation fails in a way that changes the plan
- the current strict task is exhausted and no next ready task exists

## Generation Rollover Rule

Treat roadmap generations as substantial sequencing eras, not tiny buckets. In a long-running repo, expect roughly 20 to 40 roadmap files in one generation before rollover is even worth discussing.

Treat rollover as full closeout:

- every task in the old generation must be explicitly closed, paused, superseded, or moved to triage
- the roadmap front doors must reflect that closed state before the next generation opens
- stale specs from the closing generation must be archived or removed from `docs/specs/`

If those closeout conditions are not satisfied, repair the current generation instead of opening a new one.

## Validation

- `effigy health`
- `effigy qa:docs`

## Next Task

Signal is baseline-routed with `g11.001` through `g11.003` complete. Return to
operator planning or triage review; there is no ready task. Do not start a
follow-on generation or infer a product pull from triage without operator selection. Reopen this contract when
a future generation installs a new strict lane.
