# 2026-04-09 - g09.010 Meter Plan Shell Closeout And Trigger Ready

## Summary

Closed the third strict `g09.010` batch by extracting the repeated meter
continuity plan shell into a shared staged builder, then promoted the next
honest meter continuity trigger/cause normalization seam as the ready card.

## Implementation

- added a shared meter continuity staged-plan builder in
  `crates/signal-analysis-rhythm/src/meter_state/meter_state_continuity_plan_shell.rs`
- rewired `meter_state_continuity_hold_arms.rs`,
  `meter_state_continuity_lock_arms.rs`, and
  `meter_state_continuity_watch_clear_arms.rs` to declare their policy through
  that shared shell while keeping hold, lock, watch, and clear differences
  explicit
- added focused helper coverage proving the shared shell preserves per-plan
  differences

## Validation

- `cargo check -p signal-analysis-rhythm`
- `cargo test -p signal-analysis-rhythm meter_state::meter_state_continuity_plan_shell::tests::meter_plan_shell_preserves_per_plan_policy_differences -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm meter_continuity_actions -- --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm meter_continuity_triggers -- --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm meter_continuity_reason -- --nocapture --test-threads=1`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Planning Reassessment

- the repeated meter continuity plan shell is now closed
- corpus-proof and demo work remain deferred
- the next honest strict seam is the parallel trigger/reason/cause derivation
  logic across `meter_state_continuity_helpers.rs` and
  `meter_state_continuity_cause_stack.rs`
- promoted new ready card:
  `docs/specs/batch-cards/016-g09-010-meter-continuity-trigger-cause-normalization.md`

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/016-g09-010-meter-continuity-trigger-cause-normalization.md`.
