# Concealed Coherent Source Comparison Pack

Date: 2026-07-14
Roadmap: `g10.029`, Batch 29.6DA
Scope: report-only coherent Signal versus pinned Signalsmith listening export

## Decision

Mark the two-way pack ready for concealed operator listening. Keep the
source-studied baseline decision open.

## Pack

Path: `target/stretch-source-studied-da-concealed-pack`

- six five-second source references
- twelve level-matched concealed trials
- ratios: `1.5x` and `2.0x`
- format: `44.1 kHz`, mono, 32-bit float
- source frames: `220500`
- output frames: `330750` or `441000`
- holdout reads: `0`
- structural failures: `[0, 0, 0, 0, 0, 0]`

Files:

- `listening-manifest.tsv`: operator-safe row and path map
- `listening-notes.tsv`: required row-complete findings
- `listening-key.tsv`: concealed identity and gain map; keep closed
- `audio-receipt.tsv`: sample rate, channels, frames, and file hashes

## Frozen Hashes

- audio: `cb135aa644887edb`
- assignment: `64c2874dd6e47521`
- gain: `ffbbba5df08c762c`
- manifest: `fd1255a2fc007590`
- closed key: `f7320382d5bac785`
- notes: `91d68633349f1944`
- metadata receipt: `6d1ba75b59a6ad1f`

The complete exporter repeats exactly. Its prerequisite Batch 29.6CZ objective
confirmation also repeats with zero hard or structural failures.

## Listening Contract

Complete every row without opening the key. Record:

- musical continuity
- transient definition
- grain or atonal ringing
- tonal stability
- start-boundary artifacts
- end-boundary artifacts
- preference and any broad defect

## Closed Lanes

- algorithm changes and parameter sweeps
- stereo and dynamic ratio
- product routing and promotion

## Next Task

Complete all six `listening-notes.tsv` rows, then return the findings for the
source-studied baseline decision.
