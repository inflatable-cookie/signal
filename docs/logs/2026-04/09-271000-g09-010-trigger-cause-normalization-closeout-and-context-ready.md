# 2026-04-09 - g09.010 Trigger Cause Normalization Closeout And Context Ready

## Summary

Closed strict card `016-g09-010-meter-continuity-trigger-cause-normalization`
after landing a shared meter continuity rule surface, then promoted the next
bounded `g09.010` seam as
`017-g09-010-meter-stage-plan-context-unification`.

## Implementation

- added `meter_state_continuity_rule_surface.rs` as the shared trigger/reason/
  cause derivation surface
- rewired `meter_state_continuity_helpers.rs` to delegate trigger and reason
  selection through the shared rule surface
- rewired `meter_state_continuity_cause_stack.rs` to derive primary causes
  through the shared rule surface instead of repeating mapping logic
- reassessed the remaining meter continuity seam and promoted the duplicated
  stage-versus-plan assembly shell in `meter_state_continuity_context.rs` as
  the next honest strict card

## Validation

- `cargo check -p signal-analysis-rhythm`
- `cargo test -p signal-analysis-rhythm meter_state::meter_state_continuity_rule_surface::tests::meter_continuity_rule_surface_preserves_trigger_reason_and_cause_mapping -- --exact --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm meter_continuity_reason -- --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm meter_continuity_triggers -- --nocapture --test-threads=1`
- `cargo test -p signal-analysis-rhythm meter_continuity_causes -- --nocapture --test-threads=1`
- `effigy health`
- `effigy qa:docs`
- `effigy qa:northstar`

## Notes

- corpus-proof and demo work remain deferred to later `g09.010` or `g09.011`
  surfaces
- the next bounded seam is still inside policy normalization, not new rhythm
  heuristics

## Next Task

Continue the active strict `g09` lane from
`docs/specs/batch-cards/017-g09-010-meter-stage-plan-context-unification.md`.
