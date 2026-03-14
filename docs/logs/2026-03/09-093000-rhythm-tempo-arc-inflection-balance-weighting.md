# Rhythm Tempo Arc Inflection Balance Weighting

Date: 2026-03-09
Owner: core-product

## Summary

Extended the tempo continuity arc inflection surface with explicit stage-balance
weighting. Signal now quantifies how much the primary and competing stages each
contribute to the current downgrade path, instead of only exposing the
secondary stage when it crosses a materiality threshold.

## Work completed

- added `TempoContinuityArcDowngradeInflectionBalance` to
  `crates/signal-analysis-rhythm/src/lib.rs`
- extended `TempoContinuityArcDowngradeInflection` with:
  - `balance.primary_weight`
  - `balance.competing_weight`
  - `balance.unattributed_weight`
  - `balance.dominance`
- calibrated the balance model so:
  - stable integer lock keeps `NextStage` primary while still showing a smaller
    but non-zero `TerminalClear` contribution
  - boundary-drift core-window carry keeps `NextStage` primary while exposing a
    meaningful long-horizon clear contribution
  - guarded refined reacquisition keeps `NextStage` primary while exposing the
    residual terminal-clear share behind ambiguity carry
  - flat cleared tempo remains fully unattributed, with zero primary and
    competing weight
- updated `offline_rhythm_demo` to print the balance tuple inline with the
  existing inflection and competing-stage output
- expanded the direct tempo-state tests and aggregate arc calibration checks so
  the weighting surface is pinned as part of the public contract

## Validation

- `cargo test -p signal-analysis-rhythm --no-run`
- `cargo check -p signal-analysis-rhythm --example offline_rhythm_demo`
- `effigy health`
- `effigy test`
- `effigy validate`
- `git diff --check`

## Notes

- The weighting model stays intentionally relative to the current inflection
  deltas. It is meant to explain how much the modeled downgrade path is being
  shaped by the primary and competing stages, not to replace the higher-level
  arc classification.
- `unattributed_weight` preserves room for flat or weakly shaped paths where
  neither staged horizon carries strong modeled pressure.

## Next Task

Deepen the tempo continuity arc inflection surface with stage-specific rationale
weighting, so Signal can quantify not just which stages contribute to the
downgrade path, but whether that contribution is being driven more by boundary
drift, ambiguity carry, or terminal evidence loss within each stage.
