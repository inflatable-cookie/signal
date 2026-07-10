# g10.029 Frequency-Adaptive Reassessment

Date: 2026-07-10
Status: decision frozen

## Evidence

Exact lattice bounded source-map error to `0.4` frame and retained the
phase-gradient candidate's tonal gains. The complete mono result still passed
only `3/60`: timing worsened `17.789744` frames on average, replica protection
passed `27/48`, and integrity passed `57/60`. Mapping was a real defect, not the
dominant placement or shape mechanism.

Frequency-adaptive painless nonstationary Gabor frames are the next materially
different clean-room family. They support nonuniform frequency resolution and
canonical-dual perfect reconstruction inside one filter bank. Long
low-frequency atoms can retain tonal selectivity while short high-frequency
atoms expose attacks at finer time resolution.

The published onset-adaptive NSG phase vocoder is not adopted. It detects
attacks and locally forces unity stretch, then constructs synthesis windows
around relocated onsets. Signal already rejected local time redistribution.

## Decision

Batch 29.6I proves the transform before the stretch:

- one frequency-adaptive painless frame
- constant-Q interior bands plus DC and Nyquist completion
- canonical dual filters from the diagonal frame operator
- identity analysis/synthesis only
- explicit reconstruction, coverage, band-delay, and determinism evidence

No phase modification, corpus render, stereo, dynamic ratio, cache identity,
or product routing opens. A passing proof authorizes a separate
frequency-adaptive phase-gradient mechanism contract; it does not authorize a
product candidate.

## Next Task

Implement Batch 29.6I and run its focused synthetic reconstruction gate.
