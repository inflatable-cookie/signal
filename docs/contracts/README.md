# Contracts

Status: active
Updated: 2026-03-14

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
- `008-backend-neutral-plugin-capability-and-adapter-breadth-contract.md`
- `009-shared-host-convenience-api-and-consumer-edge-contract.md`
- `010-publication-grade-packaging-manifest-and-release-receipt-contract.md`
- `011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`
- `012-runtime-interruption-taxonomy-and-resumability-contract.md`
- `013-recording-continuity-midi-capture-and-checkpoint-contract.md`
- `014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md`

## Rule

Add a new contract only when the boundary needs stronger guarantees than
`architecture/` alone can provide.

## Next Task

Continue `g06.004` with Batch 4.1 by freezing offline-render recovery and
resumability semantics on top of the completed interruption and plugin
continuity contracts before deeper render session receipts widen.
