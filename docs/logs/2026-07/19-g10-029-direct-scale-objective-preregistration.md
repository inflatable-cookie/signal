# g10.029 Direct Scale Objective Preregistration

Date: 2026-07-19
Batch: 29.7AU
Status: active, sequence frozen before audio

## Decision

Run one direct scale-timeline candidate through the unchanged failure-first
objective gate. Rule 31Z representation hash `fdf90f6127749341`, direct-state
hash `430543f8e1dce721`, geometry, masks, windows, thresholds, ties, capacities,
and corpus thresholds do not move.

The no-audio entry gate reruns all release-only Rule 31Z representation and
state mechanics. Candidate evidence then runs in this order:

1. silence, tone, noise, impulse, mixed, and transient synthetic sources at
   `0.75`, `1.5`, and `2.0`
2. one corrected `48`-row stereo corpus run
3. six unchanged exact-source mono rows
4. their unchanged long-development measurements

Every later stage requires complete prior passage. Synthetic evidence must be
structurally finite, deterministic, bounded by prepared storage, exercise all
non-scripted terminal states, and keep the four hard channel mechanics at or
below `1e-6`. Stereo requires zero calibrated and structural failures,
deterministic repeat, at least `245/384` improved local windows, at most
`13/48` Signal-relative local-row failures, and maximum normalized-Gram
residual `0.01744693815260`. Mono and long-development each require zero hard
failures and no row-complete regression against current Signal.

## Renderer Boundary

The direct renderer uses the absolute target-to-source tick map, one analysis
and inverse transform per active scale and channel per tick, the fixed ten-
tick coefficient and nineteen-tick magnitude rings, one shared terminal
decision per atom, same-channel scale summation in the `8HC` output ring, and
exact target crop. Caller-owned returned audio is not processing history.

## Stop Rule

Stop at the first miss. No factor sweep, row repair, policy change, retry,
fallback, listening, export, concealed material, or holdout access is allowed.
Only complete passage may open Batch 29.8.

## Next Task

Implement the frozen direct renderer, rerun the no-audio entry gate, then
execute objective stages in order through the first miss.
