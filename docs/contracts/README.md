# Contracts

Status: active
Updated: 2026-03-10

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

## Rule

Add a new contract only when the boundary needs stronger guarantees than
`architecture/` alone can provide.

## Next Task

Keep the current contracts stable while Signal is reoriented as a polished
generic library system, then add new contracts only for boundaries that
actually need validation-grade precision.
