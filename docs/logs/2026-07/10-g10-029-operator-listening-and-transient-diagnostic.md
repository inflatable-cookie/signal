# g10.029 Operator Listening And Transient Diagnostic

Status: partial quality evidence
Date: 2026-07-10

## Operator Findings

The operator reviewed all 15 blind pairs. The same pattern repeated strongly
enough that row-level notes were not filled:

- transients were mostly similar
- Signal showed occasional visible pop spikes; `L001-B` was the clearest case
- at compression ratios, Signal softened less-pronounced attacks slightly
- at the longest stretches, Signal sounded slightly grainier with subtle
  atonal ringing
- Rubber Band sounded slightly more musical and higher-resolution at longer
  stretches
- transient placement felt slightly uncertain, but the listening pass could
  not establish drift
- stereo was not assessed because the operator has hearing in one ear; a
  second listener remains required
- no distinct boundary or formant failure was isolated from the reported
  transient and tonal behavior

The key was revealed after the report. `L001-B` is Signal Default at `0.75x`.

## Objective Follow-Up

Added event-level transient evidence distinct from output-length drift:

- coarse spectral candidates refined to sample-frame energy-rise positions
- unique source/output event matching
- signed, mean-absolute, and maximum timing offsets
- level-invariant local transient crest growth and worst-event locations
- independent-bin draft control alongside Signal Default and Rubber Band

Target-local evidence:
`target/stretch-corpus-g10-029-transient-detail-v1.tsv`.

Across 47 of 60 rows with matched attacks:

- Signal mean absolute event offset: `102.826` frames
- Rubber Band mean absolute event offset: `101.845` frames
- Signal mean maximum crest growth: `1.328 dB`
- Rubber Band mean maximum crest growth: `1.233 dB`

This does not confirm a corpus-wide Signal timing defect. The timing search is
bounded to `±256` frames and some maxima reach that boundary, so these values
are diagnostic evidence, not an acceptance threshold.

`L001` at `0.75x` is an objective outlier:

- Signal Default maximum transient crest growth: `5.655 dB`
- independent-bin draft control: `3.647 dB`
- Rubber Band: `1.832 dB`
- Signal worst source/output event: frame `180354` / `135290`

At `0.75x`, Signal Default uses identity phase locking without transient phase
reset. The result rules out reset triggering as the direct cause. Identity
locking increases the worst crest relative to the independent-bin control,
but the draft still exceeds Rubber Band. The next probe must isolate peak-region
locking and overlap-add reconstruction at the same source event before changing
production synthesis.

## Gate State

- operator transient and tonal findings: recorded
- aggregate five-family listening pass: recorded
- row-complete TSV validation: incomplete
- stereo assessment: blocked on an independent listener
- product-quality promotion: closed
- Batch 29.4 structural hybrid work: not started

## Next Task

Add a same-event `L001` control probe for identity locking, independent bins,
and bounded peak-region variants. Reject any candidate that moves the crest
problem or worsens the 60-row pack. Keep stereo and product promotion closed.
