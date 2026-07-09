use crate::{
    detect_stretch_transients_with_policy, Sample, StretchTransientDetectorPolicy,
    StretchTransientEvent,
};

mod event;

pub use event::{measure_transient_event_detail, StretchTransientEventDetail};

/// Fine-grained timing and peak-shape evidence for matched stretch transients.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientDetailMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Number of detected input transient candidates.
    pub input_transients: usize,
    /// Number of detected output transient candidates.
    pub output_transients: usize,
    /// Number of uniquely matched input/output transient pairs.
    pub matched_transients: usize,
    /// Mean signed output timing offset after ratio projection, in frames.
    pub mean_signed_timing_offset_frames: f64,
    /// Mean absolute output timing offset after ratio projection, in frames.
    pub mean_absolute_timing_offset_frames: f64,
    /// Largest absolute output timing offset, in frames.
    pub max_absolute_timing_offset_frames: f64,
    /// Refined input onset for the largest absolute timing offset.
    pub max_timing_input_frame: usize,
    /// Refined output onset for the largest absolute timing offset.
    pub max_timing_output_frame: usize,
    /// Largest output-versus-input local transient crest growth, in decibels.
    pub max_transient_crest_growth_db: f64,
    /// Refined input onset for the largest transient crest growth.
    pub max_crest_input_frame: usize,
    /// Refined output onset for the largest transient crest growth.
    pub max_crest_output_frame: usize,
}

/// Measure event-level transient placement and local crest growth.
///
/// Spectral detection supplies coarse candidates. Each matched onset is then
/// refined at sample-frame resolution from the strongest short-time energy
/// rise. Output candidates are consumed once so one attack cannot satisfy
/// several projected source events. Crest growth compares peak-to-RMS ratios,
/// so whole-render gain differences do not masquerade as transient spikes.
pub fn measure_transient_detail(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
) -> StretchTransientDetailMeasurement {
    if !ratio.is_finite()
        || ratio <= 0.0
        || input.is_empty()
        || output.is_empty()
        || window_size < 16
        || hop_size == 0
    {
        return invalid_measurement(ratio);
    }

    let input_events = detect_stretch_transients_with_policy(
        input,
        window_size,
        hop_size,
        StretchTransientDetectorPolicy::production(),
    );
    let output_events = detect_stretch_transients_with_policy(
        output,
        window_size,
        hop_size,
        StretchTransientDetectorPolicy::candidate_review(),
    );
    let mut output_used = vec![false; output_events.len()];
    let tolerance = window_size.max(hop_size * 4) as f64;
    let mut signed_offset_sum = 0.0;
    let mut absolute_offset_sum = 0.0;
    let mut matched = 0usize;
    let mut max_absolute_offset = -1.0f64;
    let mut max_timing_input_frame = 0usize;
    let mut max_timing_output_frame = 0usize;
    let mut max_crest_growth_db = f64::NEG_INFINITY;
    let mut max_crest_input_frame = 0usize;
    let mut max_crest_output_frame = 0usize;

    for input_event in &input_events {
        let expected_output_frame = input_event.frame_index as f64 * ratio;
        let Some(output_index) = nearest_unused_event(
            &output_events,
            &output_used,
            expected_output_frame,
            tolerance,
        ) else {
            continue;
        };
        output_used[output_index] = true;

        let input_frame = refine_onset(input, input_event.frame_index, window_size, hop_size);
        let projected_input_frame = input_frame as f64 * ratio;
        let output_frame =
            refine_projected_onset(output, projected_input_frame.round() as usize, hop_size);
        let signed_offset = output_frame as f64 - input_frame as f64 * ratio;
        let absolute_offset = signed_offset.abs();
        signed_offset_sum += signed_offset;
        absolute_offset_sum += absolute_offset;
        matched += 1;

        if absolute_offset > max_absolute_offset {
            max_absolute_offset = absolute_offset;
            max_timing_input_frame = input_frame;
            max_timing_output_frame = output_frame;
        }

        let input_crest = local_crest_factor(input, input_frame, window_size / 2);
        let output_crest = local_crest_factor(output, output_frame, window_size / 2);
        let crest_growth_db = amplitude_delta_db(output_crest, input_crest);
        if crest_growth_db > max_crest_growth_db {
            max_crest_growth_db = crest_growth_db;
            max_crest_input_frame = input_frame;
            max_crest_output_frame = output_frame;
        }
    }

    if matched == 0 {
        return StretchTransientDetailMeasurement {
            ratio,
            input_transients: input_events.len(),
            output_transients: output_events.len(),
            matched_transients: 0,
            ..invalid_measurement(ratio)
        };
    }

    StretchTransientDetailMeasurement {
        ratio,
        input_transients: input_events.len(),
        output_transients: output_events.len(),
        matched_transients: matched,
        mean_signed_timing_offset_frames: signed_offset_sum / matched as f64,
        mean_absolute_timing_offset_frames: absolute_offset_sum / matched as f64,
        max_absolute_timing_offset_frames: max_absolute_offset,
        max_timing_input_frame,
        max_timing_output_frame,
        max_transient_crest_growth_db: max_crest_growth_db,
        max_crest_input_frame,
        max_crest_output_frame,
    }
}

