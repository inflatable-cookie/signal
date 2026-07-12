# g10.029 Mixed-Phase Distribution Rejection

Date: 2026-07-12
Status: operator review required

## Outcome

- fixed audit pairs: `25`
- separating pairs: `0`
- minimum chirp leakage: `0.7759762445`
- mixed-event recall at cutoff `0`, radius `0.125`: `0.1161677536`
- mixed-event recall at cutoff `0.01`, radius `0.125`: `0.0078331429`
- maximum equivalence error: `2.6562923909e-5`
- equivalence owner: boundary impulse control
- structural failures: `[0,0,0,1]`
- evidence hash: `5b3becee90745c1f`

Cell accounting, quantile ordering, finiteness, repeat, gain, polarity, hard
pan, and channel swap pass. Every cutoff/radius pair overlaps event and negative
families. Chirp evidence remains dominant near the nominal impulsive phase;
useful magnitude cutoffs erase mixed-event or isolated-impulse evidence.

The mixed-phase family is rejected before smoothing, prominence, schedule
mapping, or audio.

## Next Task

Operator review must choose a different transient evidence family or pause.
