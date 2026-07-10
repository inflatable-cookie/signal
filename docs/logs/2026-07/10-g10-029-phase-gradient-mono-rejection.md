# g10.029 Phase-Gradient Mono Rejection

Date: 2026-07-10
Status: rejected
Contract: `082`
Batch: `29.6G`

## Candidate

The report-only candidate applies the passing Batch 29.6F whole-band
phase-gradient kernel to the unchanged 20-source, three-ratio mono corpus. It
uses the frozen `4092` Hann / `8192` FFT geometry, ratio-derived analysis hop,
fixed `1024` synthesis hop, centered derivatives, `1e-6` tolerance, and stable
heap priority. No parameter search, source separation, local time map, onset
detector, phase reset, or product route enters the run.

## Mechanism Evidence

- `60/60` exact significant-bin assignment with zero missing or duplicates
- `60/60` finite derivatives and output
- `60/60` conjugate symmetry and overlap-add coverage
- maximum heap high-water `4100/8194`
- `60/60` exact target length, zero added silence, and peak-growth limit
- `57/60` endpoint-integrity and complete-integrity passes

## Frozen Gate

- anchored `L001` improvement: `1.667930 dB`; required at least `3 dB`
- candidate worst crest: `4.103372 dB`; limit `5.655483 dB`
- measurable event-placement mean delta: `+16.738760` frames across `47` rows;
  limit `+1` frame
- worst event-placement delta: `+137` frames
- mean fast-movement delta: `-0.003056500` at `1.25x` and `-0.002028650` at
  `1.5x`; both pass
- mean static-residual delta: `-0.034376250` at `1.25x` and `-0.039958950` at
  `1.5x`; both pass
- mean unsupported-bin delta: `-0.001094150` at `1.25x` and `-0.001203250` at
  `1.5x`; both pass
- post-attack replica gate: `28/48`; worst delta `+0.675459`
- transient regression-free rows: `18/60`
- tonal regression-free rows: `55/60`
- formant regression-free rows: `10/60`
- boundary regression-free rows: `53/60`
- combined gate: `3/60`; required `60/60`

## Comparator Evidence

Every candidate row includes aligned candidate-to-Rubber-Band evidence and the
external source-relative transient, tonal, formant, boundary, and integrity
fields. Mean aligned correlation improved from `0.327354900` for the current
kernel to `0.367353969` for the candidate. Mean aligned RMS error improved from
`0.187637615` to `0.166064781`.

The result is materially better than the additive H/R/P candidate in tonal,
formant, boundary, integrity, combined, and comparator evidence. It still does
not meet the complete gate. The remaining failure matches the operator report:
attacks are softened or misplaced, post-attack energy can replicate, and
spectral-envelope shape is not preserved consistently.

## Decision

Reject the fixed-resolution whole-band phase-gradient mono candidate without
tuning. Do not change window/FFT geometry, tolerance, derivative policy, heap
priority, or crop policy to rescue it. Do not open linked stereo.

Return to research with a narrower target: preserve the phase-gradient core's
tonal and comparator gains while adding a materially different clean-room
attack-placement and shape-preservation mechanism. Do not reopen component
synthesis, branch crossfades, or local time compensation.

## Evidence Artifact

Generated local report:
`target/stretch-corpus-g10-029-phase-gradient-v1.tsv`.
