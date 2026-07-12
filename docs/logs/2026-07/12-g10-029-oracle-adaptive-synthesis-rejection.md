# Oracle Adaptive Synthesis Rejection

Date: 2026-07-12
Roadmap: `g10.029`
Batch: `29.6BA`

## Result

Reject the frozen oracle time-adaptive synthesis candidate before corpus
rendering.

The report-only renderer implements the declared four-window schedule, absolute
source-to-output centre mapping, actual-hop identity-locked phase transport,
deterministic spectral-peak ownership, whole-sample reflection, and the exact
output-side dual.

Across tone, chirp, impulse, dense impulse, boundary, mixed, noise, and silence
controls, the mechanism passes schedule legality, mapping error, output
coverage, frame-operator positivity, identity reconstruction, coefficient and
sample finiteness, conjugate symmetry, imaginary residue, exact length, and
deterministic repeat.

The isolated impulse placement errors at `[1.0, 0.75, 1.25, 1.5]` are
`[0, 0, 0, -127]` frames. The `1.5x` peak occurs at frame `6017`; its declared
position is `6144`.

## Decision

This is a synthetic mechanism failure, not a listening ambiguity. The contract
requires a stop before the 15-row sidecar and corpus candidate. Batch 29.6BB
closes without export. Batch 29.6BC retires the time-adaptive successor lane.
Automatic selection remains closed.

## Next Task

Pause DSP successor implementation. Reassess the algorithm class at the next
operator checkpoint without reopening detector research or tuning this
candidate.
