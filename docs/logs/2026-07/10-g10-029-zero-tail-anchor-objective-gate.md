# g10.029 Zero Tail Anchor Objective Gate

Date: 2026-07-10
Roadmap: `g10.029`
Scope: bounded digital-silence tail control

## Candidate

The report-only zero-tail control starts from the current OfflineHighQuality
mono output. It applies a half-cosine correction over the final 256 frames and
lands the final sample exactly on digital silence. It changes no earlier frame,
does not alter output length, and does not enter production routing.

The corpus report emits current, rejected source-anchor, and zero-anchor rows
under one evidence schema. Each row includes changed frames, peak correction,
exterior-step improvement, full-render integrity, transient placement/crest,
tonal texture, and broad formant-envelope evidence.

## Gate

The zero control inherited the source-anchor gate:

- all 60 renders pass `offline-high-quality-v1`
- no transient placement or crest regression beyond `0.25` frames / `0.1 dB`
- no tonal residual, sideband, or spectral-movement regression beyond `0.001`
- no formant-envelope residual or centroid regression beyond `0.001` / `2 Hz`
- no exterior-step worsening
- at least 13 of the 17 current rows above `-20 dBFS` improve by `3 dB`

Objective success is necessary but not sufficient. The altered tail span still
requires listening and linked-stereo evidence.

## Result

The zero control passed the complete objective gate.

- `60/60` rows changed, each inside the final 255 samples
- `17/17` loud-tail targets improved by at least `3 dB`
- worst exterior step improved from `-6.328693` to `-29.129923 dBFS`
- `60/60` passed integrity
- `60/60` passed transient, tonal-texture, and formant-envelope tolerances
- no exterior edge worsened
- maximum correction was `0.482576`
- mean peak correction was `0.075467`
- 17 rows exceeded `0.1` peak correction; five exceeded `0.25`
- maximum endpoint-energy change remained `5.772470 dB`
- maximum peak growth was `3.574918 dB`

The rejected source anchor improved only `5/17` loud tails and left a
`-7.393442 dBFS` worst edge. Targeting digital silence, not source amplitude,
is the direct reason the new control closes the exterior-step metric.

Evidence is target-local at
`target/stretch-corpus-g10-029-zero-tail-anchor-review-v1.tsv`.

## Decision

Qualify the zero-tail control for focused listening. Do not promote it.
Production DSP and cache identity remain unchanged.

The existing transient, tonal, and formant probes do not inspect the final
255-sample correction closely enough to classify its local sound. A correction
as large as `0.482576` over roughly 5.3 ms at 48 kHz may trade the exterior
click for an audible pull or thump. The next evidence must expose the corrected
tail followed by digital silence, level-match the candidates, and include the
worst corrections. Linked-stereo policy remains a separate promotion blocker.

## Next Task

Generate a bounded mono tail-listening pack for the largest corrections and
loudest current tails. Compare current, source-anchor, and zero-anchor tails
with a short post-tail silence pad under concealed assignment. Keep production
unchanged pending operator listening and independent stereo review.
