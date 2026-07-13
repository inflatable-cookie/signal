# g10.029 Window-Kernel Attribution

Date: 2026-07-13
Batch: 29.6CC
Rule: 30X

## Scope

Kept the nine development rows, ratios, measurements, fixed `4096` geometry,
event-warped lattice, ordinary transport, and exact pairwise dual normalization
frozen. Crossed square-root-Hann and Hann analysis and synthesis kernels. No
production-path, holdout, listening, detector/schedule, stereo, dynamic-ratio,
cache, or routing change.

## Result

Hann helps on both sides but does not own the defect.

- Hann analysis reduces mean static residual by `0.003732` with root-Hann
  synthesis and `0.003815` with Hann synthesis.
- Hann synthesis reduces mean static residual by `0.005078` with root-Hann
  analysis and `0.005161` with Hann analysis.
- Hann/Hann cuts mean timing loss from `82.027778` to `41.333333` frames.
- Hann/Hann reduces mean static/formant residual deltas from
  `0.087938/0.049590` to `0.079045/0.046138`.
- All four pairs still regress static-spectrum and formant residual in `9/9`
  rows against current Signal.

Hard failures by mode are `[0,0,1,0,0]`; one root-analysis/Hann-synthesis row
breaches endpoint energy. Hann/Hann retains clean integrity. The remaining
owner is coefficient geometry: shared-grid zero-padding and centered/reflected
frames versus current Signal's native-grid start-aligned padded frames.

## Frozen Evidence

- rows: `9`
- modes: `5`
- renders: `45`
- repeatability pass: `90` renders
- changed from current: `[9,9,9,9]`
- manifest hash: `7d7886402f662bc7`
- render hash: `76298cafc83779af`
- measurement hash: `a2173e14c6eb7535`
- aggregate hash: `1f7a65480074cf7b`
- TSV: `target/stretch-successor-cc-window-attribution.tsv`
- TSV SHA-256:
  `7c2a89d3d13ae3988742fc4a549c29f944d216de95951b3101da7488230fef18`
- holdout reads: `0`
- listening exports: `0`

## Next Task

Execute Batch 29.6CD under Rule 30Y. On Hann/Hann `2048`, separate shared-FFT
zero-padding from centered/reflected versus start-aligned padded frame geometry.
Keep detector/schedule policy, holdout, listening, tuning, stereo, dynamic
ratio, cache, and routing closed.
