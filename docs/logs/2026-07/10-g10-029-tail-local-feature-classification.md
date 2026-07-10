# g10.029 Tail-Local Feature Classification

Date: 2026-07-10
Status: labeled separation found; selector validation required

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

## Next Task

Measure the broad source pool with the same feature row. Select a bounded
cross-source validation pack with distinct sources below and above 2 kHz. Compare
current, additive, and multiplicative candidates under concealment. Do not wire
the provisional selector before those notes are frozen.
