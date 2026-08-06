# A24 Fixed - Resampler Seam Bit-Exactness

Status: complete
Created: 2026-08-05
Scope: `signal-dsp-resample`, and the pitched chunk-independence gate it blocked

## The Cause

`StreamingResampler` advanced a running `next_source_index` by `+= step` for
every output sample, and rebased it by `-= drain_count` whenever `pending` was
trimmed.

Floating-point addition is not associative. Interleaving those subtractions
produces a different value from the unchunked sequence, which is why the
difference appeared exactly at the first seam and measured one ULP — `2.98e-8`.

## The Fix

The read position is now derived rather than accumulated. The resampler counts
output samples in `emitted` and input samples dropped in `drained`, and computes
the position as a pure function of both.

The integer and fractional parts are separated before rebasing:

```rust
let absolute = self.emitted as f64 * self.step;
let absolute_floor = absolute.floor();
let fraction = absolute - absolute_floor;
(absolute_floor - self.drained as f64) + fraction
```

The fraction is carried untouched and only the whole-sample offset moves, so the
`(left_index, fraction)` pair the interpolator receives is identical however the
input was chunked. Rebasing a fraction through subtraction would have reintroduced
the rounding this removes.

## Result

Bit-exact across `2`, `3` and `7` chunks. The guard in
`signal-dsp-resample/tests/chunk_boundary_exactness.rs` is un-ignored and passes
with exact `==` comparison rather than a threshold — a threshold is what hid this
for several rounds.

`g10.042`'s pitched chunk-independence gate is un-ignored and passes. That lane's
renderer never had a defect; it was reporting one it did not cause.

## Behaviour Version

`SIGNAL_STRETCH_BEHAVIOR_VERSION` advances to
`signal-stretch-behavior-2026-08-05-a24-resampler-seam`.

The whole-buffer `resample_mono` path also drains before finishing, so its output
can shift by an ULP too. No test asserted a value that changed, and the full
suite passes — but "no test noticed" is not "nothing changed", and a cached
pitched artifact rendered by the old resampler may differ. Bumping is the cheap
side of that.

## Next Task

`g10.042` Batch 42.4: route pitched artifacts through the resumable renderer,
then delete `materialize_chunked_offline_stretch_artifact_frames` and
`smooth_artifact_chunk_boundaries_interleaved`. That is a shipped DSP change for
pitched renders, so Contract `084` Rule 5 listening applies before adoption.
