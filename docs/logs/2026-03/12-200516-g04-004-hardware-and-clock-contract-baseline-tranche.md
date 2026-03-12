# 2026-03-12 - g04.004 Batch 4.1 Hardware And Clock Contract Baseline

- Milestone: `g04.004`
- Batch: `4.1`
- Status: complete

## What changed

- added `docs/contracts/006-runtime-hardware-portability-and-clock-domain-contract.md`
  to freeze the first backend-neutral hardware capability, negotiated-stream,
  and clock-domain authority chain
- defined the Batch 4.1 semantic states for hardware paths:
  `same-clock`, `cross-clock`, `aggregate`, and `degraded`
- explicitly separated host-neutral export surfaces already backed by
  `signal-runtime` from backend-private detail that still needs typed receipts
  before consumers can depend on it
- updated the `g04.004` roadmap and reference trail to move the queue from
  Batch 4.1 contract freeze into Batch 4.2 runtime/hardware depth

## Validation

- `git diff --check`
- `effigy health --repo .`

## Residual risk

- live aggregate and multi-clock runtime behavior is now named but not yet
  implemented as typed runtime-owned receipts
- backend-specific drift/fallback detail still remains internal until Batch 4.2
  and Batch 4.3 promote the needed portability state into shared reports

## Next Task

Continue `g04.004` with Batch 4.2 and implement stronger clock-domain and
fallback handling in Signal-owned runtime and hardware crates, keeping
resampling and degradation semantics aligned with the new contract.
