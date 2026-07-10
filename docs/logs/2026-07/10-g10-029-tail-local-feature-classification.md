# g10.029 Tail-Local Feature Classification

Date: 2026-07-10
Status: cross-source concealed validation ready

## Purpose

Test whether content-derived tail measurements separate the multiplicative
fade's three concealed wins from its two clear losses. Stop fixed-envelope work
if no separation exists.

## Measurement

The report-only diagnostic measures the current render before either correction.
It uses one final 2048-frame window and reports:

- absolute DC offset divided by RMS
- sub-250 Hz spectral-energy share
- magnitude-weighted spectral centroid
- normalized spectral movement between the final two 1024-frame windows
- distance from the endpoint to the last zero crossing
- additive correction energy divided by current tail energy
- multiplicative correction energy divided by current tail energy

Evidence is target-local at:

- `target/stretch-corpus-g10-029-tail-classifier-review-v1.tsv`
- `target/stretch-corpus-g10-029-tail-classifier-labeled-v1.tsv`

## Result

Only spectral centroid cleanly separates every decisive label.

- multiplicative wins: `662.676157` to `1450.409813 Hz`
- multiplicative losses: `2422.601943` to `2441.410312 Hz`
- neutral trial: `2485.733931 Hz`

DC ratio, low-band share, short spectral movement, zero-crossing distance, and
both correction-energy ratios overlap between wins and losses. They do not
support a clean rule on this evidence.

A provisional `< 2000 Hz` spectral-centroid rule separates all five decisive
trials and leaves the neutral trial on the unchanged side. It uses no case-family
label or endpoint-amplitude threshold.

## Limit

The six trials contain only three unique source excerpts: one pad, one drum, and
one full mix. The separator may encode those source identities rather than a
general tail property. It is not sufficient to enable adaptive DSP.

## Decision

Keep the selector report-only and unimplemented. Production DSP and cache
identity remain unchanged. Fixed-envelope experimentation stays closed.

The next evidence must use different source excerpts on both sides of the 2 kHz
threshold, conceal candidate identity, and avoid repeated-source labels. Failure
to reproduce the preference split closes tail-envelope work.

## Cross-Source Validation Pack

Target-local path:
`target/stretch-corpus-g10-029-tail-classifier-validation-pack-v1`

The exporter measured all 60 broad-manifest rows, excluded the three labeled
source excerpts, ranked current endpoint jumps, and selected:

- three distinct sources below `2000 Hz`, spanning `939.215402` to
  `1973.789903 Hz`
- three distinct sources at or above `2000 Hz`, spanning `2222.696644` to
  `3652.827333 Hz`
- six distinct sources total; one selected ratio per source
- current exterior steps from `-14.123834` to `-22.034705 dBFS`

The sealed key contains band membership, centroid, and candidate identity. The
notes manifest exposes only A/B/C. Each trial compares current, additive zero,
and multiplicative zero with the existing shared-gain, mono final-second, and
`250 ms` post-tail-silence policy.

No selector or production path changed.

## Next Task

Complete all six cross-source trials. Freeze click/pop, pull/thump, fade,
continuity, and preference notes before opening the key. Reproduce the expected
centroid preference split or close tail-envelope work.
