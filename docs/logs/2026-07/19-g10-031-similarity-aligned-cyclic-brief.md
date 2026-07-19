# g10.031 Similarity-Aligned Cyclic Brief

Date: 2026-07-19
Status: Batch 31.13 complete; isolated candidate ready

## Decision

Froze one complete clean-room `SimilarityAlignedCyclic` renderer for explicit
fixed expansion above `1x` through `8x`.

The renderer uses one exact rational ideal map, one seeded output lattice, one
bounded two-stage zero-mean correlation search, one strictly increasing source
anchor path, forward unit-rate native reads, complementary overlap windows,
and exact rolling normalization.

`detail` maps overlap support from about `64 ms` to `32 ms`. `motion` maps the
launch hop from `2.5` to `1.5` overlap lengths. Search radius stays fixed at
about `12 ms`; coarse spacing stays near `0.25 ms`. A full score below `0.25`
returns to the nominal legal anchor. Candidate-time geometry, score, search,
confidence, fallback, window, stereo, or threshold choices are closed.

Linked channels share candidates, scores, anchors, windows, and normalization
while synthesizing native samples. Neutral `space` is identity; maximum width
uses a bounded `1.5` side gain. State is duration-independent and capped at
`4 MiB`.

## Admission

Frozen order:

1. complete structural controls
2. retained neutral `110 Hz` at `2x` first
3. remaining creative synthetic controls
4. five-source concealed mono comparison at `2x`, `4x`, and `8x`
5. exact `16x` rejection before any `16x` output allocation
6. independent linked-stereo listening
7. minimal private admission only after every prior gate passes

Any miss rejects and deletes the whole candidate. A second pitch/join failure
closes explicit `Cyclic`; another dominant cause returns only to docs-level
reassessment. No correction, rerun, parameter change, score variant, threshold
sweep, or partial mechanism survives rejection.

## Boundary

Changed documentation only. No DSP, candidate module, harness, fixture,
comparator audio, report mode, public API, cache, route, dependency, Loophole,
or Chorus surface entered `main`. The three unrelated binaural/reverb edits
remain untouched.

## Next Task

Execute `g10.031` Batch 31.14 only. Create sibling worktree
`/Users/tom/Dev/projects/signal-candidate-31-14` on branch
`candidate/g10-031-similarity-aligned-cyclic` from the Batch 31.13 docs commit,
implement the frozen brief once, and stop at the first failed gate.
