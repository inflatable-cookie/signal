# 2026-03-19 18:46:59 - g08.005 pin-matrix and bus-negotiation contract opening

## Summary

Opened `g08.005` by freezing the first runtime-owned complex plugin pin-matrix
and dynamic bus-negotiation contract.

## What changed

- added `docs/contracts/056-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-contract.md`
- updated the active roadmap in
  `docs/roadmaps/g08/005-complex-plugin-pin-matrix-and-dynamic-bus-negotiation-depth.md`
  to mark Batch 5.1 complete and point Batch 5.2 at runtime-owned receipt depth
- updated the contracts index, roadmap indexes, and architecture reference so
  `g08.005` now has one explicit pin-matrix and dynamic bus-negotiation
  authority line
- kept the existing `complex-io` consumer and acceptance seam as a prior
  reusable anchor rather than reopening or duplicating it in this contract-only
  tranche

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual risk

This tranche freezes meaning, not runtime realization. Pin-group identity,
pin-matrix posture, and dynamic bus-negotiation truth still need runtime-owned
receipts before downstream consumers can inspect them without adapter-private
reconstruction or host-local bus policy.

## Next Task

Continue `g08.005` with Batch 5.2 by materializing the first runtime-owned
complex plugin pin-matrix and dynamic bus-negotiation receipts, then align
stable host-edge export to the same bounded model.
