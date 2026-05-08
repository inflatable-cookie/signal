# 037 - g09.014 Workspace Validate Surface Repair

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Governing contracts: `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/080-production-readiness-grade-and-generation-release-gate-contract.md`

## Objective

Repair the broken workspace validation wall so the reopened `g09` gate can use
`effigy validate` and `cargo test --workspace --no-run` as trustworthy required
or promotable evidence instead of leaving them permanently deferred.

## Scope

- repair the stale split test-module tree currently breaking
  `signal-host-local` and `signal-host-server` workspace test compilation
- repair the related host-test import drift surfaced by the same workspace
  validate pass
- keep the batch focused on restoring the repo-owned validate surface rather
  than widening into unrelated host feature work
- update the readiness gate docs if the repaired validation surface changes the
  required versus advisory evidence posture

## Out Of Scope

- broad host/runtime behavior refactors
- plugin, broker, or hardware production-depth implementation
- opening a new generation

## Acceptance Criteria

- `cargo test --workspace --no-run` completes successfully
- `effigy validate` becomes a trustworthy runnable gate surface again
- the reopened `g09` release-gate docs classify that surface honestly after the
  repair

## Validation

- `effigy validate`
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- repaired host test-module and import surface
- updated `g09.014` roadmap and currentness surfaces if the gate posture
  changes
- batch log with validation actually run

## Stop Conditions

- the compile failures turn out to require broad host test-tree redesign beyond
  this bounded seam
- the validate wall is blocked by unrelated crate families outside the current
  host test-module failures

## Outcome

- repaired the stale split test-module tree in `signal-host-local` and
  `signal-host-server` so workspace lib and public-boundary test compilation no
  longer fails under the repo-owned validate sweep
- repaired the related host-test import and type drift surfaced by that compile
  wall
- restored `effigy validate` and `cargo test --workspace --no-run` as
  trustworthy runnable release-gate evidence
- kept the result honest by leaving the remaining workspace state as warnings,
  not hidden blockers

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/038-g09-014-plugin-broker-readiness-verdict.md`.
