# 039 - g09.014 Runtime Host Hardware Broker Operational Verdict

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Governing contracts: `docs/contracts/003-crate-maturity-and-public-runtime-boundary-baseline.md`, `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/073-native-backend-device-truth-and-coreaudio-implementation.md`, `docs/contracts/074-shared-host-runtime-execution-and-recovery-unification.md`, `docs/contracts/080-production-readiness-grade-and-generation-release-gate-contract.md`

## Objective

Classify the remaining operational family honestly: decide the readiness verdict
for `signal-runtime`, `signal-host-local`, `signal-host-server`,
`signal-hardware`, `signal-hardware-coreaudio`, `signal-supervisor-tools`, and
the still-blocked `signal-plugin-sandbox`.

## Scope

- define one focused required evidence bundle for the runtime, host, hardware,
  supervisor, and broker-operational family
- promote any crates in that family that already clear the repaired gate
- keep any remaining blockers explicit and narrow, especially if
  `signal-plugin-sandbox` still lacks a role-correct long-lived broker verdict
- avoid widening into new backend, host, or broker implementation unless a
  narrow proof-wiring repair is required for an honest verdict

## Out Of Scope

- new plugin capability browsing work
- broad feature implementation across runtime, hosts, or hardware
- opening a new generation

## Acceptance Criteria

- each remaining operational-family crate has an explicit updated verdict
- the required evidence bundle for that family is named and actually runnable
- any remaining blocked scope is narrow enough to decide whether `g09` can
  close or still needs one final burn-down seam

## Validation

- `effigy health`
- `effigy validate`
- `effigy demo:coverage-matrix`
- focused runtime, host, hardware, supervisor, and broker proof commands
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated `g09.014` inventory and gate docs
- explicit operational-family verdict notes with runnable proof references
- batch log with validation actually run

## Stop Conditions

- the remaining operational family still hides multiple unrelated blockers that
  cannot honestly fit in one bounded verdict batch
- one of the remaining crates needs substantial new implementation rather than
  a readiness verdict

## Outcome

- promoted `signal-runtime`, `signal-host-local`, `signal-host-server`,
  `signal-hardware`, `signal-hardware-coreaudio`, and
  `signal-supervisor-tools` to `production-ready for role`
- kept `signal-plugin-sandbox` blocked as the final remaining crate-level gap
  in reopened `g09`
- narrowed that blocker to one explicit seam: a repo-owned long-lived broker
  operational verdict is still missing beyond the bounded lifecycle, receipt,
  and demo surfaces already in place

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/040-g09-014-sandbox-broker-operational-verdict.md`.
