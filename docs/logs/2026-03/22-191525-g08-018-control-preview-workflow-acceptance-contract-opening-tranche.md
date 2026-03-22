# 2026-03-22 - g08.018 Batch 18.1 control and preview workflow acceptance contract

## Summary

- opened `g08.018` with a new shared acceptance contract for control-surface
  workflow and preview workflow depth
- froze the authority line, scenario families, and required versus advisory
  versus deferred policy on top of the already-closed advanced-hardware and
  preview-transform consumer seams
- moved the roadmap and shared reference trail forward so the next queue is the
  grouped descriptor and acceptance lane rather than more contract churn

## Evidence

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.018` with Batch 18.2 by wiring the first repo-owned descriptor
and acceptance lane for the shared control-surface and preview workflow seam
while keeping device-native and browser-native workflow depth explicit and
non-blocking.
