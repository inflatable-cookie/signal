# g04.003 Deferred Service Export Closure Tranche

Date: 2026-03-12
Scope: `crates/signal-runtime/`, `crates/signal-host-local/`, `crates/signal-host-server/`, `crates/signal-supervisor-tools/`, `docs/contracts/`, `docs/architecture/`, `docs/roadmaps/g04/`

## Summary

Closed `g04.003` by carrying the deferred-service decision surface through
runtime observation/supervisor export and proving it on two real service
families.

## What changed

- extended the typed deferred-service receipt model beyond offline render queue
  into offline render purge
- stored the latest deferred-service receipt in `signal-runtime` and projected
  it through `RuntimeObservationReport` and `RuntimeSupervisorReport`
- kept host adapters aligned with the runtime-owned queue/purge supervisor
  surface so downstream consumers do not need private runtime access
- added focused proofs for:
  - queue `Run`, `Throttle`, and `Defer` behavior under live and safe-mode
    runtime state
  - purge defer/resume behavior under safe mode
  - consumer-facing supervisor export of the latest deferred-service receipt
- closed `g04.003` and opened `g04.004`

## Why this tranche

Batch 3.2 established one real orchestration baseline, but `g04.003` was not
finished until consumers could observe that policy through the shared report
path and at least one more deferred service family used the same typed
decision surface.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime --lib runtime_offline_render_queue`
- `cargo test -p signal-runtime --lib runtime_purge_defers_in_safe_mode_and_observation_export_surfaces_last_decision`
- `cargo test -p signal-supervisor-tools export_json_carries_last_deferred_service_receipt`
- `git diff --check`
- `effigy health`

## Next

Continue `g04.004` with Batch 4.1 and define the backend-neutral hardware
capability and clock-domain contract on top of the closed scheduling and
deferred-work substrate.
