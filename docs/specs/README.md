# Specs

Status: active
Updated: 2026-04-11

Signal used a lane-first strict Northstar surface for the reopened `g09`
interactive-demo stream. There is currently no open strict lane.

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

## Active Entry Points

- no active strict lane currently
- `001-g09-lane-first-strict-adoption.md` remains as the completed reference
  for the closed `g09` lane

There is currently no ready batch card because `g09` is complete.

## Next Task

COMPLETED: `g09` is closed. Re-enter planning at the next-generation boundary
before promoting another strict execution lane.
