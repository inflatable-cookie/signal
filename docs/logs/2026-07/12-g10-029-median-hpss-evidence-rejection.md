# g10.029 Median-HPSS Evidence Rejection

Date: 2026-07-12
Status: operator review required

## Outcome

- false-positive controls: `7/7`
- impulse/boundary event failures: `3`
- dense-event failures: `1`
- mixed-control failures: `1`
- equivalence failures: `0`
- perturbation failures: `3`
- structural failures: `0`
- gate failures: `[7,3,1,1,0,3,0]`
- evidence hash: `b4812090f561ea14`

The isolated impulse peak is `896` frames late. Boundary offsets are `7168` and
`1023`; dense offsets are `896` and `1152`; mixed-event offset is `3968`.
Isolated and dense perturbations change occupancy by more than `0.60`; boundary
change is `0.0524528981` and its peak moves `56` anchors.

Gain, polarity, hard pan, channel swap, and equal-energy stereo pass below
`1.34e-15` with exact peak indices. Median HPSS is stable but does not make
percussive occupancy local maxima selective. No component audio, schedule, or
synthesis changed.

## Next Task

Operator review must choose a different selector abstraction or pause.
