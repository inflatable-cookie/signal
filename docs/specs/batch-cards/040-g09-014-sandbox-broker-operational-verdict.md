# 040 - g09.014 Sandbox Broker Operational Verdict

Status: complete
Owner: core-product
Updated: 2026-04-10
Parent roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Governing contracts: `docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md`, `docs/contracts/072-real-plugin-hosting-discovery-and-sandbox-execution.md`, `docs/contracts/074-shared-host-runtime-execution-and-recovery-unification.md`, `docs/contracts/080-production-readiness-grade-and-generation-release-gate-contract.md`

## Objective

Decide the final remaining crate-level blocker in reopened `g09`: either
promote `signal-plugin-sandbox` to `production-ready for role` through one
explicit long-lived broker operational proof bundle, or leave it blocked with a
named residual gap that prevents `g09` closeout.

## Scope

- define the smallest honest required evidence bundle for a long-lived
  broker-operational verdict
- use existing broker-backed recovery, deferred-teardown, and cleanup-retry
  proof surfaces where possible
- repair only narrow proof-wiring or operational-verdict gaps if required for
  an honest decision
- do not widen into new plugin hosting or backend feature work

## Out Of Scope

- general sandbox feature expansion
- new plugin capability browsing work
- reopening already-promoted runtime, host, hardware, or adapter verdicts

## Acceptance Criteria

- `signal-plugin-sandbox` has an explicit updated verdict
- the verdict is backed by one named runnable operational proof bundle
- the result is strong enough to decide whether `g09` can enter final release
  gate closeout or must stay open

## Validation

- `effigy health`
- `effigy validate`
- `effigy demo:coverage-matrix`
- focused broker-backed recovery, cleanup, and lifecycle proof commands
- `effigy qa:docs`
- `effigy qa:northstar`

## Evidence

- updated `g09.014` inventory and gate docs
- explicit broker-operational verdict notes with runnable proof references
- batch log with validation actually run

## Stop Conditions

- the remaining broker gap still needs substantial new implementation rather
  than an operational verdict
- the broker proof surface fragments into multiple unrelated seams that cannot
  honestly fit in one bounded batch

## Outcome

- promoted `signal-plugin-sandbox` to `production-ready for role`
- named one explicit runnable broker-operational proof bundle:
  - `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
  - `cargo test -p signal-plugin-sandbox broker::tests::broker_reports_recoverable_vst3_timeout_after_refresh_cycle -- --exact --nocapture --test-threads=1`
  - `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
  - `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_reports_broker_backed_vst3_deferred_teardown_fault -- --exact --nocapture --test-threads=1`
  - `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_recovers_after_broker_backed_vst3_cleanup_retry -- --exact --nocapture --test-threads=1`
  - `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
  - `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_reports_broker_backed_lv2_deferred_teardown_fault -- --exact --nocapture --test-threads=1`
  - `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_recovers_after_broker_backed_lv2_cleanup_retry -- --exact --nocapture --test-threads=1`
- removed the final remaining crate-level blocker from reopened `g09`
- narrowed the lane to one final release-gate closeout batch rather than any
  further readiness burn-down work

## Next Task

Continue the reopened strict `g09` lane from
`docs/specs/batch-cards/041-g09-014-final-release-gate-closeout.md`.
