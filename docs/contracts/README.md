# Contracts

Status: active
Updated: 2026-03-12

## Why this section matters now

Contracts freeze the reusable boundaries that Signal consumers should be able to
rely on.

## Scope

Use this section for:

- stable reusable-DSP and runtime boundary contracts
- export/report contracts
- host-edge and policy contracts when prose architecture is not precise enough

## Current Baseline

- `001-shared-dsp-and-host-boundary.md`
- `002-supervisor-export-schema-and-report-boundary.md`
- `003-crate-maturity-and-public-runtime-boundary-baseline.md`
- `004-runtime-multicore-scheduling-and-anticipative-execution-contract.md`
- `005-runtime-work-orchestration-and-deferred-service-policy.md`
- `006-runtime-hardware-portability-and-clock-domain-contract.md`
- `007-plugin-backend-and-host-neutral-delegation-contract.md`

## Rule

Add a new contract only when the boundary needs stronger guarantees than
`architecture/` alone can provide.

## Next Task

COMPLETE. The current contract set is stable through the closed `g04`
boundary. Promote
`docs/roadmaps/backlog/post-g04-consumer-release-and-backend-breadth.md` only
when maintainers choose to open the next generation.
