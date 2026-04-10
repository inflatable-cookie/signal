# Specs

Status: active
Updated: 2026-04-10

Signal used a lane-first strict Northstar surface for `g09`. That generation
is now closed.

## Purpose

Use this folder for the active strict lane only:

- one live spec that binds the stricter execution model to the active
  generation while it is open
- bounded batch cards that let the active thread continue without fresh
  planning decisions

Signal is not yet using specs as a repo-wide default. This is a lane-first
strict surface that can be attached to one active generation at a time.

## Rules

- specs are provisional planning surfaces, not permanent authority
- architecture remains the durable structure surface
- contracts remain the durable behavior and policy surface
- the active strict lane should stay small and current rather than turning into
  a second roadmap archive
- archive or remove closed strict-lane planning once it no longer governs live
  work
- in the strict lane, a bare `continue` should resolve through the previous
  closeout's `Next Task`
- that `Next Task` should normally point at the current ready card or an
  explicit stop/reassessment step
- if no current ready card exists, do not infer the next code task from memory;
  re-enter planning first

## Entry Points

- `001-g09-lane-first-strict-adoption.md`
- `batch-cards/001-install-g09-strict-lane-surfaces.md`

There is currently no ready batch card. `g09` is closed and awaiting
next-generation planning.

## Next Task

Re-enter planning at the next-generation boundary before promoting another
strict execution lane.
