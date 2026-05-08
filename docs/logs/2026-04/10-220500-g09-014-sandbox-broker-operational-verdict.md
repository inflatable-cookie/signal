# 2026-04-10 - g09.014 Sandbox Broker Operational Verdict

Roadmap: `docs/roadmaps/g09/014-production-readiness-grade-and-generation-release-gate.md`
Card: `docs/roadmaps/g09/batch-cards/040-g09-014-sandbox-broker-operational-verdict.md`

Closed `040-g09-014-sandbox-broker-operational-verdict.md` by deciding the
final remaining reopened `g09` crate verdict and promoting
`signal-plugin-sandbox` to `production-ready for role`.

## What changed

- promoted `signal-plugin-sandbox` in the reopened `g09.014` readiness
  inventory
- froze one explicit broker-operational proof bundle spanning:
  - broker attached-session stream continuity
  - broker recoverable timeout after refresh
  - local VST3 broker-backed crash recovery, deferred teardown fault, and
    cleanup retry
  - server LV2 broker-backed crash recovery, deferred teardown fault, and
    cleanup retry
- updated the active roadmap and strict currentness surfaces to point at the
  final closeout card
- created `041-g09-014-final-release-gate-closeout.md` as the last remaining
  reopened `g09` batch

## Validation

- `cargo test -p signal-plugin-sandbox broker::tests::broker_streams_vst3_execution_without_tearing_down_attached_session -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-plugin-sandbox broker::tests::broker_reports_recoverable_vst3_timeout_after_refresh_cycle -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_can_drive_broker_backed_vst3_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_reports_broker_backed_vst3_deferred_teardown_fault -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-local --test public_host_edge_sandbox_broker local_public_host_edge_recovers_after_broker_backed_vst3_cleanup_retry -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_reports_broker_backed_lv2_deferred_teardown_fault -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_recovers_after_broker_backed_lv2_cleanup_retry -- --exact --nocapture --test-threads=1`
- `effigy health`
- `effigy validate`
- `effigy demo:coverage-matrix`

## Result

The reopened `g09` lane no longer has any crate-level blocked verdicts. The
remaining work is final release-gate closeout only.

## Next Task

Continue the reopened strict `g09` lane from
`docs/roadmaps/g09/batch-cards/041-g09-014-final-release-gate-closeout.md`.
