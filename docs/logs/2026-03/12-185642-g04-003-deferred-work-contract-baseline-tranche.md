# g04.003 Deferred Work Contract Baseline Tranche

Date: 2026-03-12
Scope: `docs/contracts/`, `docs/architecture/`, `docs/roadmaps/g04/`

## Summary

Completed Batch 3.1 of `g04.003` by freezing the first runtime-owned deferred
work contract before implementing a reusable orchestration baseline.

## What changed

- added `docs/contracts/005-runtime-work-orchestration-and-deferred-service-policy.md`
  to define the first deferred-work classes Signal owns directly:
  finalization/materialization, analysis/inspection, and recovery-adjacent
  service work
- froze the shared policy vocabulary of `Run`, `Defer`, `Throttle`, and
  `Abort`, and documented which existing runtime-owned service families fall
  into each class
- named the canonical inspection and export surfaces for this policy around
  offline render queue/purge/materialization receipts,
  `RuntimeTransportConcurrencySnapshot`, delegated offline boundaries, and
  profiling/soak receipts
- recorded the intentionally deferred scope so later `g04.003` work can add a
  concrete orchestration baseline without drifting into product UX or
  distributed job scheduling
- moved the roadmap queue from Batch 3.1 contract freezing to Batch 3.2
  implementation

## Why this tranche

`g04.003` could not safely implement deferred-work orchestration while the
meaning of offline finalization, cleanup retries, report materialization, and
delegated merge work was still partly implicit across runtime and host code.
This tranche makes the authority boundary explicit first.

## Validation

- `git diff --check`
- `effigy health`

## Next

Continue `g04.003` with Batch 3.2 and implement the first reusable
runtime-owned orchestration baseline, connecting defer/throttle/resume behavior
to runtime state for at least one real deferred service path.
