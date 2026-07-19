# g10.031 Complex-Relation Reassessment

Date: 2026-07-19
Status: Batch 31.7 complete; final candidate brief frozen

## Decision

Closed polar native-relation reconstruction. The Batch 31.6 failure came from
wrapped angle subtraction, not the continuous excitation or source-orientation
owner.

Froze one direct complex relation law:

- normal linked state uses `unit(X_c*conj(R))` in scaled `f64`
- exact cancellation uses a channel-symmetric polarity-unoriented axis
- incidental cancellation combines that axis with source orientation
- exact whole-source anti-phase lets native relations own the polarity flip
- linked carrier reference follows the same normal/cancellation split
- signed zero, silence, DC, Nyquist, swap, and polarity behavior are explicit

The first candidate gate is exhaustive coefficient symmetry. The first
renderer row is the exact failed common-polarity case. Crest admission remains
closed until every structural row passes.

## Boundary

No DSP, candidate module, test, harness, fixture, report mode, public API,
cache, route, dependency, generated audio, Loophole, or Chorus surface changed.
The final brief permits one more disposable candidate. Any failure closes the
current diffusive owner.

## Validation

- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed

## Authority

- `docs/architecture/offline-creative-continuous-excitation-complex-relation-brief.md`
- `docs/contracts/085-creative-time-stretch-product-and-routing-contract.md`
- `docs/roadmaps/g10/031-creative-time-stretch.md`

## Next Task

Run Batch 31.8 only. Implement the final brief once in a disposable worktree.
Run relation proof and the prior common-polarity row first. Stop and delete on
failure. Do not open crest admission, later owners, or product routing early.
