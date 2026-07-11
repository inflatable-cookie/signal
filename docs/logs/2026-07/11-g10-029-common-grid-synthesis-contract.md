# g10.029 Common-Grid Synthesis Contract

Date: 2026-07-11
Status: decision frozen

## Decision

Batch 29.6N synthesizes a guard-protected centre. It does not treat a
zero-origin circular transform as an honest audio boundary.

Measure every finalized canonical-dual atom. Select the smallest whole
`384`-frame two-sided guard whose excluded energy is at most `1e-12` for every
channel, plus one projected-field neighbor column. Reject the mechanism if the
guard exceeds `16384` frames.

Whole-sample even reflection supplies source padding. Batch 29.6M projects and
integrates the guarded field. Complete canonical-dual block solves assemble the
positive spectrum; explicit mirroring and real DC/Nyquist values produce the
real inverse transform. Crop only the guard-protected centre.

## Stop Gate

Identity, `0.75x`, and `1.5x` controls must pass guard, dual residual, symmetry,
imaginary residue, exact length, identity reconstruction, impulse placement,
silence, finite-value, and repeat-hash gates from Contract `082`.

No fade, normalization, zero fill, endpoint correction, corpus render, linked
stereo, dynamic ratio, production route, or product integration is allowed.

## Runway

- Batch 29.6N: guarded synthetic synthesis
- Batch 29.6 mono gate: unchanged 60-row fixed-ratio corpus
- Batch 29.7: shared-decision linked stereo after complete mono passage
- Batch 29.8: listening and dynamic-ratio checkpoint

## Next Task

Implement the dual-atom guard proof. Stop before coefficient assembly on guard
radius, tail-energy, finite-value, or determinism failure.
