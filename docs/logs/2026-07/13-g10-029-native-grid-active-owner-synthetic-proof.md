# g10.029 Native-Grid Active-Owner Synthetic Proof

Date: 2026-07-13
Batch: 29.6CF
Rule: 30AA

## Result

Rejected before real-source rendering.

The report-only renderer now selects an FFT per adaptive frame, uses centered
reflected Hann/Hann coefficients, preserves native magnitudes, projects active
physical-frequency owners onto native bins, retains native within-region phase
offsets, and applies the exact diagonal dual. Exact transient anchors and the
conflicted-bridge owner remain part of the same path.

## Passing Evidence

- identity maximum peak error: `7.967289e-16`
- active resolution transitions matched: `300/300`
- expected anchors detected and attached: `24/24`
- maximum event errors: `[0,0,0]`
- owner births/matches/retirements: `4784/45608/4780`
- native region assignments: `867076`
- mid- and high-tone rows pass
- dense one-to-one placement and replica protection pass
- coverage, boundary, symmetry, residue, finiteness, and silence pass
- combined regressions: `0`

## Stop

Only the stretched `55 Hz` rows fail the unchanged `1e-6` rendered angular-
frequency limit:

- `0.75x`: `3.695086e-5`
- `1.5x`: `1.023948e-5`
- `2.0x`: `1.597330e-6`

The fixed analytic tracker is not the failing owner. Maximum tracked-owner
frequency error is `1.263528e-7`. Ownership continuity also passes every active
resolution transition. The earliest unresolved boundary is native owner-bin
and phase-region projection into per-frame inverse output and exact-dual
accumulation.

- mechanism failures: `[0,0,3,0,0,0,0,0]`
- complete synthetic rows: `48`
- complete hard failures: `3`
- mechanism hash: `19c5548baf4a10c8`
- quality hash: `2410e33944214b72`
- holdout reads: `0`
- listening exports: `0`
- real-source renders: `0`

## Next Task

Execute Batch 29.6CG under Rule 30AB. Freeze the failed low-tone rows and trace
active phase through native owner-bin/region projection, inverse output, and
exact-dual accumulation. Do not change thresholds, windows, detector, schedule,
or gate.
