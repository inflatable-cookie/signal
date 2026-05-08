# 036 - g09.014 Release Gate Baseline

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Governing contracts: `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/080-production-readiness-grade-and-generation-release-gate-contract.md`

## Objective

Build the first repo-owned production-readiness gate surface for reopened
`g09`: define the required, advisory, and deferred evidence set and repair any
broken gate wiring needed so later crate verdicts can be promoted against a
real runnable baseline.

## Scope

- define the required/advisory/deferred evidence families for the reopened
  `g09` gate
- map those families onto existing Effigy tasks, descriptors, and proof
  surfaces where possible
- repair any stale gate-surface wiring or proof-reference drift needed to make
  that baseline honest
- keep the work additive over existing proof surfaces instead of inventing new
  technical behavior

## Out Of Scope

- burning down every blocked plugin, host, runtime, or hardware gap
- new runtime or adapter implementation work
- opening a new generation

## Acceptance Criteria

- one explicit repo-owned `g09` production-readiness gate posture exists
- required, advisory, and deferred evidence are visible in canonical docs state
- later `g09.014` batches can target blocked crate groups against a stable gate

## Validation

- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated `g09.014` roadmap and strict surfaces
- explicit gate evidence mapping in repo docs
- batch log with validation actually run

## Stop Conditions

- the gate baseline cannot be expressed honestly without fresh contract work
  beyond `080`
- implementation gaps turn out to be prerequisites for merely naming the gate

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/037-g09-014-workspace-validate-surface-repair.md`.
