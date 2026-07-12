# g10.029 Mixed-Phase Calibration Direction

Date: 2026-07-12
Status: distribution audit ready

## Decision

Continue the mixed-phase transient-evidence family through bounded calibration
research. Do not fit thresholds directly to the failed detector controls.

The public SELEBI method reports:

- absolute magnitude threshold `0.01`
- empirical mixed-phase thresholds `0.5/0.75`
- one-dimensional median smoothing with no stated length
- peak prominence `0.1`

Those values are evidence anchors, not Signal defaults. Signal's magnitude
normalization differs, and the smoothing definition is incomplete.

## Boundary

Batch 29.6AV measures normalized-magnitude and mixed-phase distributions only.
A later calibrated-mask contract opens only if event and negative controls show
a stable separating interval. No mask, peak, schedule, phase, stretched audio,
corpus output, dynamic ratio, cache, or routing change is authorized.

Primary source: [Akaishi, Holighaus, and Yatabe, 2026](https://arxiv.org/abs/2602.16421)

## Next Task

Run Batch 29.6AV mixed-phase distribution audit.
