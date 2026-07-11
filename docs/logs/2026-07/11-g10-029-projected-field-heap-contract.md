# g10.029 Projected Field And Heap Contract

Date: 2026-07-11
Status: decision frozen

## Decision

Batch 29.6M is report-only. At exact source coordinate `u=m/ratio`, interpolate
coefficient magnitude, absolute instantaneous angular frequency, and
delay-compensated vertical phase derivatives. Never interpolate wrapped
complex coefficients.

Deterministic phase seeds use the nearest source column; halfway ties choose
the lower column. This keeps wrapped phase outside linear interpolation.

Integrate positive-frequency phases one output column at a time. Horizontal
candidates advance from the preceding solved column. Vertical candidates
advance from an adjacent solved channel. Magnitude owns priority; deterministic
direction, target-channel, and predecessor-channel ordering break ties.

## Stop Gate

Ratios `0.75`, `1.0`, and `1.5` must prove `1e-9` coordinate accuracy,
monotonicity, padding coverage, finite fields, exact projected-column counts,
complete single assignment, a duration-independent `3072`-entry heap bound,
and identical repeat hashes across the Contract `082` synthetic controls.

No canonical-dual audio synthesis, corpus render, linked stereo, dynamic ratio,
or product route opens in this proof.

## Runway

- Batch 29.6M: projected fields and bounded phase assignment
- Batch 29.6N: separately frozen synthetic canonical-dual synthesis and
  placement proof
- Batch 29.6 mono gate: unchanged 60-row corpus only after synthetic synthesis
  passes
- Batch 29.7: shared-decision linked stereo only after the complete mono gate

## Next Task

Implement Batch 29.6M and stop on projection, padding, finite-value, assignment,
heap-bound, or determinism failure.
