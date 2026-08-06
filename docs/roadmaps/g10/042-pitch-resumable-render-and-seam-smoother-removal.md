# 042 - Pitch Resumable Render And Seam Smoother Removal

Status: complete; the chunked renderer and its seam smoother are deleted
Owner: dsp
Created: 2026-08-05
Depends on: `g10.039`
Governing contracts: `docs/contracts/046-sample-domain-time-stretch-engine-contract.md`,
`docs/contracts/084-stretch-candidate-isolation-and-promotion-contract.md`
Vision tags: `DSP`, `STRETCH`, `RENDER`

## Problem

`g10.039` adopted the resumable renderer on the default offline path and left
both seam smoothers in place, because "selector paths and pitch composition
still take the legacy per-chunk path". Removal was deferred to "adopting the
remaining offline paths".

That framing was too broad, and acting on it directly would have meant
redesigning the selectors for no reason.

## What Is Actually Left

Measured against the code rather than the roadmap's summary.

The artifact renderer branches four ways. Selector paths —
`CompressionShortWindowSelector` and `ExpansionShortWindowSelector` — take
`selector_offline_path_requires_static_materialization` and render
**whole-buffer**. They never chunk, so they never create a chunk seam and the
smoother was never for them.

The resumable path covers `Default` with no pitch shift. Single-chunk artifacts
with pitch take a whole-buffer call.

So `materialize_chunked_offline_stretch_artifact_frames`, and with it
`smooth_artifact_chunk_boundaries_interleaved`, is reached in exactly one case:
**pitch-shifted, multi-chunk artifacts**.

The remaining work is therefore not "adopt the selectors". It is: teach the
resumable renderer pitch, then delete one renderer and one smoother.

The other smoother, `smooth_dynamic_segment_boundaries_interleaved`, is a
separate question. It patches dynamic-ratio *segment* joins inside the
whole-buffer stretcher, which has consumers unrelated to the artifact path, so
it does not leave with the chunked renderer.

## Goals

- [x] teach the resumable renderer pitch composition, carrying resampler state
  across chunk boundaries the way phase state already is
- [ ] route pitch-shifted artifacts through it
- [x] delete `materialize_chunked_offline_stretch_artifact_frames` and
  `smooth_artifact_chunk_boundaries_interleaved`
- [x] state explicitly what remains of the dynamic-segment smoother and why

## Non-Goals

- no selector redesign; they render whole-buffer and are correct as they are
- no removal of the dynamic-segment smoother, which serves other callers
- no change to admitted DSP behaviour

## Batches

### Batch 42.1 - Establish Scope, Close The Fallback

Status: complete

- [x] determine which paths actually still produce chunk seams
- [x] remove the silent renderer fallback found while doing so

#### The Fallback Broke The Invariant Its Own Comment Asserts

`materialize_resumable_offline_stretch_artifact_frames` returned `Option`, and
the caller did:

```rust
// Length must not select the algorithm: a single-chunk artifact and
// a multi-chunk artifact of the same source share a cache key, so
// they must share a renderer.
materialize_resumable_offline_stretch_artifact_frames(..)
    .unwrap_or_else(|| materialize_chunked_offline_stretch_artifact_frames(..))
```

Three `.ok()?` sites fed that `Option` — construction, `render`, and `flush` —
so any error switched to the legacy chunked renderer and produced a different
render under the same cache key. That is precisely what the comment forbids,
undone by the safety net directly beneath it.

`render` is not hypothetically fallible. `g10.039` deliberately made it return
an error rather than discard source when a drain cannot advance, after the
silent-render incident. So the one error the fallback was most likely to catch
is the one it would have hidden.

Construction cannot fail here — the configuration is fixed, and the only
rejections are an over-large window or an unsupported channel count. It is now
an expectation. `render` and `flush` failures surface instead of switching
renderers.

#### Padding Is Now Bounded

The same function ended with `output.resize(planned, 0.0)`. Unbounded
zero-padding to a contracted length is the mechanism that let `g10.039` ship
three silent specimens: the renderer produced `3.8s` of audio, the rest was
padded, and nothing downstream noticed.

Rounding can legitimately leave the render a frame or two short, so padding is
capped at `4` samples and asserted. The whole render-plane suite passes with
that bound, which says the renderer was not relying on padding.

