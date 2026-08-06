# A24 - StreamingResampler Is Not Bit-Exact At Chunk Seams

Status: open, guarded
Created: 2026-08-05
Scope: `signal-dsp-resample`, and why pitched resumable renders cannot be
byte-identical across chunk counts

## The Chain

`g10.042` Batch 42.3 implemented pitch in the resumable renderer. It failed
chunk-count independence by `0.0057568103`. The renderer turned out to be
correct.

Isolating with one fixed pitched buffer through two push patterns, and one
differing buffer through a fixed pattern:

| experiment | worst delta |
| --- | --- |
| same material, different push sizes | `0.0000000` |
| different material, same push sizes | `0.0057568` |

So the stretch stage is push-pattern independent, and every bit of the
divergence comes from its input.

`StreamingResampler` is not bit-exact across chunk boundaries: `2.98e-8`, one
ULP, first differing at sample `44609` for `2` chunks, `29093` for `3`, `13771`
for `7` — the first seam in each case. Seam arithmetic rather than accumulating
drift.

## The Amplification

One ULP in, `5.8e-3` out. Roughly `190000x`.

That is not a defect in the vocoder, it is what a phase vocoder is. A magnitude
change of any size can flip which bin is a local spectral peak, which changes
the phase-locking region a bin belongs to, which changes its synthesis phase by
an arbitrary amount. Peak picking is a discontinuous function of its input.

The consequence generalises beyond this lane: **any stage upstream of the
vocoder must be bit-exact, not merely accurate**, or downstream byte-comparison
gates cannot pass. `g10.039` never met this because nothing sat upstream of its
stretcher.

## Three Measurements That Were Wrong

The first pass reported `StreamingResampler` byte-exact across `3`, `7` and `16`
chunks, and reported the pitched material byte-exact too. Both were wrong the
same way: a `1.0e-6` threshold with seven-decimal printing, so `2.98e-8`
displayed as `0.0000000` and passed twice.

Those two false eliminations sent the search away from the actual cause for
several rounds, while two genuinely correct eliminations — a missing carry
buffer and ratio-dependent chunk independence — cost little.

The rule this generation keeps relearning turns out to apply to precision as
much as to detection. A gate has to be shown capable of seeing the thing it
claims absent, and a `1.0e-6` threshold cannot see one ULP.

## Guard

`signal-dsp-resample/tests/chunk_boundary_exactness.rs`. Two tests: output
length is chunk-independent and passes today; bit-exactness reproduces `A24` and
is `#[ignore]`d with the measurement in its reason.

## Next Task

Make the resampler bit-exact at the seam — most likely how the `pending` history
and the fractional `next_source_index` combine on the first sample after a
boundary. That un-ignores the pitched chunk-independence gate and unblocks
`g10.042` Batch 42.4.