fn nearest_unused_event(
    events: &[StretchTransientEvent],
    used: &[bool],
    expected_frame: f64,
    tolerance: f64,
) -> Option<usize> {
    events
        .iter()
        .enumerate()
        .filter(|(index, _)| !used[*index])
        .map(|(index, event)| (index, (event.frame_index as f64 - expected_frame).abs()))
        .filter(|(_, distance)| *distance <= tolerance)
        .min_by(|left, right| left.1.total_cmp(&right.1))
        .map(|(index, _)| index)
}

fn refine_onset(
    samples: &[Sample],
    coarse_frame: usize,
    window_size: usize,
    hop_size: usize,
) -> usize {
    const ENERGY_SPAN: usize = 16;
    let search_start = coarse_frame.saturating_sub(hop_size);
    let search_end = coarse_frame
        .saturating_add(window_size)
        .saturating_add(hop_size)
        .min(samples.len().saturating_sub(ENERGY_SPAN));
    if search_end <= search_start.saturating_add(ENERGY_SPAN) {
        return coarse_frame.min(samples.len().saturating_sub(1));
    }

    let mut best_frame = search_start + ENERGY_SPAN;
    let mut best_rise = f64::NEG_INFINITY;
    for frame in search_start + ENERGY_SPAN..search_end {
        let before = mean_square(&samples[frame - ENERGY_SPAN..frame]);
        let after = mean_square(&samples[frame..frame + ENERGY_SPAN]);
        let rise = after - before;
        if rise > best_rise {
            best_rise = rise;
            best_frame = frame;
        }
    }
    best_frame
}

fn refine_projected_onset(samples: &[Sample], projected_frame: usize, radius: usize) -> usize {
    const ENERGY_SPAN: usize = 16;
    let search_start = projected_frame.saturating_sub(radius).max(ENERGY_SPAN);
    let search_end = projected_frame
        .saturating_add(radius)
        .min(samples.len().saturating_sub(ENERGY_SPAN));
    if search_end <= search_start {
        return projected_frame.min(samples.len().saturating_sub(1));
    }

    let mut best_frame = projected_frame.clamp(search_start, search_end);
    let mut best_rise = f64::NEG_INFINITY;
    for frame in search_start..=search_end {
        let before = mean_square(&samples[frame - ENERGY_SPAN..frame]);
        let after = mean_square(&samples[frame..frame + ENERGY_SPAN]);
        let rise = after - before;
        if rise > best_rise {
            best_rise = rise;
            best_frame = frame;
        }
    }
    best_frame
}

fn local_crest_factor(samples: &[Sample], center: usize, radius: usize) -> f64 {
    let start = center.saturating_sub(radius);
    let end = center.saturating_add(radius).min(samples.len());
    let region = &samples[start..end];
    if region.is_empty() {
        return f64::NAN;
    }
    let peak = region
        .iter()
        .map(|sample| sample.abs() as f64)
        .fold(0.0, f64::max);
    let rms = mean_square(region).sqrt();
    peak / (rms + 1.0e-12)
}

fn mean_square(samples: &[Sample]) -> f64 {
    samples
        .iter()
        .map(|sample| {
            let sample = *sample as f64;
            sample * sample
        })
        .sum::<f64>()
        / samples.len().max(1) as f64
}

fn amplitude_delta_db(output: f64, input: f64) -> f64 {
    20.0 * ((output + 1.0e-12) / (input + 1.0e-12)).log10()
}

fn invalid_measurement(ratio: f64) -> StretchTransientDetailMeasurement {
    StretchTransientDetailMeasurement {
        ratio,
        input_transients: 0,
        output_transients: 0,
        matched_transients: 0,
        mean_signed_timing_offset_frames: f64::NAN,
        mean_absolute_timing_offset_frames: f64::NAN,
        max_absolute_timing_offset_frames: f64::NAN,
        max_timing_input_frame: 0,
        max_timing_output_frame: 0,
        max_transient_crest_growth_db: f64::NAN,
        max_crest_input_frame: 0,
        max_crest_output_frame: 0,
    }
}

#[cfg(test)]
#[path = "transient_detail/tests.rs"]
mod tests;
