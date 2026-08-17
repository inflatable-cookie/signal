# g11.001 Batch 1.1 Host-Assembly Integration Map

Status: batch closeout
Date: 2026-08-17
Owner: core-product
Milestone: `docs/roadmaps/g11/001-production-host-assembly-wiring.md`

## Summary

Closed Batch 1.1 docs-only: froze the production host-assembly integration map
and opened `g11` as the active generation.

## Deliverables

- `docs/architecture/production-host-assembly-integration.md`
- `docs/roadmaps/g11/README.md`
- `docs/roadmaps/g11/001-production-host-assembly-wiring.md`
- `docs/roadmaps/g11/002-shared-sandbox-tier.md`
- front-door updates across `docs/README.md`, `docs/roadmaps/README.md`, and
  `docs/roadmaps/generation-index.md`

## Decisions

- plugin hosting baseline is shipped; `g11.001` owns integration only
- v1 supports `InProcess` and `DedicatedSandbox`; `SharedSandbox` stays in
  `g11.002` under Contract `014`
- SharedSandbox needs roadmap/implementation, not a separate research program

## Validation

- `effigy qa:docs`

## Next Task

Execute `docs/roadmaps/g11/batch-cards/001-g11-001-bridge-backend-factory.md`.
