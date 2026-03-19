# 2026-03-19 - g08.001 Linux Live Ownership Contract Opening Tranche

## Summary

Opened the first `g08` contract by freezing runtime-owned live Linux audio
backend ownership and session-lifecycle meaning across ALSA, JACK, and
PipeWire.

## Why this tranche matters

`g07` closed bounded Linux backend identity and parity, but later `g08` Linux,
device, and workflow work would still fall back into backend-private or
host-local session stories without one explicit live-ownership authority line.
This tranche fixes that before runtime realization widens.

## Work completed

- added
  `docs/contracts/052-live-linux-audio-backend-ownership-and-session-lifecycle-contract.md`
- recorded the Batch 1.1 outcome in
  `docs/roadmaps/g08/001-live-linux-audio-backend-ownership-and-session-lifecycle-substrate.md`
- rolled the shared contract, roadmap, and architecture references forward so
  Batch 1.2 is now the explicit next queue

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Deferred

- runtime-owned live ALSA, JACK, and PipeWire ownership receipts
- stable host-edge export for the widened session lifecycle seam
- public consumer proof for live Linux backend ownership

## Next task

Continue `g08.001` with Batch 1.2 by materializing the first runtime-owned
live Linux backend ownership, session-lifecycle, and device-claim receipt
family across runtime, supervision, and stable host-edge surfaces.
