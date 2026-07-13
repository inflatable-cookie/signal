# g10.029 Ordinary Shared-Mechanism Factor Attribution

Date: 2026-07-13
Batch: 29.6CB
Rule: 30W

## Scope

Kept the nine development rows, ratios, measurements, fixed `4096` geometry,
window kernel, detector, and event schedule frozen. Crossed event-warped and
global-linear output placement, instantaneous-frequency transport and analysis
phase passthrough, and exact diagonal-dual and normalized analysis-partition
overlap. Current Signal remained the ninth reference. No production-path,
holdout, listening, stereo, dynamic-ratio, cache, or routing change.

## Result

The factored mechanisms are not the primary owner of the broad timbral defect.

- Lattice: global-linear placement changes transported diagonal-dual mean
  static residual by `+0.000538` and formant residual by `+0.000676`.
- Phase: analysis passthrough worsens static residual in all nine rows on both
  exact-dual lattices. Transport is beneficial.
- Overlap: analysis-partition overlap worsens static and formant residual in all
  nine rows on both transported lattices. The exact diagonal dual is beneficial.
- Common path: all eight factor modes still regress static-spectrum and formant
  residual in `9/9` rows against current Signal.

Hard failures by mode are `[0,0,0,5,5,0,0,4,5]`. They occur only in phase
passthrough modes; both transported overlap variants retain clean integrity on
both lattices. The next owner is the shared windowed coefficient representation,
starting with square-root-Hann versus Hann analysis and synthesis kernels.

## Frozen Evidence

- rows: `9`
- modes: `9`
- renders: `81`
- repeatability pass: `162` renders
- changed from current: `[9,9,9,9,9,9,9,9]`
- manifest hash: `63d64c56e0e402bb`
- render hash: `671bfeb418981df8`
- measurement hash: `aaf112446dc0f0a8`
- aggregate hash: `3c9f3f66ae65d5c1`
- TSV: `target/stretch-successor-cb-mechanism-attribution.tsv`
- TSV SHA-256:
  `856e4a5484cba6175034150e707a143d0a672d506a698b3bff94876b01244c32`
- holdout reads: `0`
- listening exports: `0`

## Next Task

Execute Batch 29.6CC under Rule 30X. Cross square-root-Hann and Hann analysis
and synthesis kernels with exact pairwise dual normalization on fixed `4096`.
Keep resolution, detector/schedule policy, holdout, listening, tuning, stereo,
dynamic ratio, cache, and routing closed.
