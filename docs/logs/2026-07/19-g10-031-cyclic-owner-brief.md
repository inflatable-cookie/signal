# g10.031 Cyclic Owner Brief

Date: 2026-07-19
Status: Batch 31.10 complete; isolated candidate ready

## Decision

Froze one clean-room Signal-owned `CyclicGrain` renderer for explicit fixed
expansion above `1x` through `8x`.

The renderer uses one sample-centred monotonic map, one deterministic grain
lattice, at most two overlapping unit-rate reads, normalized raised-cosine
crossfades, linked-channel scheduling, and exact rolling output. Unit-rate
reads own pitch. Mapped source-anchor advance owns duration. The scheduled
source offset owns the intentional Akai-style cycle under normalized crossfade.

`detail` maps logarithmically from long to short cycle support. `motion` maps
from broad to denser overlap. `space` applies one bounded linked mid/side law.
No raw grain control becomes product vocabulary.

## Admission

Frozen order:

1. structural controls at identity, `2x`, `4x`, and `8x`
2. cyclic synthetic controls
3. five-source concealed mono comparison at `2x`, `4x`, and `8x`
4. exact `16x` rejection before allocation
5. independent linked-stereo listening
6. minimal private admission only after every prior gate passes

The missing `2x` ReaReaRea comparator must be captured under the retained
common-RMS / `0.95` peak-ceiling policy inside ignored candidate state. Any
gate miss rejects and deletes the whole candidate. No local scalar or mechanism
sweep follows.

## Boundary

Changed documentation only. No DSP, candidate module, harness, fixture, report
mode, generated audio, public API, dependency, cache, route, Loophole, or
Chorus surface entered `main`. The three unrelated binaural/reverb edits remain
untouched.

## Next Task

Execute `g10.031` Batch 31.11 only. Create one disposable worktree from this
docs commit, implement the frozen `CyclicGrain` brief once, and stop at the
first failed gate.
