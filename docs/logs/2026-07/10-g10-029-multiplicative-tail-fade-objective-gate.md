# g10.029 Multiplicative Tail Fade Objective Gate

Date: 2026-07-10
Status: listening complete; unconditional multiplicative fade rejected

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

## Operator Result

Notes were frozen for all six trials before candidate identity was revealed.

- `T001`: multiplicative clean; additive low thump; current high-end click
- `T002`: same result as `T001`
- `T003`: multiplicative clean; current clicked; additive clicked subtly
- `T004`: multiplicative low thump; additive and current clean
- `T005`: same material split as `T004`
- `T006`: all three similar and reasonably clean

The multiplicative control wins the two pad trials and the drum trial. It loses
both decisive full-mix trials. The additive control shows the inverse sustained
material failure. No fixed 256-frame endpoint envelope is universally safe.

## Decision

Reject unconditional promotion of the multiplicative fade. Keep both envelope
controls report-only. Production DSP and cache identity remain unchanged.

Do not add another fixed fade shape from these six cases. First test whether
tail-local measurements separate the three multiplicative wins from its two
clear losses. Case-family labels are evidence metadata, not a production
selector. Linked-stereo behavior and independent stereo listening remain
separate blockers for any later adaptive policy.

## Next Task

Measure tail-local DC offset, low-band energy share, spectral centroid, short
spectral movement, zero-crossing distance, and correction energy for the six
labeled trials. Define a deterministic content-derived selector only if those
features separate the wins from the losses without case-family labels.
