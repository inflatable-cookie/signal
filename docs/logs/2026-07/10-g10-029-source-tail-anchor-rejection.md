# g10.029 Source Tail Anchor Rejection

Date: 2026-07-10
Roadmap: `g10.029`
Scope: bounded fixed-ratio exterior-tail control

## Candidate

The report-only candidate starts from the current OfflineHighQuality render. If
the output's final sample is louder than the source's final sample, it applies a
half-cosine correction over the final 256 frames and lands exactly on the
source endpoint. Otherwise it leaves the output unchanged.

The candidate does not fade to silence, alter the head, change output length,
or enter production routing. It tests one question: is failure to land on the
source endpoint the main cause of the measured loud tails?

## Gate

Promotion required all of these:

- all 60 candidate renders pass `offline-high-quality-v1` integrity
- no transient placement or crest regression beyond `0.25` frames / `0.1 dB`
- no tonal residual, sideband, or spectral-movement regression beyond `0.001`
- no formant-envelope residual or centroid regression beyond `0.001` / `2 Hz`
- no exterior-step worsening
- at least 75% of the 17 current rows above `-20 dBFS` improve by `3 dB`

The last condition requires at least 13 materially improved loud-tail rows.

## Result

The candidate was regression-free but ineffective as the boundary fix.

- `26/60` rows changed; no row changed outside the final 255 samples
- largest correction was `0.488065`
- `60/60` passed integrity
- `60/60` passed transient, tonal-texture, and formant-envelope tolerances
- no exterior edge worsened
- only `5/17` loud-tail targets improved by at least `3 dB`
- worst candidate edge remained `-7.393442 dBFS`

The five material improvements covered two vocal sources and one full-mix
source. The worst pads/sustains row moved only from `-6.328693 dBFS` to the
source endpoint at `-7.393442 dBFS`.

Evidence is target-local at
`target/stretch-corpus-g10-029-tail-anchor-review-v1.tsv`.

## Decision

Reject source-endpoint anchoring as the OfflineHighQuality boundary fix. Keep
the implementation report-only as a control. Production DSP and cache identity
remain unchanged.

The result narrows ownership: many unsafe standalone tail steps are already
present at the source endpoint, so exact source matching and standalone-safe
output are different policies. The next control must target digital silence,
declare its bounded content alteration, and retain the same combined integrity
and quality gates. It must remain report-only until linked-stereo behavior and
listening are available.

## Next Task

Build one bounded zero-tail anchor control and compare it with current output
and the rejected source-anchor control. Require material improvement on the
loud-tail set, explicit changed-frame/correction evidence, and no integrity,
transient, tonal-texture, or formant-envelope regression. Do not promote mono
production behavior while independent stereo review remains open.
