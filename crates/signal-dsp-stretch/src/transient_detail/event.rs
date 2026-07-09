use crate::Sample;

use super::{amplitude_delta_db, local_crest_factor, refine_projected_onset};

/// Local timing and crest evidence for one source-projected transient event.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientEventDetail {
    /// Refined source event frame supplied by the caller.
    pub input_frame: usize,
    /// Refined output event frame near the ratio-projected source position.
    pub output_frame: usize,
    /// Signed output offset from the ratio-projected source frame.
    pub timing_offset_frames: f64,
    /// Output-versus-input local transient crest growth, in decibels.
    pub crest_growth_db: f64,
}

/// Measure one known source event against a stretched output.
///
/// `input_frame` must already be a refined source onset. Output refinement is
/// bounded to one `hop_size` around its ratio projection, matching the summary
/// transient-detail measurement.
pub fn measure_transient_event_detail(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    input_frame: usize,
    window_size: usize,
    hop_size: usize,
) -> Option<StretchTransientEventDetail> {
    if !ratio.is_finite()
        || ratio <= 0.0
        || input_frame >= input.len()
        || output.is_empty()
        || window_size < 16
        || hop_size == 0
    {
        return None;
    }

    let projected_frame = input_frame as f64 * ratio;
    let output_frame = refine_projected_onset(output, projected_frame.round() as usize, hop_size);
    let input_crest = local_crest_factor(input, input_frame, window_size / 2);
    let output_crest = local_crest_factor(output, output_frame, window_size / 2);
    Some(StretchTransientEventDetail {
        input_frame,
        output_frame,
        timing_offset_frames: output_frame as f64 - projected_frame,
        crest_growth_db: amplitude_delta_db(output_crest, input_crest),
    })
}
