# 2026-07-27 g10.039 Batch 39.4 Render Plane Adoption

Status: implemented and structurally proven; listening evidence owed

## What Changed

The offline artifact path now renders through one `ResumableOfflineStretch`
driven across the chunk plan, instead of constructing a fresh
`OfflineHighQualityStretcher` per chunk and patching the joins.

The chunk plan keeps its job: it still bounds how much source is in flight. It
no longer cuts the render into independent pieces.

## Chunk Independence In The Artifact Path

Sixty seconds of stereo material at ratio `1.25`, rendered twice through the
artifact path with different chunk policies:

| comparison | result |
| --- | --- |
| `2`-chunk against `12`-chunk, both resumable | **bit-identical** |
| correlation | `1.000000` |

Batch 39.1 measured the same comparison on the shipped path at `0.389976`.

## A Wrong Reading, Corrected Mid-Batch

The first probe compared the adopted path against a "whole-buffer control" and
reported correlation `0.028056`, which looked like the adoption had failed.

It had not. The control took the `is_single_chunk` branch, which still used the
old whole-buffer renderer, so the probe was comparing two different algorithms
rather than two chunk policies of one. The real control is two multi-chunk
resumable renders, and those are bit-identical.

That mistake exposed a genuine defect in the first wiring. Routing only
multi-chunk artifacts through the resumable renderer meant **source length
selected the algorithm**: a short artifact and a long artifact of the same
source would render through different code while sharing one cache key. Every
supported configuration now takes the resumable path regardless of chunk count.

## Adoption Is Partial, So The Smoothers Stay

The resumable renderer owns the default offline path with no pitch shift.
Selector paths and pitch composition still route through the legacy per-chunk
path, because the resumable renderer does not implement them.

The batch task was to remove both seam smoothers "once measurement shows they
are no longer needed". Measurement shows they are still needed: the legacy
branch still creates the boundaries they patch.
`smooth_artifact_chunk_boundaries_interleaved` and the crate's internal
`smooth_dynamic_segment_boundaries_interleaved` are retained, and removal moves
to Batch 39.5 conditional on the remaining paths being adopted.

## Behavior Version

Adoption changes rendered output for every artifact longer than one chunk, so
`SIGNAL_STRETCH_BEHAVIOR_VERSION` advances to
`signal-stretch-behavior-2026-07-27-resumable`, per the Contract `046` rule that
it moves in the same change as the output. Artifacts written before this change
are correctly invalidated.

The corpus report is unchanged, which is expected: it measures the crate's own
renderers, not the render-plane artifact path.

## What Is Not Proven

No listening evidence exists. Output changed against the shipped path, so
Contract `084` Rule 5 applies and this cannot be called admitted until a
concealed pack is judged. The comparison that matters is the adopted artifact
path against the current shipped artifact path on material longer than one
chunk, where the seam pulse from the `g10.036` rounds should be gone.

`segmented_render_matches_whole_render_at_constant_ratio` in
`transparent_correctness_owners` stays `#[ignore]`d. It measures the crate's own
`stretch_dynamic_ratio_mono`, which is untouched; only the artifact path
adopted the new renderer.

## New Finding: A21

`fake_clocked_soak` `clocked_soak_advances_health_counters_and_meters` failed
once under a three-crate parallel run and passed three times in isolation, and
`signal-render-plane` passes cleanly on its own. The test sleeps for three block
durations to provoke an xrun and then for `1500 ms`, so it is wall-clock
dependent in the same way as `A20`.

Not attributed to this batch: the change touches artifact rendering, not clock
or callback health.

## Validation Run

- artifact chunk-independence probe, in-crate, removed after use
- `cargo test -p signal-render-plane`: green alone
- `cargo test -p signal-dsp-stretch`: green alone
- `cargo build --workspace --all-targets`
- `cargo clippy --workspace --all-targets --all-features`: pre-existing warning
  set unchanged
- corpus report unchanged
- `effigy qa:docs`

## Next Task

Build the concealed listening pack for `g10.039`: the adopted artifact path
against the shipped path, on material longer than one chunk, so the seam pulse
heard in the `g10.036` rounds can be judged directly. Admission under Contract
`084` Rule 5 waits on that judgement.

Batch 39.5 then closes the lane and decides whether the remaining offline paths
adopt the resumable renderer, which is what would let both seam smoothers be
deleted.
