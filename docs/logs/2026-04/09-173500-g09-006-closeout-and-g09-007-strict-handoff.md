# 2026-04-09 - g09.006 closeout and g09.007 strict handoff

## Summary

Re-entered planning after the last `g09.006` strict batch closed and confirmed
that `g09.006` no longer had another honest broad shared-support seam. Then
shifted the active strict lane into `g09.007` with one new ready batch card
based on the real remaining runtime decomposition seam.

## Reassessment

The previous `g09.007` roadmap draft was partially stale: it still talked like
`crates/signal-runtime/src/interfaces.rs` was the main oversized runtime root,
but the live file is already a thin front door. The real remaining milestone
seams are:

- the heavy internal assembly wall in
  `crates/signal-runtime/src/interfaces_offline_contract_family/request_preview/request_assembly.rs`
- the still-broad runtime test front door in
  `crates/signal-runtime/src/tests.rs`

That means the correct handoff is not another `g09.006` card. It is a
milestone rollover into `g09.007`, starting with the offline preview assembly
carveout and leaving test-surface normalization as the next follow-on seam.

## Changes

- promoted
  `~/Dev/projects/signal/docs/contracts/075-runtime-public-interface-decomposition-and-internal-assembly-boundary-contract.md`
  to `active`
- updated
  `~/Dev/projects/signal/docs/contracts/001-working-rules.md`
  so the current strict milestone is now `g09.007`
- marked
  `~/Dev/projects/signal/docs/roadmaps/g09/006-shared-host-runtime-execution-and-recovery-unification.md`
  as `complete`
- marked
  `~/Dev/projects/signal/docs/roadmaps/g09/007-runtime-interface-decomposition-and-test-surface-normalization.md`
  as `active` and corrected its live seam description
- added the new ready card at
  `~/Dev/projects/signal/docs/roadmaps/g09/batch-cards/004-g09-007-offline-preview-assembly-carveout.md`
- refreshed the currentness/front-door surfaces in:
  - `~/Dev/projects/signal/docs/specs/README.md`
  - `~/Dev/projects/signal/docs/specs/001-g09-lane-first-strict-adoption.md`
  - `~/Dev/projects/signal/docs/logs/README.md`
  - `~/Dev/projects/signal/docs/contracts/contract-index.md`
  - `~/Dev/projects/signal/docs/README.md`
  - `~/Dev/projects/signal/docs/roadmaps/README.md`
  - `~/Dev/projects/signal/docs/roadmaps/g09/README.md`

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Outcome

The strict lane is executable again:

- `g09.006` is closed
- `g09.007` is now the active strict milestone
- the live ready batch is the offline preview assembly carveout card
- the test-surface normalization seam remains explicit for the follow-on step

## Next Task

Continue the active strict `g09` lane from
`docs/roadmaps/g09/batch-cards/004-g09-007-offline-preview-assembly-carveout.md`.
