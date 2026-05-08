# 038 - g09.014 Plugin Broker And IPC Readiness Verdict

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Governing contracts: `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution.md`, `docs/contracts/080-production-readiness-grade-and-generation-release-gate-contract.md`

## Objective

Use the repaired release gate to classify the plugin, broker, and transport
family honestly: decide which of `signal-plugin`, `signal-plugin-clap`,
`signal-plugin-vst3`, `signal-plugin-au`, `signal-plugin-lv2`,
`signal-plugin-sandbox`, and `signal-ipc` are now production-ready for role,
and which still need explicit blocking work.

## Scope

- define one focused required evidence bundle for the plugin, broker, and IPC
  family using the real adapter, sandbox, broker, and demo proof surfaces that
  already exist
- repair only narrow proof-wiring or gate-reference drift if that is required
  to make the verdict honest
- update the `g09.014` inventory and release-gate docs to promote crates that
  clear the repaired gate or leave explicit blockers for the ones that do not
- keep the batch inside readiness classification rather than widening into new
  plugin-capability or host-feature implementation

## Out Of Scope

- inventing the deferred plugin capability browser surface
- broad new adapter or broker implementation work beyond narrow proof repair
- host, runtime, or hardware readiness verdicts outside the plugin family

## Acceptance Criteria

- each plugin, broker, and IPC crate has an explicit updated verdict in the
  reopened `g09` readiness inventory
- the required evidence bundle for that family is named and actually runnable
- any remaining plugin-family blockers are explicit and narrow rather than
  buried in broad `production-capable but blocked` language

## Validation

- `effigy health`
- `effigy validate`
- `effigy demo:coverage-matrix`
- focused plugin, broker, and IPC proof commands actually used for the verdict
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated `g09.014` readiness inventory and gate docs
- explicit plugin-family verdict notes with runnable proof references
- batch log with validation actually run

## Stop Conditions

- one or more crates still need substantial new implementation rather than a
  readiness verdict and proof-bundle decision
- the family turns out to split into separate seams that cannot honestly be
  handled in one bounded classification batch

## Outcome

- promoted `signal-plugin`, `signal-plugin-clap`, `signal-plugin-vst3`,
  `signal-plugin-au`, `signal-plugin-lv2`, and `signal-ipc` to
  `production-ready for role`
- kept `signal-plugin-sandbox` blocked on one explicit remaining gap:
  there is still no repo-owned long-lived broker operational verdict beyond the
  bounded lifecycle, receipt, and demo surfaces already in place
- narrowed the remaining reopened `g09` burn-down to the shared
  runtime/host/hardware/broker operational family instead of keeping the
  plugin adapters in a vague blocked bucket

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/039-g09-014-runtime-host-hardware-broker-operational-verdict.md`.
