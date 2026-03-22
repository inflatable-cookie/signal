# 2026-03-22 - g08.016 Batch 16.1 Linux Live Acceptance Contract Opening

## Summary

- opened `g08.016` Batch 16.1 by freezing the shared live Linux backend
  acceptance and failure-injection contract in
  `docs/contracts/067-live-linux-backend-acceptance-and-failure-injection-contract.md`
- defined the authority chain, grouped scenario families, and
  required/advisory/deferred policy for one repo-owned Linux live-backend
  acceptance lane on top of the closed live ownership, JACK coordination,
  PipeWire/ALSA parity, and clock-topology seams
- rolled the roadmap, contract index, generation pointers, and architecture
  reference forward so the next actionable queue is `g08.016` Batch 16.2

## Validation

- `git diff --check`
- `effigy health`
- `effigy qa:docs`

## Next Task

Continue `g08.016` with Batch 16.2 by wiring the first repo-owned descriptor
and acceptance lane for the shared live Linux backend seam while keeping
backend-native recovery depth explicit and non-blocking.
