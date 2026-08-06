# g10.042 Batch 42.1 - Scope Established, Fallback Closed

Status: complete
Created: 2026-08-05
Scope: what still produces chunk seams, and a defect found establishing it

## The Deferred Work Was Smaller Than Its Description

`g10.039` left both seam smoothers in place and deferred removal to "adopting
the remaining offline paths", naming selector paths and pitch composition.

Taken at face value that means redesigning the selectors. Read against the code
it does not.

Selector paths — `CompressionShortWindowSelector` and
`ExpansionShortWindowSelector` — hit
`selector_offline_path_requires_static_materialization` and render
**whole-buffer**. They never chunk, so they never produce a chunk seam, and the
smoother was never for them. Single-chunk artifacts with pitch also take a
whole-buffer call.

`materialize_chunked_offline_stretch_artifact_frames`, and with it
`smooth_artifact_chunk_boundaries_interleaved`, survives in exactly one case:
**pitch-shifted, multi-chunk artifacts**.

So the remaining work is teaching the resumable renderer pitch, then deleting
one renderer and one smoother. Acting on the roadmap's summary instead would
have meant redesigning two paths that are already correct.

## The Fallback Broke The Invariant Directly Above It

`materialize_resumable_offline_stretch_artifact_frames` returned `Option`, and
the caller read:

```rust
// Length must not select the algorithm: a single-chunk artifact and
// a multi-chunk artifact of the same source share a cache key, so
// they must share a renderer.
materialize_resumable_offline_stretch_artifact_frames(..)
    .unwrap_or_else(|| materialize_chunked_offline_stretch_artifact_frames(..))
```

Three `.ok()?` sites fed that `Option`: construction, `render`, and `flush`. Any
of them switched to the legacy chunked renderer, producing a different render
under the same cache key — exactly what the comment forbids, undone by the
safety net beneath it.

`render` is not hypothetically fallible. `g10.039` made it return an error
rather than discard source, specifically after the incident where it discarded
source silently. The fallback would therefore have hidden the error it was most
likely to catch, by swapping in the renderer that error exists to distinguish
from.

Construction cannot fail at that call site: the configuration is fixed and the
only rejections are an over-large window or an unsupported channel count. It is
now an expectation. `render` and `flush` failures surface.

## Padding Is Bounded

The same function ended with `output.resize(planned, 0.0)`.

Unbounded zero-padding to a contracted length is the mechanism that let
`g10.039` ship three silent specimens: the renderer produced `3.8s` of audio and
the remaining `108s` was padding, which is why five structural gates passed it.

Rounding can leave a render a frame or two short, so padding is capped at `4`
samples and asserted. The whole render-plane suite passes with that bound, which
is evidence the renderer was not relying on padding rather than an assumption
that it wasn't.

## Note On A Gate Failure

The `test` gate failed once on `signal-plugin-sandbox` `plugin_hosting`, then
passed `15` consecutive runs. That binary is `A22`, closed earlier today by
serialising its child-spawning tests; nothing in this batch touches it. Recorded
rather than ignored, because a single failure in a binary with a known history
deserves the count next to it.

## Next Task

Open Batch 42.2, resumable pitch. The resumable renderer already carries phase,
detector, and overlap-add state across chunk boundaries; pitch composition adds
a resampler whose state must carry the same way.