### Batch 42.2 - Freeze The Pitch Design

Status: complete

- [x] freeze how resampler state carries across chunk boundaries, alongside the
  phase, detector, and overlap-add state already carried
- [x] freeze the ratio-coordinate relationship under pitch
- [ ] ~~freeze the working-set ceiling~~ — deferred to Batch 42.3, after the
  design exists rather than before it, per Contract `046`

Frozen below. Batch 42.3 implements this and does not renegotiate it; a defect
in the brief is corrected here and re-frozen.

#### The State-Carrying Resampler Already Exists

`signal-dsp-resample` already exposes `StreamingResampler`, with `process_chunk`
and `finish`, carrying a `pending` history buffer and a fractional
`next_source_index` cursor. `resample_mono` — the function the whole-buffer
pitch path calls — is a thin wrapper that constructs one, processes the whole
buffer, and finishes it.

So the design question this batch was opened to answer is largely already
answered in code. The resumable renderer needs one `StreamingResampler` per
mid/side channel, fed per chunk, finished at flush. No new resampler is
required and none should be written.

#### Frozen: Stage Order

Pitch composition is **resample, then stretch** — not the reverse.
`stretch_pitch_interleaved_stereo` resamples the source to a nominal rate and
feeds the result to the stretcher, so in the resumable renderer the resampler
sits upstream of the existing phase-vocoder stage and its output is what the
stretcher consumes.

Mid/side, not left/right: the existing path converts to mid/side, resamples each,
and recombines. That is what keeps the stereo image stable under pitch, so the
resumable version does the same rather than resampling channels independently.

#### Frozen: The Ratio Coordinate Shifts Under Pitch

This is the part that would have been discovered painfully during
implementation, so it is frozen first.

`pitch_shift_resample_config` builds a resampler from a *virtual* input rate of
`sample_rate * 2^(semitones/12)` to the nominal rate, so it changes the frame
count by `2^(-semitones/12)`. But `target_frames` is computed from the
**original** frame count times the ratio, before any resampling.

The stretcher's effective ratio is therefore not the nominal ratio:

```
effective = target_frames / pitched_frames
          = (frames * ratio) / (frames * 2^(-semitones/12))
          = ratio * 2^(semitones/12)
```

The resumable renderer's ratio curve is expressed in source-frame coordinates.
Under pitch it must be expressed in *pitched*-frame coordinates, with both the
curve's positions and its ratios scaled by `2^(semitones/12)`. Getting this
wrong produces a render of the right length whose ratio automation lands in the
wrong places — which no length or chunk-independence check would catch.

#### Frozen: Flush Order

`flush` must finish the resamplers first, push their residual through the
stretch stage, and only then flush the stretcher. Flushing the stretcher first
would discard the resampler tail, which is a source drop of exactly the kind
`g10.039` spent a lane on.

### Batch 42.3 - Implement Resumable Pitch

Status: complete; `A24` fixed and every gate green

- [x] implement isolated per Contract `084` Rule 2
- [ ] freeze the working-set ceiling — deferred until the render is correct
- [x] prove chunk-count independence — passes once `A24` was fixed
- [x] prove the ratio curve lands correctly in pitched coordinates
- [x] prove the pitch shift happens, and in the right direction

`ResumableStretchConfig` gains `sample_rate` and `pitch_shift_semitones`, and
`ResumableOfflineStretch` gains a `PitchStage` holding one `StreamingResampler`
per mid/side channel, per the frozen design. `resumable_render_supported` is
unchanged, so pitched artifacts still take the legacy path and nothing in
production reaches this code.

#### Fixed

`A24` is fixed and the gate passes. `StreamingResampler` now derives its read
position from the absolute output index instead of accumulating and rebasing it,
with the integer and fractional parts separated so the rebase is exact. Bit-exact
across `2`, `3` and `7` chunks, and the pitched render is chunk-count independent.

What follows is the diagnosis, kept because the eliminations are the useful part.

#### Diagnosed: The Renderer Is Correct, The Resampler Is Not Bit-Exact

Pitched renders differ across chunk counts by `0.0057568103` at `-5` semitones
with `3` chunks. The cause is not in this renderer.

