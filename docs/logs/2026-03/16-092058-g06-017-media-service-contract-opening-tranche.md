# 2026-03-16 - g06.017 Batch 17.1 media-service contract opening

## Summary

Opened `g06.017` with the first reusable Signal-owned media-service contract so
later waveform, preview, and metadata work deepens one shared boundary instead
of rebuilding product-local media cache or preview semantics.

## Work completed

- froze the runtime-owned media indexing, waveform-analysis, preview, and
  invalidation boundary in
  `docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md`
- fixed the authority split between raw media inputs, shared
  `signal-analysis*` crates, and `signal-runtime` service-state meaning for
  asset identity, readiness, invalidation, and preview state
- rolled the roadmap and reference trail forward so `g06.017` now points to
  Batch 17.2 service realization instead of keeping the contract-opening step
  active

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

Batch 17.1 intentionally does not promise:

- product-local media browser, playlist, tagging, or collection UX
- remote catalog sync or editorial media management
- ML-driven ranking or semantic media workflows
- final waveform visualization format or publication-grade library exchange

## Next Task

Continue `g06.018` with Batch 18.1 by freezing the first reusable
analysis-metadata and library-service descriptor family on top of the closed
media-service boundary.
