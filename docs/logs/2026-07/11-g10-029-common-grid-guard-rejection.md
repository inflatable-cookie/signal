# g10.029 Common-Grid Guard Rejection

Date: 2026-07-11
Status: rejected before synthesis

## Result

Batch 29.6N stops at its first guard channel. The exact complete-frame
canonical dual for lowpass channel `0` does not meet the bounded two-sided crop
guard.

## Evidence

- probe transform: `34176` frames
- evaluated channels: `1/1536` by fail-fast policy
- legal guard cap: `16384` frames
- guard lower bound: `16768` frames
- excluded energy at largest legal support radius: `6.270778968e-7`
- canonical-dual block-solve residual: `1.051209509e-12`
- non-finite values: `0`
- repeated evidence and atom hash: exact
- dual atom hash: `e9533630d4621fb6`

The guard misses its `1e-12` energy limit by more than five orders of magnitude.
This is not a frame-solve accuracy failure.

## Boundary

No coefficient assembly, inverse transform, audio output, corpus render,
stereo, dynamic ratio, or product route was opened.

## Next Task

Freeze a report-only diagnostic that attributes the limiting tail across the
analysis filter, canonical dual, tightening, analytic mirroring, and lowpass
completion stages.
