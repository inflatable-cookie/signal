# g11.002 Batch 2.0 Multiplexing Design Note

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/002-shared-sandbox-tier.md`

## Summary

Closed Batch 2.0 docs-only after operator product pull. SharedSandbox v1
groups by plugin type identity. No new contract. No research lane.

## Deliverables

- `docs/architecture/shared-sandbox-multiplexing.md`
- `docs/roadmaps/g11/002-shared-sandbox-tier.md` now active
- batch cards `004` (complete), `005` (ready), `006`–`007` (auto-start)

## Decisions

- grouping key: `plugin:{plugin_type_id}`
- reuse `ShmPluginProcessor` per member lease; no new audio-thread backend
- existing broker commands stay; omitted `instance_id` means `sandbox_id`
- `start-processing` is boundary-level; no members after start in v1
- DedicatedSandbox remains default and single-slot
- Contract `014` needs no vocabulary change; runtime SharedSandbox default
  group key must stop falling back to `sandbox:{sandbox_id}`

## Validation

- `effigy qa:docs`

## Next Task

Execute `docs/roadmaps/g11/batch-cards/005-g11-002-broker-multiplexing.md`.
