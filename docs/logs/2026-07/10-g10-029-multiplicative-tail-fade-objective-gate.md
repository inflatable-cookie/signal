# g10.029 Multiplicative Tail Fade Objective Gate

Date: 2026-07-10
Status: objective pass; ready for concealed listening

## Purpose

Test whether the sustained-tail thump came from the additive endpoint
correction. Hold the 256-frame span and half-cosine shape constant. Change only
the correction law from an added offset to multiplicative amplitude reduction.

## Control

The report-only control multiplies the final 256 output frames by a half-cosine
gain from one to zero. The first frame in the span remains unchanged. The final
sample is digital silence. No production renderer or cache identity exposes the
control.

## Objective Gate

Evidence is target-local at
`target/stretch-corpus-g10-029-multiplicative-tail-fade-review-v1.tsv`.

- `60/60` rows changed only the final 255 samples
- `60/60` passed integrity
- `60/60` passed transient, tonal-texture, and formant tolerances
- `60/60` passed the combined gate
- `17/17` loud-tail targets improved by at least `3 dB`
- no exterior edge worsened
- worst exterior step improved from `-6.328693` to `-29.129923 dBFS`
- maximum correction was `0.769897819`
- mean peak correction was `0.199423069`
- 43 rows exceeded `0.1` correction; 17 exceeded `0.25`
- maximum endpoint-energy change remained `5.772470 dB`
- maximum peak growth remained `3.574918 dB`
- no silence frames were added

The multiplicative control closes the objective boundary defect but changes
more local amplitude than the additive control. The additive maximum correction
was `0.482575566`; the multiplicative maximum is `0.769897819`. Objective
success cannot establish that the fade is less audible.

## Listening Pack

Target-local path:
`target/stretch-corpus-g10-029-multiplicative-tail-listening-pack-v1`

The six previous worst current endpoints are reused. Each concealed trial
contains current Signal, the rejected additive zero anchor, and the
multiplicative zero fade. WAVs remain mono final-second excerpts with `250 ms`
post-tail silence and one shared per-trial gain.

## Decision

Qualify the multiplicative control for concealed mono listening. Do not promote
it. Production DSP and cache identity remain unchanged. Linked-stereo behavior
and independent stereo listening remain separate blockers.

## Next Task

Complete all six concealed trials. Freeze click/pop, pull/thump, fade,
continuity, and preference notes before opening the key.
