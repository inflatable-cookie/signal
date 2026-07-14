# g10.029 Long-Form Musical Gate

Status: operator listening ready
Date: 2026-07-14
Batch: 29.6CK

## Short-Form Decision

The corrected nine-row pack does not promote the weighted predictor.

- best or tied: L002, L013
- competitive: L001, L007
- regressions: L004 softness/end pop, L005 smear, L008 softness, L014 grain
- current Signal and Rubber Band are more consistently safe

The rows are about `0.37` seconds before stretching. They close input identity,
transient, and boundary questions but do not expose sustained texture or
musical continuity well enough.

## Long-Form Pack

Six five-second mono rows cover drums, bass, vocals, pads, and full mix at
`1.5x` or `2.0x`. Candidates are limited to:

- Signal weighted predictor
- current Signal
- Rubber Band R3 `4.0.0`

Every path consumes the same row-specific 44.1 kHz mono 16-bit input. The pack
contains six references and `18` concealed trials.

Evidence:

- input files: `6/6`, exactly `220500` frames
- Rubber Band renders: `6/6`, exactly `330750` or `441000` frames
- structural failures: `[0,0,0,0]`
- holdout reads: `0`
- input hash: `f82238ad4e332c26`
- external hash: `78485bfe53e1a1d9`
- assignment hash: `43b1b12791ced723`
- gain hash: `69b33fe2cc5f77ec`
- notes hash: `605f25c668ff5db9`

## Next Task

Listen at `target/stretch-source-studied-ck-long-form-pack`. Judge musical
continuity, sustained grain/ringing, tonal stability, transient integrity, and
boundaries. If weighted prediction does not show a coherent advantage, reject
this implementation without per-row tuning.
