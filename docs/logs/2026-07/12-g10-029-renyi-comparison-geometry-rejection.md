# g10.029 Rényi Comparison-Geometry Rejection

Date: 2026-07-12
Status: operator review required

## Passing Evidence

- exact membership `[29,13,5,1]` at every anchor
- complete-window support escapes: `0`
- membership hash: `13eebb7276ee283d`
- finite values, legal paths, and linked-channel closure pass
- steady, dense, boundary, chirp, noise, gain, polarity, and stereo gates pass

## Rejection

- isolated impulse counts: `[31,2,2,29]`; legal transition shoulders violate
  the `2048`-frame far-field return-to-long rule
- mixed control counts: `[0,0,0,64]`; declared transient missed
- perturbation changes: `[0,0,0,0,0,0.125,0.125,0.125,0,0,0,0]`
- direct equivalence failures: `0`
- gate failures: `[0,1,0,0,1,1,0]`
- evidence hash: `8e6e86b6830bfa3e`

The terminal geometry fails. No region, threshold, path, weighting, or detector
change is authorized by this result.

## Next Task

Operator review must retire Rényi automatic selection, authorize a new contract,
or pause the successor lane. No implementation batch is ready.
