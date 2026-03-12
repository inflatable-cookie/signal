# g04.002 Batch 2.2 Closure Transition Acceptance Tranche

Date: 2026-03-12
Scope: `crates/signal-runtime/`, `docs/architecture/`, `docs/contracts/`,
`docs/roadmaps/g04/`

## Summary

Closed Batch 2.2 by proving the runtime-owned schedule-width policy survives
restart, reconfigure, and mixed execution-class graph transitions without
falling back to a different scheduler model.

## What changed

- added a compact acceptance proof that compatible schedule width still drives
  widened requested/effective prework service scope after restart and after
  reconfigure/start churn
- added a mixed execution-class graph-transition proof showing the widened
  schedule policy survives graph replacement across pure, stateful,
  plugin-backed, and latency-bearing nodes while scheduler receipts stay
  coherent
- updated the multicore scheduling contract, architecture reference, and
  roadmap so Batch 2.2 is now explicitly closed and Batch 2.3 is the next
  active step

## Why this tranche

The prior Batch 2.2 work proved steady-state service behavior plus refresh and
gate collapse, but the batch was not really done until lifecycle churn and
execution-class transitions were covered by the same runtime-owned contract.
This tranche closes that gap and gives `g04.002` a tighter acceptance spine.

## Validation

- `cargo fmt --all`
- `cargo test -p signal-runtime runtime_schedule_width_survives_restart_and_reconfigure_transitions`
- `cargo test -p signal-runtime runtime_mixed_execution_class_graph_transition_reuses_schedule_widened_scope`
- `cargo test -p signal-runtime runtime_restart_and_reconfigure_keep_realtime_scheduler_window_coherent`
- `cargo test -p signal-runtime runtime_forecast_profile_change_keeps_realtime_scheduler_coherent`
- `git diff --check`
- `effigy health --repo .`

## Next

Continue `g04.002` with Batch 2.3 by adding focused stress proofs for mixed
execution-class graphs, invalidation-heavy transitions, and constrained
anticipative windows, then record the deferred performance risks that still
belong to later regression work.
