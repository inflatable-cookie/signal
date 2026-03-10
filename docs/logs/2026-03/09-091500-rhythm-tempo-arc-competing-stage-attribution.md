# Rhythm Tempo Arc Competing Stage Attribution

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc inflection surface so Signal can publish a
competing stage when both the immediate next stage and the terminal clear
horizon materially shape the downgrade path. The primary inflection marker
still names the dominant stage, but callers can now see when the other stage
is also contributing meaningful pressure.

## Work completed

- extended `TempoContinuityArcDowngradeInflection` in
  `crates/signal-analysis-rhythm/src/lib.rs` with:
  - `competing_stage`
  - `competing_after_beats`
  - `competing_delta`
  - `competing_support`
- calibrated competing-stage attribution so:
  - stable integer lock still points at `NextStage`, but now exposes
    `TerminalClear` as a meaningful secondary horizon
  - boundary-drift core-window carry still points at `NextStage`, but now
    exposes `TerminalClear` as a competing long-horizon pressure source
  - guarded refined reacquisition still points at `NextStage`, but now exposes
    `TerminalClear` when ambiguity carry still leaves a significant terminal
    clear path
  - flat cleared tempo continues to expose no competing stage
- updated `offline_rhythm_demo` to print the competing-stage attribution inline
  with the existing inflection output
- expanded the direct tempo-state tests and aggregate arc calibration test so
  competing-stage behavior is part of the public contract

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health --repo .`
- `effigy test --repo .`
- `effigy validate --repo .`
- `git diff --check`

## Notes

- The competing-stage layer is intentionally relative to the primary marker. It
  does not replace the main inflection stage; it explains when another stage is
  close enough in pressure to matter to downstream tempo-state policy.
- This keeps the arc interpretation Signal-owned and avoids forcing Finch to
  infer whether terminal clear pressure is still relevant when the next-stage
  marker remains primary.

## Next Task

Deepen the tempo continuity arc inflection surface with explicit stage-balance
or weighting metadata, so Signal can quantify how much the primary and
competing stages each contribute instead of only exposing the secondary stage
when it clears a materiality threshold.