Isolating by feeding one fixed pitched buffer through two push patterns, and one
differing buffer through a fixed pattern:

| experiment | worst delta |
| --- | --- |
| same material, different push sizes | `0.0000000` |
| different material, same push sizes | `0.0057568` |

The stretch stage is push-pattern independent. All of the divergence comes from
the input material.

`StreamingResampler` is not bit-exact across chunk boundaries. Measured with
exact comparison rather than a threshold: `2.98e-8` — one ULP — first differing
at sample `44609` for `2` chunks, `29093` for `3`, and `13771` for `7`, which is
the first seam in each case. Seam arithmetic, not drift. Recorded as `A24` with a
guard in `signal-dsp-resample/tests/chunk_boundary_exactness.rs`.

#### The Amplification Is The Interesting Number

One ULP in, `5.8e-3` out. The phase vocoder amplifies the difference by roughly
`190000x`.

That is not a bug, it is what a phase vocoder does: a magnitude change of any
size can flip which bin is a local spectral peak, which changes the phase-locking
region a bin belongs to, which changes its synthesis phase by an arbitrary
amount. Peak picking is a discontinuous function of the input.

It has a consequence for every bit-exactness gate in this generation. Any stage
placed upstream of the vocoder must be bit-exact, not merely accurate, or
downstream byte-comparison gates cannot pass. `g10.039` got away with this only
because nothing sat upstream of its stretcher.

#### Three Measurements That Were Wrong

The first pass reported `StreamingResampler` byte-exact with `0.0` delta at `3`,
`7` and `16` chunks, and the pitched material byte-exact too. Both were wrong in
the same way: the comparison used a `1.0e-6` threshold and printed to seven
decimal places, so `2.98e-8` displayed as `0.0000000` and passed.

Two further hypotheses were tested and eliminated correctly — a missing carry
buffer, and ratio-dependent chunk independence — but the eliminations that
mattered were the two false ones, and they sent the search away from the actual
cause for several rounds.

The rule this generation keeps relearning applies to precision as well as to
detection: a measurement has to be shown capable of seeing the thing. A threshold
of `1.0e-6` cannot see a one-ULP difference, and formatting hid it twice.

#### What Did Work

`G7` proves the pitch happens and in the right direction — `+12` semitones
takes a `220 Hz` tone to `440 Hz`, `-12` to `110 Hz`, `+7` to `329.6 Hz`, each
within `6%`.

`G8` proves the ratio curve lands in pitched coordinates, which is the trap
Batch 42.2 froze. A curve of `1.0` for the first half and `2.0` for the second
produces `1.5x` overall duration under a `+7` semitone shift, within `2%`.

### Batch 42.4 - Delete The Chunked Renderer

Status: complete; admitted by listening 2026-08-05 and adopted

- [x] prove the two renderers differ enough to judge, and that both carry audio
- [x] build the concealed pack
- [x] listening admission under Contract `084` Rule 5
- [x] route pitch-shifted artifacts through the resumable renderer
- [x] delete `materialize_chunked_offline_stretch_artifact_frames` and
  `smooth_artifact_chunk_boundaries_interleaved`
- [x] record what remains of `smooth_dynamic_segment_boundaries_interleaved`
  and which callers keep it alive

#### The Pack

`~/Downloads/signal-listening-pack-42-pitch`, built by an `#[ignore]`d test in
`offline.rs` so it reaches the two private renderers directly with one shared
chunk plan.

| case | duration | pitch | chunks |
| --- | --- | --- | --- |
| `F1` | `90s` | `+5` semitones | `3` |
| `F2` | `150s` | `+5` semitones | `5` |

Sustained low chord with a percussive attack every `500 ms`, stereo, ratio
`1.25`. Sides assigned from a fixed seed.

Checked before delivery: both renderers honour the planned length exactly; every
decile of both carries audio, which `g10.039` did not check before shipping three
silent specimens; the sides differ by `1.65` peak so the pack can discriminate;
and RMS matches within `0.2%` so level gives nothing away.

Those last two need reading together. A `1.65` peak difference on material
peaking near `1.05` looks alarming until the levels are compared: the sides are
equally loud and differ in waveform, which is what two phase-vocoder renders of
the same material look like. Checking only the difference would have suggested
one side was broken; checking only the levels would have missed whether the pack
discriminates at all.

