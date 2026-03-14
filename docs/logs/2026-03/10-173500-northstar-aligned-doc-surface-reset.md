# 2026-03-10 17:35:00 - Northstar-aligned doc surface reset

## Summary

Normalized the active Signal docs entry surfaces so the repo reads like a
Northstar-based generic library system rather than a project-specific planning
index.

## Changes

1. Rewrote `docs/README.md` around Northstar core sections and optional add-ons.
2. Rewrote the active section indexes for:
   - `docs/architecture/README.md`
   - `docs/contracts/README.md`
   - `docs/roadmaps/README.md`
   - `docs/logs/README.md`
3. Shifted section language away from Loophole-specific implementation framing
   and toward Signal’s reusable library/runtime posture.

## Validation Performed

- `git diff --check`
- `effigy signal/health`
- `effigy signal/validate`

## Evidence

- Northstar section contracts reviewed from `../northstar/bundle-docs/sections/`

## Risks

- This batch changes the documentation control surface only; it does not yet
  move the legacy C++ implementation behind a cleaner reference boundary.

## Next Task

Push the Signal reset deeper by moving legacy C++ implementation material out of
the active library surface and tightening the remaining docs around the generic
library-system posture.
