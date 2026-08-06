# g10.042 Batch 42.2 - Pitch Design Frozen

Status: complete
Created: 2026-08-05
Scope: how the resumable renderer takes on pitch composition

## Most Of The Answer Was Already In The Codebase

The batch existed to design resampler state carry across chunk boundaries.
`signal-dsp-resample` already has it.

`StreamingResampler` exposes `process_chunk` and `finish` and carries a
`pending` history buffer plus a fractional `next_source_index` cursor — exactly
the state a chunk boundary destroys. `resample_mono`, which the whole-buffer
pitch path calls, is a thin wrapper that constructs one, processes everything,
and finishes it.

So the frozen design is: one `StreamingResampler` per mid/side channel, fed per
chunk, finished at flush. No new resampler, and none should be written.

Worth stating because the roadmap described this as the lane's design problem.
Reading the dependency first turned a design batch into a wiring batch.

## The Trap, Frozen Before Implementation Meets It

`pitch_shift_resample_config` resamples from a virtual rate of
`sample_rate * 2^(semitones/12)` down to the nominal rate, changing the frame
count by `2^(-semitones/12)`. But `target_frames` is computed from the
**original** frame count, before any resampling.

So the stretcher's effective ratio is not the nominal one:

```
effective = (frames * ratio) / (frames * 2^(-semitones/12))
          = ratio * 2^(semitones/12)
```

The resumable renderer takes its ratio curve in source-frame coordinates. Under
pitch that curve must be converted to *pitched*-frame coordinates, positions and
ratios both scaled by `2^(semitones/12)`.

This is frozen first because of how it fails. Get it wrong and the render comes
out exactly the right length, chunk-count independent, with no dropped source —
and its ratio automation lands in the wrong places. Every gate `g10.039` built
for the default path would pass it.

Batch 42.3 therefore has to prove the curve lands correctly as a separate gate,
because none of the existing ones can see it.

## Stage And Flush Order

Resample first, then stretch — the reverse of what the batch name suggests, and
the order the whole-buffer path already uses. Mid/side rather than left/right,
which is what keeps the stereo image stable under pitch.

`flush` finishes the resamplers first, pushes their residual through the stretch
stage, and flushes the stretcher last. The other order discards the resampler
tail, which is a source drop of exactly the kind `g10.039` spent a lane on.

## What Was Not Frozen

The working-set ceiling. Contract `046` records that a memory bound is a
consequence of a working design rather than something fixable ahead of one, after
`g10.039` moved its ceiling three times and `g10.040` overshot its estimate by
`2x`. It is deferred to Batch 42.3 and derived from the implementation.

## Next Task

Open Batch 42.3 and implement the frozen design.
