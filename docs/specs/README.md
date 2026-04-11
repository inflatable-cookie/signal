# Specs

Status: active
Updated: 2026-04-10

Signal is using a lane-first strict Northstar surface for the reopened `g09`
interactive-demo stream.

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

- `001-g09-lane-first-strict-adoption.md`
- `batch-cards/001-install-g09-strict-lane-surfaces.md`

There is no current ready batch card.

## Next Task

Re-enter planning for the active strict `g09` lane and decide whether the next
honest `g09.015` seam is another crate-family operator-view uplift, deeper
live plugin interaction, or a planning pause.
