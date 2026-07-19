# g10.031 Similarity-Aligned Cyclic Rejection

Date: 2026-07-19
Status: Batch 31.14 complete; candidate rejected

## Result

Implemented the frozen `SimilarityAlignedCyclic` brief once in disposable
worktree `/Users/tom/Dev/projects/signal-candidate-31-14` on branch
`candidate/g10-031-similarity-aligned-cyclic`, based on docs commit
`fac44bd6`.

The private candidate contained only
`creative_similarity_cyclic/{mod,plan,schedule,alignment,synthesis,tests}.rs`
and one private `lib.rs` declaration. It implemented the frozen exact map,
seeded lattice, bounded two-stage correlation search, strictly increasing
source path, native unit-rate reads, linked-channel decisions, complementary
overlap, rolling normalization, exact crop, and bounded state.

Compile-only validation passed. Six of seven structural cases passed:

- request rejection and byte-exact identity
- exact length, finiteness, silence, and determinism
- geometry, exact map, schedule, and strict anchor path
- impulse energy confined to scheduled read support
- linked-stereo synthesis and `space` ownership
- peak, overlap, candidate-count, and state-cap bounds

## Stop

The frozen known-offset search case failed. The previous anchor was `2000` and
its exact natural continuation was source frame `6352`. The selector chose
frame `6432`.

The dominant cause is coarse-shortlist reachability. An exact match between
coarse samples is not guaranteed to enter full refinement when neighbouring
noise correlations do not rank in the top three. This is a structural search
failure, distinct from Batch 31.11 pitch displacement.

The gate was not corrected or rerun. The first `110 Hz` / `2x` synthetic row,
remaining synthetic rows, comparator capture, mono listening, `16x` probe, and
independent stereo listening did not run.

## Cleanup

Deleted the candidate worktree, branch, private module, tests, and build state.
No candidate DSP, harness, fixture, comparator audio, report mode, public API,
cache, route, Loophole, or Chorus surface entered `main`. The three unrelated
binaural/reverb edits remain untouched.

## Validation

- candidate compile-only gate: passed
- candidate structural filter: `6/7` passed; known-offset recovery failed
- `git diff --check`: passed
- `effigy qa:docs`: passed
- `effigy qa:northstar`: passed
- `effigy health`: passed
- `effigy validate`: passed
- disposable worktree and candidate branch: absent
- `effigy doctor`: unchanged pre-existing god-file and attention-marker
  findings

## Next Task

Execute `g10.031` Batch 31.15 only. Reassess explicit cyclic ownership at docs
level. Do not repair or reimplement `CyclicGrain` or
`SimilarityAlignedCyclic`.
