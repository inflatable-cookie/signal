# g10.042 Batch 42.3 - Pitch Implemented, Chunk Independence Open

Status: implemented; one gate failing, defect open
Created: 2026-08-05
Scope: resumable pitch composition

## What Landed

`ResumableStretchConfig` gains `sample_rate` and `pitch_shift_semitones`.
`ResumableOfflineStretch` gains a `PitchStage` holding one `StreamingResampler`
per mid/side channel, upstream of the stretch stage, per the Batch 42.2 freeze.

`resumable_render_supported` is unchanged, so pitched artifacts still take the
legacy path. Nothing in production reaches the new code, and the seam smoother
stays until Batch 42.4.

## Two Gates Pass

`G7`: the pitch happens, in the right direction. `+12` semitones takes a
`220 Hz` tone to `440 Hz`, `-12` to `110 Hz`, `+7` to `329.6 Hz`, each within
`6%`. Length alone could not see this — a renderer ignoring pitch entirely
still produces the contracted length.

`G8`: the ratio curve lands in pitched coordinates, which is the trap Batch 42.2
froze in advance. A curve of `1.0` for the first half of the source and `2.0`
for the second produces `1.5x` overall duration under a `+7` semitone shift,
within `2%`. Freezing that rule before implementing is why this worked first
time rather than being found by a listener later.

## One Gate Fails

Pitched renders are not chunk-count independent. Worst sample delta
`0.0057568103` at `-5` semitones with `3` chunks, on a `0.4`-amplitude tone.
Lengths match exactly, and the first divergence is `39.8%` through the render —
not at a chunk boundary in either coordinate system, and not at the tail.

Four causes are eliminated by measurement:

| ruled out | evidence |
| --- | --- |
| `StreamingResampler` itself | `0.0` delta across `3`, `7`, `16` chunks |
| the mid/side pitched material this stage builds | `0.0` delta across `3` and `7` chunks |
| the unpitched renderer at the effective ratio | `0.0` delta at `1.5`, `1.123`, `1.0`, `2.0`, `0.8` |
| frames stranded by the feed loop | instrumented; the carry path never fires |

The resampler is correct. The pitched material is correct. The stretcher is
correct at the ratio pitch produces. Nothing is dropped. And the output still
differs. That combination is not explained yet.

## Two Wrong Diagnoses, Recorded

The first was a missing carry buffer: the feed loop can exit with frames
outstanding, and the pitched buffer was local, so a remainder would have been
dropped. Adding the carry produced a delta identical to the last digit —
`0.0057568103` before and after — which proved the path never fires. The carry
is kept because the hazard is real, but it was not this bug, and the identical
digits are what showed that rather than a second opinion.

The second was that chunk independence might be ratio-dependent, exposed by the
`1.123` effective ratio pitch produces. Testing the unpitched renderer at that
exact ratio gave `0.0` delta.

Both were plausible, both were cheap to test, and both were wrong. Recording
them so the next attempt does not spend the same time.

## Status Of The Gate

`#[ignore]`d with the measured value and the ruled-out list in its reason,
following the `g10.039` `G5` and `g10.041` `A18` precedent. The gate stays and
reproduces the defect on demand; un-ignoring it is what closes the batch.

## Next Task

The one experiment not yet run: feed pre-computed pitched material into the
*unpitched* renderer, sliced the way the pitch stage slices it. That separates
"the stretcher dislikes this push pattern" from "the pitch stage corrupts
something on the way in", and the four eliminations above leave the interaction
between the stages as what remains.
