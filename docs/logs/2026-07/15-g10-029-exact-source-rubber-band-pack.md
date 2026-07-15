# Exact-Source Rubber Band Pack

Date: 2026-07-15
Roadmap: `g10.029`, Batch 29.6DB
Scope: report-only coherent Signal versus Rubber Band R3 evidence

## Decision

Open the six-row concealed mono comparison. Do not select either engine from
objective metrics.

## Evidence

- inputs: six exact `44.1 kHz` mono 16-bit five-second sources
- ratios: `1.5x` and `2.0x`
- Rubber Band: R3 `4.0.0`, `-q -3`
- structural failures: `[0, 0, 0, 0, 0, 0, 0, 0, 0]`
- hard-integrity failures: coherent Signal `0/6`, Rubber Band `0/6`
- exact repeats: both engines
- maximum packed-candidate RMS delta: `1.31e-9 dB`
- coherent-Signal regressions against Rubber Band for timing, replicas, static
  residual, and boundary growth: `[2, 5, 0, 6]`

The objective direction is mixed. Coherent Signal has lower static residual on
all six rows and lower timing error on four. Rubber Band has lower replica
ratio on five and lower boundary growth on all six. Listening must resolve
whether those differences are audible and musically material.

## Frozen Hashes

- input: `8ede75dbae2254b2`
- coherent audio: `7ec654eb414041ce`
- Rubber Band audio: `3ee61b19c9498523`
- measurements: `1c4b6398bf49d9bf`
- objective report: `eb1144f437a6ae65`
- render receipt: `4338e41ab85fe116`
- packed audio: `bd7dec22a565a32f`
- assignment: `c9724071b3aa2ded`
- gain: `d2b29e930726e10f`
- manifest: `fd1255a2fc007590`
- closed key: `14d5bbab2061b8fd`
- notes: `91d68633349f1944`
- audio receipt: `1f80e9da6c011beb`

## Pack

Path: `target/stretch-source-studied-db-concealed-pack`

Keep `listening-key.tsv` closed. For every row, report continuity, transient
definition, grain or ringing, tonal stability, start and end boundaries,
preference, and any broad defect.

## Next Task

Complete all six concealed rows, then decode the result and decide whether the
coherent baseline is competitive with Rubber Band or which source-studied
mechanism remains necessary. Keep stereo, dynamic ratio, routing, and promotion
closed.
