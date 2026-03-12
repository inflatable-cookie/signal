# 2026-03-12 - g04.005 Batch 5.1 Plugin Backend Contract Baseline

- Milestone: `g04.005`
- Batch: `5.1`
- Status: complete

## What changed

- added `docs/contracts/007-plugin-backend-and-host-neutral-delegation-contract.md`
  to freeze the first Signal-owned plugin backend and delegation authority split
- defined which current surfaces are format-neutral and reusable in
  `signal-plugin`
- defined which runtime-owned plugin lifecycle, chain, recall, and delegated
  offline execution receipts are the canonical execution/export boundary in
  `signal-runtime`
- explicitly classified current `signal-plugin-clap` discovery, extension, and
  protocol helpers as adapter-specific detail rather than reusable consumer
  boundary

## Validation

- `git diff --check`
- `effigy health --repo .`

## Residual risk

- the contract is frozen, but richer format-neutral scan/discovery receipts do
  not exist yet
- wider adapter coverage beyond the current CLAP-first path remains deferred
  until Batch 5.2 and Batch 5.3

## Next Task

Continue `g04.005` with Batch 5.2 and deepen the typed plugin backend and
host-neutral delegation surfaces in Signal-owned crates, starting with richer
reusable scan or discovery receipts and tighter runtime-owned delegation
inputs.
