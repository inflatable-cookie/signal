# g10.029 Transient-Evidence Rejection

Date: 2026-07-12
Status: operator review required

## Rejection

- false-positive controls failed: `7/7`
- impulse/boundary event failures: `3`
- dense-event failures: `1`
- mixed-control failures: `1`
- equivalence failures: `1`
- perturbation failures: `3`
- structural failures: `0`
- gate failures: `[7,3,1,1,1,3,0]`

Isolated and dense impulse occupancy changes reach `0.6262388968`; boundary
change reaches `0.4192063519`. Isolated and dense peak counts change. Boundary
equal-energy stereo occupancy differs by `0.0014662757`. Evidence hash
`6f6733bda80316a9` repeats.

The analytic midpoint detector is not viable. No threshold, smoothing,
prominence, detector vote, schedule, or audio change opens.

## Next Task

Operator review must choose calibrated mixed-phase research, a different
evidence family, or pause. No implementation batch is ready.
