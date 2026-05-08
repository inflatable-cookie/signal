# 035 - g09.014 Readiness Rubric And Gap Inventory

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Governing contracts: `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/080-production-readiness-grade-and-generation-release-gate-contract.md`

## Objective

Reopen `g09` by defining the production-readiness rubric for the existing
crates and producing one explicit gap inventory that says which crates are
already production-credible for their intended role and which still block
generation closeout.

## Scope

- define the readiness rubric against the crate-role vocabulary from contract
  `003`
- inventory every active workspace crate into:
  - production-ready for role
  - production-capable but blocked by named remaining work
  - explicitly deferred or not ready
- tie each verdict back to concrete existing proof surfaces, validation, and
  open gaps
- record the first blocking-gap groups that must become later `g09.014` batches

## Out Of Scope

- implementing the blocking fixes themselves
- opening a new generation
- reworking the older closed `g09` technical milestones

## Acceptance Criteria

- one repo-owned readiness rubric exists in canonical docs state
- every active crate has an explicit readiness verdict for its intended role
- the blocking gaps are grouped into a small number of plausible next batches
- `g09` is visibly reopened and no longer claims completion prematurely

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated contract and roadmap state for the reopened `g09` boundary
- explicit readiness inventory in repo docs
- batch log with the recovery rationale and validation actually run

## Stop Conditions

- the inventory cannot be done honestly without fresh architecture or contract
  work beyond the reopened gate
- the required readiness vocabulary cannot be expressed through the existing
  crate-role baseline

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/036-g09-014-release-gate-baseline.md`.