#### Why This Needs Listening

Pitch-shifted multi-chunk artifacts are the only case still served by the legacy
chunked renderer, and therefore the only remaining reason both seam smoothers
exist. Routing them through the resumable renderer changes shipped DSP, so Rule 5
applies — as it did for the default path in `g10.039`, where listening rejected
the first attempt.

#### Admitted

Judged 2026-08-05. No case preferred the legacy side, so the resumable renderer
is admitted and adopted.

| case | A | B | reported |
| --- | --- | --- | --- |
| `F1` | resumable | legacy | both similar, no obvious seams; `A` consistent DC, `B` varies a lot |
| `F2` | legacy | resumable | same result, `B` consistent, `A` varies |

The listener picked out the legacy renderer in both cases without knowing which
side was which, and the sides were swapped between them. Measured per one-second
window, legacy's DC range and worst step are `0.00150` in both cases against
`0.00125` and `0.00142` for resumable — small in absolute terms, around `0.15%`
of full scale, visible at zoom rather than audible, but consistent in direction
and identified blind twice.

Neither side showed the seam artifact the pack was built to look for. The
admission therefore rests on "no case prefers legacy" rather than on the seam
being visibly fixed, which is what Rule 5 asks for.

#### Deleted

`materialize_chunked_offline_stretch_artifact_frames`,
`smooth_artifact_chunk_boundaries_interleaved`,
`materialize_stretch_chunk_payload`, and
`OFFLINE_STRETCH_ARTIFACT_CHUNK_CROSSFADE_FRAMES`. `398` lines out of
`offline.rs`.

The `is_single_chunk` branch went too. With `Default` on the resumable renderer
and both selectors rendering whole-buffer, `OfflineHighQualityPath`'s three
variants leave it unreachable — and removing it closes a latent version of the
defect this lane already fixed once: a single-chunk pitched artifact used to take
a whole-buffer call while a multi-chunk one took the chunked renderer, so length
selected the algorithm under one cache key. The branch is now an `unreachable!`
that names why rather than a silent fallback.

#### The Surviving Smoother

`smooth_dynamic_segment_boundaries_interleaved` stays. It patches dynamic-ratio
*segment* joins inside the whole-buffer stretcher, not chunk joins, and three
callers keep it alive:

- `TimeStretcher::stretch_dynamic_ratio_pitch_interleaved_stereo`
- `stretch_to_exact_mono`
- `stretch_dynamic_ratio_linked_stereo_with_engine`

None is the artifact path. It does not leave with the chunked renderer, and
removing it is a separate question about the whole-buffer dynamic-ratio path.
- [ ] update Contract `046` and the `g10` front doors

## Acceptance Criteria

- [x] the paths that still produce chunk seams are named exactly
- [x] pitch-shifted artifacts render chunk-count independently
- [x] the chunk smoother and the chunked renderer are gone
- [x] the remaining smoother's callers are recorded

## Risks and Mitigations

- Risk: resampler state carry reintroduces the seam it is meant to remove.
  Mitigation: Batch 42.2 proves chunk-count independence before adoption, as
  `g10.039` did.
- Risk: a metric that cannot see a dropped source returns a confident null.
  Mitigation: every gate must be shown to fire on an injected instance first —
  the rule `g10.039`, `g10.040` and `g10.041` each had to learn separately.

## Evidence Requirements

- [x] the branch analysis naming the one remaining chunked case
- [ ] chunk-count independence for pitch, across at least three chunk counts
- [ ] a drop-detection metric proven to fire on an injected drop

## Next Task

This roadmap is complete. The chunked artifact renderer and its seam smoother are
deleted, pitched artifacts render through the resumable renderer, and
`SIGNAL_STRETCH_BEHAVIOR_VERSION` advances so artifacts cached by the old path
invalidate.

`smooth_dynamic_segment_boundaries_interleaved` remains, with its three callers
recorded above. Removing it is a separate question about the whole-buffer
dynamic-ratio path, not a continuation of this lane.

Open elsewhere: `g10.040` Batch 40.6, a live preview render path, gated on a
consumer asking for it.
