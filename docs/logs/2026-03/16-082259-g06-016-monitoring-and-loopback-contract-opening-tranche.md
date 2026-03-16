# 2026-03-16 08:22:59 UTC - g06.016 Monitoring And Loopback Contract Opening Tranche

## Summary

Opened `g06.016` by freezing the first runtime-owned external-I/O, monitoring
tap-point, and loopback measurement contract. This batch fixes the shared
vocabulary and authority chain before any DTO widening, so later monitoring and
calibration depth can build on one reusable boundary instead of host-local
routing or measurement heuristics.

## Work completed

- added the new contract:
  - `docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md`
- froze the shared vocabulary for:
  - external-I/O roles
  - monitor paths
  - tap points
  - loopback paths
  - measurement sessions
  - reference-path meaning
- fixed the authority line between `signal-hardware`, `signal-runtime`, and
  shared host surfaces for monitoring and loopback semantics
- made the runtime-facing versus supervisor-facing split explicit before Batch
  16.2 receipt work begins
- rolled the roadmap and index surfaces forward so Batch 16.2 is the active
  queue

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred scope

- runtime DTOs and proof surfaces for monitoring and loopback still belong to
  Batch 16.2 and Batch 16.3
- full cue-mix UX, room correction, acoustic calibration policy, and
  product-local monitor routing remain out of scope
- remote or network-audio monitoring semantics remain deferred

## Next Task

Continue `g06.016` with Batch 16.2 by materializing runtime-owned monitoring,
tap-point, and loopback receipts on top of the closed hardware supervision and
clock-topology boundaries while keeping local and server host consumers aligned
to the same model.
