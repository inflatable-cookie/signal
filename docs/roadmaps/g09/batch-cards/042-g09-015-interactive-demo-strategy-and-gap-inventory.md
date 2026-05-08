# 042 - g09.015 Interactive Demo Strategy And Gap Inventory

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/015-operator-visible-interactive-demo-and-plugin-browser-proof.md`
Governing contracts: `docs/contracts/079-interactive-demo-binary-and-crate-capability-proof-contract.md`, `docs/contracts/081-operator-visible-interactive-demo-and-low-dependency-ui-contract.md`

## Objective

Reopen `g09` on one bounded planning batch that defines the operator-visible
interactive-demo posture, identifies which existing demo surfaces are still too
receipt-heavy, and promotes the first honest implementation card.

## Scope

- define the low-dependency UI and interaction model for Signal-owned demos
- inventory the current demo suite against that model
- promote the first ready implementation batch for the highest-value gap

## Out Of Scope

- implementing the plugin browser itself
- redesigning every existing demo in one batch
- adding a heavyweight shared UI runtime

## Acceptance Criteria

- the interaction strategy is explicit and contract-backed
- the existing demo suite is classified as already interactive enough or still
  needing operator-visible uplift
- the next implementation batch is ready without fresh planning judgment

## Validation

- `effigy tasks`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- new interactive-demo contract
- new `g09.015` roadmap milestone
- refreshed front-door/currentness surfaces
- batch log with the planning outcome

## Outcome

- reopened `g09` with `g09.015` as the active milestone
- froze the low-dependency operator-visible demo contract in `081`
- classified the current demo suite honestly:
  - existing domain demos remain valid proof surfaces
  - the highest-value missing interactive surface is still
    `signal.demo.plugin.capability-browser`
  - current adapter reality means that browser is not yet an honest first
    implementation batch because CLAP discovery is still harness-backed and
    VST3/AU discovery still relies on Signal-specific metadata files
- corrected the first ready implementation batch to
  `044-g09-015-real-plugin-discovery-gap-burn-down.md`

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/044-g09-015-real-plugin-discovery-gap-burn-down.md`.
