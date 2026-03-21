# 2026-03-19 - g08.007 speaker deployment and monitoring contract opening tranche

## Summary

Closed Batch 7.1 of `g08.007` by freezing the first runtime-owned speaker
deployment, fold-down, and monitoring-scene contract.

## Changes

- added
  `docs/contracts/058-speaker-deployment-fold-down-and-monitoring-scene-contract.md`
  to define deployment class, fold-down policy, monitoring-scene class,
  monitoring-scene authority, and monitoring outcome
- updated `docs/roadmaps/g08/007-speaker-deployment-fold-down-and-monitoring-scene-depth.md`
  with Batch 7.1 outcome and the Batch 7.2 handoff
- updated shared roadmap, contract, and architecture indexes so the active next
  step points at runtime-owned deployment and monitoring receipt materialization
  instead of the contract-opening batch

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Residual Risk

This tranche freezes meaning, not runtime realization. Deployment, fold-down,
and monitoring-scene truth still needs runtime-owned receipts before
supervisor, diagnostic, and stable host-edge consumers can inspect it without
renderer-private reconstruction.

## Next Task

Continue `g08.007` with Batch 7.2 by materializing the first runtime-owned
speaker deployment, fold-down, and monitoring-scene receipts, then align stable
host-edge export to the same bounded model.
