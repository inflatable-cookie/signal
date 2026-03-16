# 2026-03-16 19:28:48 UTC - g06.020 closeout surface and soak lane tranche

## Summary

Implemented the first runnable `g06` closeout surface by adding a bounded
long-session soak descriptor and Effigy lane, then retargeting the generation
closeout descriptor and task to the actual `g06` authority chain.

## Why this tranche matters

`g06.020` could not close on policy alone. This tranche turns the frozen
closeout contract into machine-readable and runnable shared surfaces so the
generation has one real repo-owned gate before the final Loophole-facing
readiness review.

## What changed

- added `signal.g06.long-session-soak-lane` to
  `crates/signal-supervisor-tools/src/main.rs`
- added `acceptance:g06-soak-lane` and `acceptance:g06-closeout` to
  `effigy.toml`
- updated the generation-closeout descriptor to report `g06`-specific contract,
  validation, residual-risk, and readiness-area state
- updated the roadmap, contract, and reference trail to mark Batch 20.2
  complete and move the queue to Batch 20.3

## Validation

- `cargo fmt --all`
- `cargo test -p signal-supervisor-tools parse_args_supports_describe_g06_soak_lane_mode -- --nocapture`
- `cargo test -p signal-supervisor-tools g06_soak_lane_json_reports_required_and_deferred_policy -- --nocapture`
- `cargo test -p signal-supervisor-tools generation_closeout_json_reports_combined_boundary_and_next_queue -- --nocapture`
- `cargo run -p signal-supervisor-tools -- --describe-g06-soak-lane --format=json`
- `cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json`
- `effigy acceptance:g06-closeout --repo .`
- `git diff --check`
- `effigy health --repo .`
- `effigy qa:docs --repo .`

## Residual risk

The broader `server soak` path is still deferred because the current
recovery-overlap attach-limit issue is not stable enough for the required gate,
and the final Loophole-facing readiness verdict still belongs to Batch 20.3.
