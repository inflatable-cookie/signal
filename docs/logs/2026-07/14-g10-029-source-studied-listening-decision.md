# g10.029 Source-Studied Listening Decision

Status: complete
Date: 2026-07-14
Roadmap: `g10.029`, Batches 29.6CH and 29.6CI

## Decision

- reject the frequency-partitioned architecture
- retain the fixed-grid weighted predictor as the only successor research path
- make no Rubber Band or Signalsmith ranking from this pack
- open one exact-input comparator confirmation; no tuning or parameter search

## Listening Evidence

The frequency-partitioned path produced repeated architecture-level defects:

- stutter and a post-transient dip on L001
- pre-ringing and grain on L002
- non-zero start and a pop on L004
- blur and a doubled snare transient on L005
- softened or muted attacks on L008 and L013
- inconsistent end boundaries despite tight results on some rows

The weighted predictor was cleanest or competitive on L001, L002, L005, L007,
L013, and L014. It also retained smaller but repeatable smear, grain, transient-
shape, and end-pop defects. This is enough to select a research direction, not
enough for production promotion.

Current Signal remained strong and won or tied several rows. The operator heard
Rubber Band and Signalsmith as inconsistent, sometimes muted, tonally altered,
or boundary-damaged. Those external impressions are not admissible comparator
evidence because their inputs differed.

## Comparator Integrity Failure

Signal candidates consumed the first `16384` frames after mono downmix. The
external controls consumed the original `220500`-frame stereo files:

- `0.75x` external renders contain `165375` frames
- `1.25x` external renders contain `275625` frames
- the pack truncated those renders to `12288` or `20480` frames

The compared region therefore came from different source boundaries, channel
contracts, and algorithm state. Full-source output truncation is not equivalent
to rendering the isolated excerpt. Internal Signal-to-Signal evidence remains
valid because those candidates shared one input.

## Next Task

Batch 29.6CJ exports hash-frozen `16384`-frame mono inputs, renders both external
controls from those exact files, and creates one four-way unchanged-row pack:
weighted predictor, current Signal, Rubber Band R3, and Signalsmith Stretch.
Frequency partitioning, holdout, stereo, dynamic ratio, cache, and production
routing remain closed.
