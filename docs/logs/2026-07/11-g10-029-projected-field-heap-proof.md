# g10.029 Projected Field And Heap Proof

Date: 2026-07-11
Status: passed

## Result

Batch 29.6M passes. The report-only path projects magnitude, absolute
instantaneous angular frequency, and delay-compensated vertical phase
derivatives at exact source coordinate `u=m/ratio`. Wrapped phase uses a
deterministic nearest-column seed and is never linearly interpolated.

Positive-frequency phase assignment stays output-column-local. The heap is
preallocated and structurally capped at `3072` candidates, independent of
render duration.

## Evidence

The `0.75`, `1.0`, and `1.5` ratios pass steady low/mid/high tones, two-tone,
linear and exponential chirps, impulse, deterministic noise, mixed
tonal/transient content, and silence:

- maximum coordinate error: `0`
- horizontal assignments: `34592`
- vertical assignments: `10405`
- duplicate assignments: `0`
- missing significant assignments: `0`
- maximum heap high-water: `1756/3072`
- non-finite projected or phase values: `0`
- fractional and boundary-pad cases: exercised
- repeated evidence and hashes: exact

## Boundary

No canonical-dual audio synthesis, corpus render, linked stereo, dynamic ratio,
or product route opens in this batch.

## Next Task

Freeze Batch 29.6N canonical-dual synthetic synthesis and placement gates.
