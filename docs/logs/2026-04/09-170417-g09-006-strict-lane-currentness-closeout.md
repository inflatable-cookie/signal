# 2026-04-09 - g09.006 strict-lane currentness closeout

## Summary

Closed out the completed strict `g09.006` sandbox-session batch in the active
Northstar front-door surfaces, then reassessed the remaining live duplication
to decide whether the lane still has one more bounded ready tranche.

## Reassessment

The completed `002-g09-006-sandbox-session-consolidation` batch removed the
old broad broker-session shell from the host copies, so it should no longer be
presented as the current ready card.

The remaining meaningful `g09.006` seam is now narrower but still real. Inside
`sandbox_sessions.rs`, the next bounded shared-support target is:

- the duplicated AU broker-preparation shell
- the duplicated VST3 broker-preparation shell
- the identical AU fault-recording shell

That is still batch-cardable without widening the lane into a full runtime or
lifecycle rewrite, while LV2 remains explicitly server-specific and should stay
out of the shared extraction.

## Changes

- updated
  `~/Dev/projects/signal/docs/specs/batch-cards/002-g09-006-sandbox-session-consolidation.md`
  so its continuation points at the next card instead of presenting itself as
  the live ready boundary
- added the next strict ready card at
  `~/Dev/projects/signal/docs/specs/batch-cards/003-g09-006-au-vst3-preparation-fault-shell.md`
- updated
  `~/Dev/projects/signal/docs/roadmaps/g09/006-shared-host-runtime-execution-and-recovery-unification.md`
  with the strict-lane reassessment outcome and the new ready-card reference
- updated
  `~/Dev/projects/signal/docs/specs/README.md`
  and `~/Dev/projects/signal/docs/logs/README.md`
  so the active strict front doors point at the current ready card
- updated
  `~/Dev/projects/signal/docs/contracts/contract-index.md`
  so the contract front door reflects the true active strict card

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

The strict-lane docs surface is coherent again:

- the completed `002...` card remains as closed execution history
- the active currentness/front-door surfaces now point at the new ready card
- the next `g09.006` tranche is bounded to the remaining AU/VST3
  preparation-and-fault shell only

## Next Task

Continue the active strict `g09.006` lane from
`docs/specs/batch-cards/003-g09-006-au-vst3-preparation-fault-shell.md`.
