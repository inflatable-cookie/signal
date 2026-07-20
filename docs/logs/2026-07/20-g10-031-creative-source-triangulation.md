# g10.031 Creative Source Triangulation

Date: 2026-07-20
Status: Batch 31.16 complete; docs-only brief ready

## Decision

Select `RenewalSpectral` as one materially different, source-backed neutral
`Dream` family. Run a complete docs-only brief next. Do not implement it in
this batch.

## Evidence

Pinned whole render paths:

- PaulXStretch `v1.6.0` at
  `8ec191fdd7203354c79391cbc04c9fd83fa30ea0`
- CDP `CDP8.0` at `456ffe0687c8d8206f8bc4e22273587db4c0ee0a`
- Potenza at `ddb44a8f949b3f49320932e1d2e997b3a02149bb`

Neutral PaulXStretch uses long-window magnitude analysis, deliberate input
phase loss, new stochastic phase per output frame, and frame crossfade. Its
retained default disables onset handling and optional spectral processors.
Signal's rejected spectral briefs instead added instantaneous-frequency
recurrence, correlated or continuous excitation, magnitude evolution, and a
different overlap topology. They remain rejected but do not test this source
path.

CDP owns a separate vocoder-like path through amplitude/frequency-frame
interpolation and lower-energy frequency decoherence. Potenza owns a separate
two-grain cyclic path. Do not force all three characters into one recurrence.

## Boundary

Changed documentation only. No DSP, candidate module, harness, fixture, report
mode, comparator audio, public API, cache, routing, Loophole, or Chorus surface
changed. The three unrelated binaural/reverb edits remain untouched.

`RenewalSpectral` owns neutral `Dream` only. `Spectral`, `Rough`, `Cyclic`,
`Cloud`, routing, dynamic ratio, cache, product exposure, and the transparent
successor stay closed or paused.

## Validation

- `git diff --check` passed
- `effigy qa:docs` passed
- `effigy qa:northstar` passed
- `effigy health` passed
- `effigy validate` passed

`effigy doctor` retains the known god-file and attention-marker findings. This
batch did not expand into them.

## Next Task

Run Batch 31.17 only. Freeze one complete docs-only `RenewalSpectral` brief.
Stop before candidate DSP.
