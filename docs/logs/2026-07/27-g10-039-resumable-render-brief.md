# 2026-07-27 g10.039 Batch 39.2 Resumable Render Brief

Status: complete

Documentation only. No crate source changed.

## API Shape

```rust
pub struct ResumableOfflineStretch { /* private */ }

impl ResumableOfflineStretch {
    pub fn new(config: ResumableStretchConfig) -> Result<Self, StretchRenderError>;
    pub fn render(&mut self, source: &[Sample], output: &mut [Sample])
        -> Result<ResumableRenderReport, StretchRenderError>;
    pub fn flush(&mut self, output: &mut [Sample])
        -> Result<ResumableRenderReport, StretchRenderError>;
    pub fn reset(&mut self);
}
```

`ResumableStretchConfig` carries sample rate, channel count, window size,
analysis hop, offline path, the ratio curve, and the static pitch shift. The
curve belongs to the config because Batch 39.1 decided the renderer owns it.

`render` consumes one source chunk and writes whatever output is available.
Output is not one-to-one with input: at ratio `2.0` a chunk produces roughly
twice its frames, and the renderer holds up to one window of source it has not
yet synthesized. `flush` drains that tail at end of source. `reset` returns the
state to construction without reallocating.

The report carries frames consumed, frames produced, and the cumulative source
and output positions, so a caller can drive the chunk plan without tracking
ratio arithmetic itself.

## Carried State

Six items persist across `render` calls. The `frame_index == 0` branch becomes
a construction-time condition rather than a per-call one, which is what
actually removes the join:

| state | shape |
| --- | --- |
| `synthesis_phase` | `bins` per channel |
| `previous_phase` | `bins` per channel |
| `previous_magnitudes` | `bins` per channel |
| `previous_energy` | scalar per channel |
| overlap-add ring | `2 * window` per channel |
| normalization ring | `2 * window` per channel |

## Memory Ceiling

Every item is derived from window size, bin count, and channel count. Nothing
references source or output length.

| geometry | shared | per channel | total, stereo |
| --- | --- | --- | --- |
| `2048`, the retained default | `45060 B` | `102436 B` | `249932 B`, `0.238 MiB` |
| `65536`, the frozen maximum | `1441796 B` | `3276836 B` | `7995468 B`, `7.625 MiB` |

Ceiling: `8 MiB`, with `393140 B` of headroom at the maximum geometry.

The maximum window is new. `with_window` clamps only to a power of two at or
above `64`, with no upper limit, so "bounded by geometry" was not a number
until this brief froze one.

Duration-independence is structural, not incidental: the current whole-render
overlap-add and normalization buffers are the only duration-dependent state,
and the rings replace them. A candidate that still sizes any buffer from source
length has not met the bound, regardless of what it measures.

## Equivalence Law

For any partition of a source into chunks, the concatenated output must be
bit-identical to a single `render` call over the whole source, followed by
`flush`.

Exact, not tolerance-bounded. Batch 39.1 froze this: a tolerance readmits the
defect the lane exists to remove.

## Evidence Order

Contract `084` Rule 4 order applies. Compile, then construction proving the
geometry and capacity facts, then structural proving state carry, chunk-size
independence, and the memory ceiling, then synthetic comparing against a
whole-buffer control, then concealed listening.

Rejection is terminal per Rule 11: a candidate that fails a gate is deleted, not
repaired and rerun. Conformance iteration before the acoustic checkpoint is
allowed; iteration after it is not.

## Which `g10.036` Controls Survive

This is the question Batch 39.1 required answering before implementation opens.

Survive unchanged:

| owner | why |
| --- | --- |
| `overlap_safe_analysis_hop_*`, three owners | pure function of geometry, no renderer involvement |
| `phase_vocoder_bit_exact_baseline` | single whole-buffer static-ratio render, which has no joins to carry state across |
| `overlap_law_leaves_low_ratios_byte_exact` | asserts determinism and length, not sample values |
| `overlap_coverage_has_no_zeroed_interior_block` | behavior property, must continue to hold |
| `overlap_ripple_stays_within_ceiling` | behavior property, must continue to hold |
| `oversized_output_request_is_refused` and the two bound owners | API contract, unaffected |
| `dense_ratio_curve_preserves_pitch` | must continue to hold, and should improve |

Change meaning:

| owner | change |
| --- | --- |
| `segment_coalescing_preserves_total_output_length` | coalescing disappears when the renderer owns the curve. The length law survives; the coalescing framing does not |
| `sub_minimum_curve_spans_render_at_their_mean_ratio` | records a cost of coalescing. If coalescing goes, so does the cost, and the owner is removed rather than re-baselined |
| `dynamic_ratio_seam_click_matches_across_channel_counts` | there are no seams to measure. Superseded by the equivalence law |

Must flip to passing:

| owner | current state |
| --- | --- |
| `segmented_render_matches_whole_render_at_constant_ratio` | `#[ignore]`d, asserts `0.99` correlation, currently `0.034` |

Must be re-baselined: any dynamic-ratio output hash, and the chunked artifact
path in `signal-render-plane`.

## Cleanup And Rejection

The candidate is implemented once in an isolated worktree per Contract `084`
Rule 2. On rejection the worktree, branch, checkpoint reference, source, tests,
and build state are deleted, and no candidate code enters `main`.

On admission, Batch 39.4 removes `smooth_dynamic_segment_boundaries_interleaved`
from the crate and `smooth_artifact_chunk_boundaries_interleaved` from the
render plane. Contract `046` already states the test: if either is still needed,
the state is not being carried and the work is incomplete.

## Validation Run

- state inventory computed from geometry at both the retained and maximum
  window sizes
- `effigy qa:docs`

## Next Task

Execute `g10.039` Batch 39.3: implement the frozen brief once in an isolated
worktree, prove chunk-size independence across at least three chunk sizes,
prove the memory ceiling holds for long sources, prove dynamic-ratio
transitions carry state, and measure seam artifacts against the Batch 39.1
baseline.
