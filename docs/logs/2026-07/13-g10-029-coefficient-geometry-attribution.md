# g10.029 Coefficient-Geometry Attribution

Date: 2026-07-13
Batch: 29.6CD
Rule: 30Y

## Scope

Kept the nine development rows, ratios, measurements, event-warped schedule,
ordinary transport, Hann analysis/synthesis, and exact dual normalization
frozen. Compared centered reflected `2048` frames on shared `4096` and native
`2048` FFT grids with start-aligned zero-padded native geometry. Retained
current Signal and Hann/Hann `4096` references. No production, holdout,
listening, detector/schedule, stereo, dynamic-ratio, cache, or routing change.

## Result

Shared-grid zero-padding contributes. The remaining phase/magnitude path owns
the broad defect.

- Shared `4096` to native `2048`, centered reflected: mean timing improves
  `32.194444` frames; static/formant residual improves `0.040495/0.017523`.
- The same grid change raises mean replica ratio by `0.842327`.
- Native centered reflection to start-aligned zero padding worsens mean
  static/formant residual by `0.029572/0.011684`.
- Every candidate still regresses static-spectrum and formant residual in
  `9/9` rows against current Signal.
- Native centered reflection has two integrity failures. The shared-`4096`
  `2048` control has four. Hann/Hann `4096` remains clean.

Native-grid centered reflection is the strongest timbral geometry, but its
replica failure prevents promotion. Boundary reflection helps. Geometry changes
do not close the coefficient-path gap.

## Frozen Evidence

- rows: `9`
- modes: `5`
- renders: `45`
- repeatability pass: `90` renders
- hard failures by mode: `[0,0,4,2,2]`
- changed from current: `[9,9,9,9]`
- FFT-grid regressions, timing/replica/static/formant: `[3,6,0,0]`
- frame-geometry regressions, timing/replica/static/formant: `[5,5,9,7]`
- manifest hash: `55021268ac0cb16f`
- render hash: `d788ea7642e16b09`
- measurement hash: `b56a87e849ff3f5a`
- aggregate hash: `fcd42c867eef4419`
- TSV: `target/stretch-successor-cd-geometry-attribution.tsv`
- TSV SHA-256:
  `77fe8087a61537775f085611a99d769a47c2d6259cf524f9463af8801d691df9`
- holdout reads: `0`
- listening exports: `0`

## Next Task

Execute Batch 29.6CE under Rule 30Z. Freeze one coherent native-grid,
reflection-preserving coefficient path from the accumulated evidence. Do not
render another candidate or reopen factor sweeps, holdout, listening, tuning,
stereo, dynamic ratio, cache, or routing.
