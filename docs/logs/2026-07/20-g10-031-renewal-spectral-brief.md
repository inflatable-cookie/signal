# g10.031 RenewalSpectral Complete Brief

Date: 2026-07-20
Status: Batch 31.17 complete; isolated candidate ready

## Decision

Freeze one complete neutral `Dream` renderer in
`docs/architecture/offline-creative-renewal-spectral-brief.md`.

`RenewalSpectral` uses one sample-rate-normalized long transform, exact
sample-centred map, magnitude-only mono or mid/side analysis, fresh
counter-addressed phase per frame, and pairwise equal-power frame synthesis.
It has no coherent carrier, magnitude recurrence, transient logic, peak state,
limiter, or post normalization.

## Frozen Boundary

- fixed-ratio `4x` through `16x`, with admission at `4x`, `8x`, and `16x`
- private neutral `Dream` only
- exact target length, bounded `32 MiB` state, deterministic seed, offline only
- linked `space` through one mid/side phase field
- `motion`, `detail`, other characters, routing, cache, and product APIs absent
- hard structural gate, comparator-calibrated crest and synthetic gates,
  concealed long-form mono review, then independent stereo review
- whole-candidate deletion on the first failed gate

## Scope

Documentation only. No DSP, module declaration, candidate harness, fixture,
report mode, comparator capture, public API, cache, route, Loophole, or Chorus
surface changed. The unrelated binaural and reverb edits remain untouched.

## Validation

- `git diff --check` passed
- `effigy qa:docs` passed
- `effigy qa:northstar` passed
- `effigy health` passed
- `effigy validate` passed

`effigy doctor` retains the known god-file and attention-marker findings. This
batch does not expand into them.

## Next Task

Run Batch 31.18 only. Implement the brief once in
`signal-candidate-31-18` on `candidate/g10-031-renewal-spectral`. Stop at the
first failed gate and keep candidate code off `main` until complete admission.
