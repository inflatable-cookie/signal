# 2026-03-19 - g08.006 immersive room-policy contract opening tranche

## Summary

Closed Batch 6.1 of `g08.006` by freezing the first runtime-owned immersive
object-rendering and room-policy contract.

## Changes

- added
  `docs/contracts/057-immersive-object-rendering-and-room-policy-substrate-contract.md`
  to define immersive object-rendering posture, room-policy class,
  room-policy authority, and immersive room outcome
- updated `docs/roadmaps/g08/006-immersive-object-rendering-and-room-policy-substrate.md`
  with Batch 6.1 outcome and the Batch 6.2 handoff
- updated shared roadmap, contract, and architecture indexes so the active next
  step points at runtime-owned immersive receipt materialization instead of the
  contract-opening batch

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

This tranche freezes meaning, not runtime realization. Immersive object and
room-policy truth still needs runtime-owned receipts before supervisor,
diagnostic, and stable host-edge consumers can inspect it without
renderer-private reconstruction.

## Next Task

Continue `g08.006` with Batch 6.2 by materializing the first runtime-owned
immersive object-rendering and room-policy receipts, then align stable
host-edge export to the same bounded model.
