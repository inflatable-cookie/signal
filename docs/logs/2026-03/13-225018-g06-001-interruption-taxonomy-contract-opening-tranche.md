# 2026-03-13 22:50:18 GMT - g06.001 interruption taxonomy contract opening tranche

## Summary

Opened `g06.001` by freezing the first runtime interruption and resumability
contract so later `g06` recovery, profiling, plugin, hardware, and soak work
can all build on one shared vocabulary.

## Work completed

- added contract `012` for runtime interruption taxonomy and resumability
- defined shared terms for `interruption`, `resumable`, `restartable`,
  `recoverable`, `terminal`, and `rebindable`
- mapped that vocabulary onto current runtime-owned fault, degradation,
  recovery-history, and deferred/offline continuity seams
- activated `g06.001` and moved the thread to Batch 1.2 instead of jumping
  ahead to recording continuity work

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This tranche freezes meaning, not implementation alignment. Current runtime and
host-facing receipts still need Batch 1.2 work so the new vocabulary becomes
first-class typed surface area rather than contract prose only.

## Next Task

Continue `g06.001` with Batch 1.2 by applying the interruption and
resumability contract to active runtime-owned snapshots, receipts, and shared
host-edge surfaces.
