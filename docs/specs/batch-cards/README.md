# Batch Cards

Use this folder for ready execution cards under the active Signal strict lane.

## Rules

- create a card only when the work is specific enough to execute without fresh
  planning decisions
- keep cards attached to the active `g09` queue
- stop when the card boundary is exhausted, validation changes the plan, or the
  lane no longer matches live Signal state
- a bare `continue` should resolve through the previous closeout's `Next Task`
  rather than through chat recap
- if there is no current ready card, the lane is in planning, not execution

## Active Cards

- `001-install-g09-strict-lane-surfaces.md`
- no current ready card; the active strict lane is awaiting its next planning
  decision

## Next Task

Re-enter planning for the active strict `g09` lane before creating another
ready batch card.
