# 2026-03-16 13:57:20 UTC - g06.018 Analysis-Metadata Contract Opening Tranche

## Summary

Opened `g06.018` by freezing the first reusable analysis-metadata and
library-service contract on top of the closed `g06.017` media-service
boundary. This tranche fixes ownership for reusable asset descriptors,
bounded analysis-family coverage, and metadata readiness versus staleness
before any runtime DTO widening begins.

## Work completed

- added the new contract:
  - `docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md`
- recorded the Batch 18.1 outcome in:
  - `docs/roadmaps/g06/018-analysis-metadata-extraction-and-library-service-depth.md`
- rolled the contract, roadmap, and architecture references forward so the
  active queue now points to Batch 18.2

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- this tranche freezes descriptor meaning, not runtime DTO or export depth
- recommendation, search-ranking, product-local browser, tagging, and broader
  intelligence workflows remain outside the shared contract

## Next Task

Continue `g06.018` with Batch 18.2 by materializing the first runtime-owned
analysis-metadata and library-service descriptor family through runtime,
supervisor, and stable host-edge surfaces without reopening product-local
metadata ownership.
