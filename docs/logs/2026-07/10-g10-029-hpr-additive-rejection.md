# g10.029 H/R/P Additive Mono Rejection

Date: 2026-07-10
Status: rejected
Contract: `082`
Batch: `29.6E`

## Candidate

The report-only candidate decomposes each source through the passing Batch
29.6D separator, then applies one fixed ratio and target length to all three
components:

- harmonic: long-window identity-locked phase vocoder
- residual: current `2048/512` OfflineHighQuality kernel
- percussive: short-window normalized OLA

The three outputs are added sample-aligned. No branch switch, crossfade, delay
repair, waveform search, component gain correction, local time map, or
production route enters the candidate.

## Mechanism Evidence

- `60` rendered rows
- `60/60` exact component lengths
- `60/60` monotonic percussive synthesis maps
- `60/60` percussive overlap-add coverage
- `60/60` finite output
- `60/60` no hidden component gain
- `60/60` no added silence
- `60/60` peak-growth limit
- `51/60` endpoint-integrity and complete-integrity passes

No mask, separation-factor, processor-geometry, gain, or timing sweep was run.

## Frozen Gate

- anchored `L001` improvement: `3.375261 dB`; required at least `3 dB`
- candidate worst crest: `4.083747 dB`; limit `5.655483 dB`
- measurable event-placement mean delta: `+23.411637` frames across `42` rows;
  limit `+1` frame
- worst event-placement delta: `+178.5` frames
- mean fast-movement delta: `-0.007164100` at `1.25x` and `-0.009516000`
  at `1.5x`; both moved in the required direction
- mean static-residual delta: `+0.041365600` at `1.25x` and `+0.018794000`
  at `1.5x`; both failed
- mean unsupported-bin delta: `+0.001427950` at `1.25x` and `+0.001066300`
  at `1.5x`; both failed
- post-attack replica gate: `26/48`; worst delta `+0.708127`
- transient regression-free rows: `16/60`
- tonal regression-free rows: `4/60`
- formant regression-free rows: `1/60`
- boundary regression-free rows: `29/60`
- combined gate: `0/60`; required `60/60`

The component split reached the defining L001 crest target and reduced fast
spectral movement, but specialized processing moved other events, changed
endpoint energy, duplicated post-attack peaks, and damaged static spectral and
formant structure. The failure is broad, not a single threshold miss.

## Decision

Reject the additive H/R/P fixed-ratio mono mechanism without tuning. Do not
change binary masks, `beta_h=2`, `beta_p=2`, component gains, processor windows,
or component timelines to rescue it. Batch 29.7 linked stereo remains closed.

Production OfflineHighQuality, cache identity, pitch/dynamic routing,
RealtimePreview, product receipts, and product integration remain unchanged.

## Evidence Artifact

Generated local report:
`target/stretch-corpus-g10-029-hpr-additive-v1.tsv`.

## Next Task

Stop implementation for synthesis-policy reassessment. Decide whether a
materially different clean-room synthesis family warrants research before
opening another roadmap card.
