# Concealed Level-Match Correction

Date: 2026-07-15
Roadmap: `g10.029`, Batch 29.6DA
Scope: report-only comparison-pack repair

## Finding

Operator listening found `M002-B` significantly louder than `M002-A`. The
exporter chose the minimum raw RMS, then peak-limited each candidate
independently. A high-crest candidate could not reach the common target, so the
written pair was not level-matched.

## Correction

Choose the minimum RMS that every source and candidate can reach under the
`0.95` peak ceiling. Measure RMS and peak from each written float WAV. Fail the
pack when a concealed pair differs by more than `1e-5 dB` RMS.

Corrected evidence:

- structural failures: `[0, 0, 0, 0, 0, 0, 0]`
- maximum candidate RMS delta: `2.44e-9 dB`
- audio: `760577241605fb24`
- assignment: `64c2874dd6e47521`
- gain: `7bba88c9c701bf1c`
- manifest: `fd1255a2fc007590`
- closed key: `bb1974bba5a2a8b0`
- notes: `91d68633349f1944`
- metadata receipt: `de417d1f00e55f88`

## Listening State

The correction changes `M002` by about `4.14 dB` and `M006` by about `0.49
dB`; exclude their first judgments. `M001`, `M003`, `M004`, and `M005` move by
at most `0.05 dB`, so their completed findings remain valid.

Valid findings so far:

- `M001`: no material difference
- `M003`: one candidate may be slightly less grainy
- `M004`: no material difference
- `M005`: almost identical

## Next Task

Completed. Corrected `M002` and `M006` are audible ties. The complete decision
is recorded in `15-g10-029-coherent-source-baseline-decision.md`.
