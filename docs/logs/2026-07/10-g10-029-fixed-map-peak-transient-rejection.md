# g10.029 Fixed-Map Peak Transient Rejection

Date: 2026-07-10
Status: rejected
Contract: `082`
Batch: `29.6C`

## Candidate

The report-only candidate keeps the current `2048/512` grid, constant global
synthesis hop, overlap-add, magnitude spectrum, crop, and identity bypass. One
companion FFT uses the Hann window multiplied by a centred time ramp. Frozen
onset events guard peak-local group-delay analysis. Magnitude minima bound each
peak region. Candidate regions collect until their energy position crosses the
window-derived centre threshold; selected bins then copy analysis phase after
ordinary identity locking.

Production OfflineHighQuality, cache identity, pitch and dynamic routing,
RealtimePreview, adaptive resolution, and linked stereo did not change.

## Mechanism Evidence

- `60` rendered rows
- `2370` guarded events
- `984` unmatched guarded events
- `249687` candidate peak regions
- `1386` centre-threshold crossings
- `492156` reinitialized bins
- `0` uncovered output frames
- `60/60` integrity, no-added-silence, and peak-growth passes

The window-derived centre threshold was `152.257001` sample frames. No
threshold, sensitivity, guard, or reset-scope sweep was run.

## Frozen Gate

- anchored `L001` improvement: `0.040942 dB`; required at least `3 dB`
- candidate worst crest: `5.614542 dB`; limit `5.655483 dB`
- measurable event-placement mean delta: `+16.851522` frames; limit `+1`
- combined gate: `12/60`; required `60/60`
- transient regression-free rows: `20/60`
- tonal regression-free rows: `45/60`
- formant regression-free rows: `44/60`
- boundary regression-free rows: `32/60`
- tonal residual regressions: `21/60`
- unsupported-bin-mass regressions: `24/60`

The candidate met only the worst-crest, integrity, coverage, added-silence, and
peak-growth constraints. It failed the defining anchored crest, placement,
spectrum, formant, boundary, and combined gates.

## Decision

Reject fixed-map peak-selective phase reinitialization on the current grid. Do
not tune the window-derived threshold, `1.5` sensitivity, event guards, or reset
scope. Do not broaden the reset to whole frames or change the time map to rescue
this mechanism.

Adaptive resolution and linked stereo remain closed. Explicit
transient/residual separation may reopen only after contract `082` defines its
perfect-reconstruction analysis, mask continuity, component processing,
recombination, evidence, and stop conditions.

## Evidence Artifact

Generated local report:
`target/stretch-corpus-g10-029-fixed-map-peak-transient-v1.txt`.

## Next Task

Reassess contract `082` for explicit transient/residual separation. Freeze the
component boundary before implementation.
