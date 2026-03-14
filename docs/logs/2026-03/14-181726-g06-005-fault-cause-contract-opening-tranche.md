# 2026-03-14 18:17:26 - g06.005 fault-cause contract opening tranche

## What changed

- added the first shared `g06.005` causal diagnostics contract in
  `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
  so runtime fault attribution now has an explicit authority chain instead of
  relying on mixed counters and product-local diagnosis
- froze the first five shared causal families:
  `xrun pressure`, `callback pressure`, `plugin boundary fault`,
  `device path fault`, and `deferred-work pressure`
- pinned how those causal families compose with the already-closed
  interruption and resumability taxonomy rather than creating a second
  recovery language
- made the runtime-versus-host split explicit: runtime-owned posture and
  primary-cause truth remains canonical while host callback and backend counts
  are advisory evidence only
- marked Batch 5.1 complete in `g06.005` and moved the active queue to the
  Batch 5.2 DTO and export pass

## Evidence

- `docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md`
- `docs/roadmaps/g06/005-runtime-fault-cause-attribution-and-diagnostic-receipts.md`
- `docs/contracts/README.md`
- `docs/roadmaps/g06/README.md`
- `docs/roadmaps/README.md`
- `docs/roadmaps/generation-index.md`
- `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Deferred

- the contract freezes meaning, not runtime DTO shape yet; Batch 5.2 still
  needs to materialize typed causal receipts in runtime and supervisor export
- callback pressure is intentionally frozen as advisory host-adjacent evidence
  until Batch 5.2 proves how much of it should be promoted into canonical
  runtime-owned receipt fields
- per-event traces, fleet telemetry, and product-specific diagnostic UX remain
  out of scope for `g06.005`

## Next Task

Continue `g06.005` with Batch 5.2 by materializing typed fault-cause and
contributing-evidence receipts in runtime, supervisor, and stable host-edge
surfaces without reintroducing host-local causal reclassification.
