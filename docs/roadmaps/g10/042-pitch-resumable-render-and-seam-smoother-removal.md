# 042 - Pitch Resumable Render And Seam Smoother Removal

Status: active; pitch design frozen, implementation ready
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

- [ ] teach the resumable renderer pitch composition, carrying resampler state
  across chunk boundaries the way phase state already is
- [ ] route pitch-shifted artifacts through it
- [ ] delete `materialize_chunked_offline_stretch_artifact_frames` and
  `smooth_artifact_chunk_boundaries_interleaved`
- [ ] state explicitly what remains of the dynamic-segment smoother and why

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

Status: ready; Batch 42.2 froze the design on 2026-08-05

- [ ] implement isolated per Contract `084` Rule 2
- [ ] freeze the working-set ceiling with the resampler included, derived from
  the implementation
- [ ] prove chunk-count independence for pitch-shifted renders across at least
  three chunk counts, as `g10.039` did for the default path
- [ ] prove the ratio curve lands correctly in pitched coordinates, which a
  length check cannot see
- [ ] prove no dropped source, with a metric shown to fire on an injected drop

### Batch 42.4 - Delete The Chunked Renderer

Status: blocked on Batch 42.3

- [ ] route pitch-shifted artifacts through the resumable renderer
- [ ] delete `materialize_chunked_offline_stretch_artifact_frames` and
  `smooth_artifact_chunk_boundaries_interleaved`
- [ ] record what remains of `smooth_dynamic_segment_boundaries_interleaved`
  and which callers keep it alive
- [ ] update Contract `046` and the `g10` front doors

## Acceptance Criteria

- [x] the paths that still produce chunk seams are named exactly
- [ ] pitch-shifted artifacts render chunk-count independently
- [ ] the chunk smoother and the chunked renderer are gone
- [ ] the remaining smoother's callers are recorded

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

Open Batch 42.3 and implement the frozen design.

Two things it does not have to work out. `StreamingResampler` already exists in
`signal-dsp-resample` with `process_chunk` and `finish`, carrying the history
buffer and fractional cursor a chunk boundary needs — `resample_mono` is a thin
wrapper over it, so no new resampler should be written. And the ratio curve must
be converted to pitched-frame coordinates, scaled by `2^(semitones/12)`, because
`target_frames` is computed before resampling while the stretcher runs after it.

That second point is the one to watch. Getting it wrong yields a render of
exactly the right length with its ratio automation in the wrong places, which
neither a length check nor a chunk-independence check would catch.
