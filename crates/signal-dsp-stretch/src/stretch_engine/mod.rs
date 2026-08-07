//! Core stretch render helpers (ratio sanitization, dynamic segments, pitch).

mod dynamic_ratio;
mod limits;
mod math;
mod pitch_window;
mod render;

#[cfg(any(test, feature = "evidence"))]
pub(crate) use dynamic_ratio::dynamic_ratio_output_boundaries;
#[allow(unused_imports)]
// re-exported for sibling modules and unit tests via `crate::stretch_engine`
pub(crate) use dynamic_ratio::{
    coalesce_short_dynamic_ratio_segments, dynamic_ratio_output_frames, dynamic_ratio_segment,
    dynamic_ratio_segment_boundaries, dynamic_ratio_segments, min_dynamic_ratio_segment_frames,
    smooth_dynamic_segment_boundaries_interleaved, DynamicRatioSegment,
};
#[allow(unused_imports)]
pub(crate) use limits::{
    checked_output_frames, checked_target_frames, DYNAMIC_RATIO_SEAM_SMOOTH_FRAMES,
    MIN_DYNAMIC_RATIO_SEGMENT_EXTRA_HOPS,
};
#[allow(unused_imports)]
// re-exported for sibling modules and unit tests via `crate::stretch_engine`
pub use limits::{StretchRenderError, MAX_OFFLINE_STRETCH_OUTPUT_SAMPLES};
#[allow(unused_imports)]
pub(crate) use math::{
    abs_diff_frames, align_to_next_grid, ceil_frame_to_u64, ceil_frame_to_usize,
    floor_frame_to_u64, linear_time_scale, linear_time_scale_interleaved_stereo, sanitize_ratio,
    saturating_u128, usize_to_u64, wrap_phase,
};
#[allow(unused_imports)]
pub(crate) use pitch_window::{
    downmix_interleaved_stereo_to_mono, metric_worsened,
    pitch_shift_interleaved_stereo_to_nominal_rate, pitch_shift_mono_to_nominal_rate,
    pitch_shift_resample_config, short_window_analysis_hop_for_path, short_window_size_for_path,
    should_select_compression_short_window, should_select_compression_short_window_interleaved,
    should_select_expansion_short_window, should_select_expansion_short_window_interleaved,
};
pub(crate) use render::{
    stretch_dynamic_ratio_linked_stereo_with_engine, stretch_dynamic_ratio_mono_with_engine,
    stretch_mono_with_engine, stretch_to_exact_linked_stereo, stretch_to_exact_mono,
};
