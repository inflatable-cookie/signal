# 2026-03-16 18:26:55 UTC - g06.019 Integrated Acceptance Lane Tranche

## Summary

Implemented the first real shared integrated acceptance lane for `g06`. The
batch adds a machine-readable supervisor-tools descriptor plus a grouped Effigy
task for the required cross-family path, and it repairs stale watchdog-restart
expectations that the new lane exposed in the interruption proofs.

## Work completed

- added the machine-readable integrated acceptance descriptor:
  - `crates/signal-supervisor-tools/src/main.rs`
- added the grouped required lane task:
  - `effigy.toml`
- repaired the stale interruption-boundary watchdog restart proofs so they
  match the current safe-mode restart threshold:
  - `crates/signal-runtime/tests/public_contract_boundary.rs`
  - `crates/signal-runtime/src/runtime.rs`
- recorded the Batch 19.2 outcome in:
  - `docs/roadmaps/g06/019-fault-injection-harnesses-and-multi-backend-acceptance-depth.md`
  - `docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md`
- updated the shared reference and next-task trail:
  - `docs/contracts/README.md`
  - `docs/roadmaps/g06/README.md`
  - `docs/roadmaps/README.md`
  - `docs/roadmaps/generation-index.md`
  - `docs/architecture/graph-runtime-feature-reference.md`

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_observation_report_surfaces_restartable_interruption_summary -- --nocapture`
- `cargo test -p signal-runtime public_runtime_interruption_boundary_reports_restartable_runtime_state -- --nocapture`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_integrated_acceptance_lane_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools integrated_acceptance_lane_json_reports_required_and_advisory_policy -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json`
- `effigy acceptance:integrated-acceptance-lane --repo .`

## Deferred scope

- Batch 19.2 makes the grouped lane real, but it does not yet prove the lane
  yields genuinely cross-family evidence beyond the boundary tasks it composes
- longer-session soak thresholds, promotion gates, and broader unstable
  recovery-overlap scenarios remain outside this batch and still belong to
  `g06.020` or later follow-up

## Next Task

Continue `g06.019` with Batch 19.3 by adding focused proofs that the new
integrated acceptance lane provides meaningful cross-family evidence instead of
only re-listing the milestone-local boundary tasks it composes.
