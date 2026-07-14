# g10.029 Exact-Excerpt Comparator Pack

Status: operator listening ready
Date: 2026-07-14
Batch: 29.6CJ

## Implemented

One ignored release-test runner owns the complete local comparison operation:

- convert each frozen source region to a row-specific mono 16-bit WAV
- freeze 44.1 kHz, one channel, `16384` frames, and file hash
- invoke Rubber Band R3 `4.0.0`
- invoke pinned Signalsmith Stretch `1.3.2`
- reject output sample-rate, channel-count, target-length, or finiteness drift
- render weighted predictor and current Signal from the decoded exact input
- level-match and conceal all four paths

No external library enters Signal. The runner uses `RUBBERBAND_BIN` when set,
otherwise `rubberband`, and requires `SIGNALSMITH_STRETCH_BIN`.

## Evidence

- inputs: `9/9`, exact `16384` frames
- external renders: `18/18`, exact `12288` or `20480` frames
- concealed pack: nine references plus `36` trials
- structural failures: `[0,0,0,0]`
- holdout reads: `0`
- input hash: `69887b15e8420fd7`
- external hash: `9547b0d5e924d8fa`
- assignment hash: `5e79eb98f2fbdc78`
- gain hash: `2f1894d7c22b23de`
- notes hash: `2e09fb7ce672ec30`

The superseded five-way pack and full-source Signalsmith render directory were
removed. The broad source benchmark remains because it supplies the frozen
source regions.

## Artifacts

- exact inputs and engine receipts:
  `target/stretch-source-studied-cj-external`
- concealed pack:
  `target/stretch-source-studied-cj-development-pack`

## Next Task

Complete the nine-row four-way listen. Compare transient integrity, tonal
stability, grain/ringing, and boundaries. Keep the key and comparator receipt
closed until all rows are assessed. Then decide the weighted predictor's
remaining gap without reopening frequency partitioning or parameter search.
