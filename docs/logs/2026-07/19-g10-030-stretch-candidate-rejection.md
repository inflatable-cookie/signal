# g10.030 Stretch Candidate Rejection

Date: 2026-07-19
Status: complete
Roadmap: `g10.030` Batch 30.3

## Change

Implemented `SourceAnchoredMultiresolutionPhaseField` in disposable worktree
`signal-g10-030-candidate` on branch
`candidate/g10-030-source-anchored-multiresolution`, anchored at `13539f27`.
Production routing on `main` did not change.

The candidate passed structural admission: identity, exact length, finiteness,
absolute-map error, crop coverage, boundary integrity, determinism, fixed
working storage, mono duplication, silent peer, and channel swap. Isolated
tones also stayed within the `5` cent pitch limit at `0.75x`, `1.5x`, and
`2.0x`.

## Rejection

The first isolated-impulse anti-replica row failed at `0.75x`:

- source event: `8192`
- detector source centre: `7424`
- committed refined event: `7296`
- expected output event: `6144`
- strongest primary: `6272`, offset `+128`
- secondary peak: `6401`, projection offset `+257`
- secondary amplitude: `0.17113242`
- `-24 dB` ceiling: `0.063095726`

The middle-scale centered spectrum reports positive flux before the source
event enters `[x_k-H/2,x_k+H/2)`. Its one-shot token therefore commits early,
refines a silent location, and disarms before the same-centre short window can
own the real attack. Later overlapping frames create the displaced primary and
secondary peak. This contradicts the frozen detector, refinement, and
same-centre reassignment rules as a system.

Contract `084` stops the candidate here. Tonal rows, long-form exports, and
listening did not run. No threshold, event-row repair, selector, or second
candidate was attempted.

## Cleanup

Deleted the disposable worktree and branch with all candidate implementation,
tests, and instrumentation. `main` retains the frozen production renderer and
contains documentation only for this batch.

## Next Task

Run Batch 30.4 as an architecture reassessment. Replace the rejected brief
with one complete topology that resolves detector lookahead, event refinement,
and synthesis ownership together, or close the successor lane. Do not write a
second candidate in the reassessment batch.
