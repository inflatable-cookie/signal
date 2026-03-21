# 2026-03-19 17:29:51 - g08.004 LV2 worker/URID/patch extension contract opening

## Summary

Opened `g08.004` by freezing the first runtime-owned LV2 worker, URID, patch,
and extension-negotiation contract.

## What changed

- added `docs/contracts/055-lv2-worker-urid-patch-and-extension-negotiation-contract.md`
- added the active roadmap in
  `docs/roadmaps/g08/004-lv2-worker-urid-patch-and-extension-negotiation-baseline.md`
- updated the `g08` milestone map, roadmap indexes, and architecture reference
  to make `g08.004` the active next step
- recorded Batch 4.1 as a bounded contract-opening tranche so Batch 4.2 can
  stay focused on runtime-owned receipt depth

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual risk

This tranche freezes meaning, not runtime realization. LV2 worker posture,
URID negotiation, patch exchange, and extension-negotiation truth still need
runtime-owned receipts before downstream consumers can inspect them without
adapter-private reconstruction.

## Next Task

Continue `g08.004` with Batch 4.2 by materializing the first runtime-owned LV2
worker, URID, patch, and extension-negotiation receipts, then align stable
host-edge export to the same bounded parity model.
