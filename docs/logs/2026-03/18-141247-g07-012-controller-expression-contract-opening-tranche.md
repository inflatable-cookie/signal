# 2026-03-18 - g07.012 controller-expression contract opening tranche

## Summary

Opened Batch 12.1 of `g07.012` by freezing the widened MIDI 2.0, MPE, and
richer controller-expression boundary on top of the closed generic event and
external MIDI endpoint contracts.

This tranche gives later runtime, plugin, and hardware work one shared Signal-
owned target for widened expressive-event meaning instead of letting adapter-
private packet models or host-local controller taxonomies become the consumer
boundary.

## Key changes

- added the new contract
  `docs/contracts/043-midi-2-0-mpe-and-richer-controller-expression-contract.md`
- froze the authority chain between:
  - the existing generic event contract
  - the external MIDI endpoint and device-identity contract
  - raw adapter or backend packet evidence
  - runtime-owned widened controller-expression meaning
- defined the first widened shared vocabulary for:
  - richer controller-expression families
  - MPE posture
  - MIDI 2.0 posture
  - expression capability
  - guarded widening
- rolled roadmap and shared index pointers forward so Batch 12.2 is now the
  explicit next queue
- corrected stale closed-surface next-task pointers left in the `g07.010`
  Linux backend docs

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy validate --repo .`
- `effigy qa:docs --repo .`

## Residual risk

This is contract-only. There is still no widened runtime DTO family, no
machine-readable boundary descriptor, and no proof seam for MIDI 2.0, MPE, or
richer controller-expression behavior yet. Those now belong to Batch 12.2 and
Batch 12.3.

## Next Task

Continue `g07.012` with Batch 12.2 by materializing the first runtime-owned
MIDI 2.0, MPE, and richer controller-expression receipt family across runtime,
plugin, and hardware surfaces without reopening adapter-private packet
ownership.
