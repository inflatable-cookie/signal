# 042 - Pitch Resumable Render And Seam Smoother Removal

Status: active; Batch 42.1 complete, scope established and one defect closed
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

### Batch 42.2 - Resumable Pitch

Status: ready

- [ ] freeze how resampler state carries across chunk boundaries, alongside the
  phase, detector, and overlap-add state already carried
- [ ] freeze the working-set ceiling with the resampler included, derived after
  the design rather than before it
- [ ] implement isolated per Contract `084` Rule 2
- [ ] prove chunk-count independence for pitch-shifted renders, the way
  `g10.039` did for the default path
- [ ] prove no dropped source, with a metric shown to fire on an injected drop

### Batch 42.3 - Delete The Chunked Renderer

Status: blocked on Batch 42.2

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

Open Batch 42.2. The design question is narrow: the resumable renderer already
carries phase, detector, and overlap-add state across chunk boundaries, and
pitch composition adds a resampler whose state must carry the same way.

Everything else about the lane is settled — selectors are whole-buffer and
correct, and the only route left into the chunked renderer is pitch-shifted
multi-chunk artifacts.
