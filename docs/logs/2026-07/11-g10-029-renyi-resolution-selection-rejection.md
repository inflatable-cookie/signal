# g10.029 Rényi Resolution-Selection Rejection

Date: 2026-07-11
Status: rejected

## Passing Evidence

Silence and steady tonal controls stay entirely at `4096`. Dense and boundary
impulses pass. Stationary noise avoids `512`. Gain, polarity, hard-pan, channel
swap, equal-energy stereo, path legality, finiteness, and repeat gates pass.
Maximum perturbed-path change is `0.015625` against the `0.05` cap.

## Rejection

- isolated impulse counts: `[36,4,8,16]`; short selection persists beyond the
  frozen event neighborhood
- linear chirp counts: `[64,0,0,0]`; no adaptive resolution change
- mixed tonal/transient counts: `[0,0,0,64]`; declared transient missed
- gate failures: `[0,1,0,0,2,0,0]`
- evidence hash: `5568f0a38f679a40`

The fixed comparison region plausibly owns broad impulse contamination, while
whole-band tonal energy plausibly owns the mixed miss. These are attribution
hypotheses, not authorization to alter region size or frequency weighting.

## Next Task

Freeze Batch 29.6AL selector-failure attribution. Do not change the selector or
implement phase or stretched synthesis.
