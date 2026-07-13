# g10.029 Adaptive Study And Time-Map Proof

Date: 2026-07-13

## Scope

Batch 29.6BQ attaches the frozen linked study and Rule 30C global output map to
the single-owner adaptive frame. Coefficients and phase remain unchanged.
Corpus audio, holdout, and tuning remain closed.

## Result

The proof passes at ratios `0.75`, `1.5`, and `2.0`.

- each control selects `15` responsive study points
- each produces `104` adaptive frames: `81` in-range and `23` reflected
- window counts are `[53,24,16,11]` for `512/1024/2048/4096`
- source hops span `128..512`
- output hops span `85..376`, `132..800`, and `134..1091`
- duplicate source/output centres, off-grid centres, illegal transitions,
  non-positive hops, endpoint mismatches, and linked-order mismatches are zero
- per-window-level mapping disagreements and selected-event movement are zero
- evidence hash `3ea1d3a2297083e2` repeats exactly

The earlier identity hash `6987080e517f1aec` and ownership hash
`2a29d952d91e92ba` remain unchanged. Timing is attached without revising the
study, adaptive geometry, coefficients, or phase.

## Decision

Open Batch 29.6BR under Contract `082`, Rule 30M. Prove positive coverage and
the exact output-lattice diagonal dual before interpreting stretched phase.
Then prove one continuous actual-hop phase state, separate selected-event
correction, and current-frame vertical locking on frozen synthetic controls.

## Next Task

Execute Batch 29.6BR single-frame phase and synthesis proof. Keep corpus audio,
holdout, tuning, stereo promotion, dynamic ratio, and product routing closed.
