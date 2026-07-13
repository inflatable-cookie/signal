# Shared Full-Field Phase Rejection

Date: 2026-07-13
Roadmap: `g10.029`
Batch: `29.6BN`
Status: rejected

## Result

One physical-frequency synthesis phase field does not make three redundant
full-band resolution layers coherent. The proof passes every structural gate
but fails arrival, correlation, and replica-growth gates by wide margins.

No listening export, tuning, or holdout read opens.

## Mechanism

- common phase lattice uses the longest FFT's physical-frequency grid
- every layer bin maps exactly onto that lattice
- one state advances through the exact output schedule
- coincident layer frequency evidence combines by linked energy
- event correction mutates shared state once under the frozen reset scope
- solved phase projects back to every layer before union-dual synthesis
- identity retains the exact unmodified path

## Evidence

- configurations: `3`
- development rows: `9`
- total renders: `33`
- structural failures: `[0,0,0,0,0,0,0,0,0]`
- mean pairwise event disagreement: `162.261364` frames; gate `<8`
- maximum pairwise event disagreement: `506` frames
- mean pairwise correlation: `0.134045`; gate `>0.8`
- mean layer replica count: `36.363636`
- mean combined replica count: `37.073864`
- replica growth: `0.710227`; gate `<=0`
- maximum layer-sum error: `1.6653345369377348e-16`
- event resets: `78`
- shared phase assignments: `3,959,604`
- holdout reads: `0`
- report: `target/stretch-successor-bn-shared-phase-proof.tsv`
- report SHA-256:
  `146100352650e01687be966ec2feb2ad3d55c8f828f67d56540a2d4068629415`

## Decision

Retire redundant full-band union ownership. Batch 29.6BO must select a
non-duplicating multi-resolution representation before more synthesis code.
The review is limited to complementary source subbands, explicit coefficient
tiling, and one invertible adaptive-resolution representation.

## Next Task

Execute Batch 29.6BO architecture review. Keep implementation, holdout, and
tuning closed.
