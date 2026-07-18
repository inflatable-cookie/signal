use super::{render, SharedRotationRender, StateCounts};
use crate::frequency_adaptive::source_studied::faithful_predictor::{
    hash_samples, linked_stereo::render::StereoRender,
};

pub(in super::super) fn add_overlap(
    output: &mut [f64],
    normalization: &mut [f64],
    output_center: isize,
    frame: &[f64],
    window: &[f64],
    transform_length: usize,
) {
    for offset in 0..frame.len() {
        let output_index = output_center - frame.len() as isize / 2 + offset as isize;
        if (0..output.len() as isize).contains(&output_index) {
            let output_index = output_index as usize;
            output[output_index] += frame[offset] * window[offset] / transform_length as f64;
            normalization[output_index] += window[offset] * window[offset];
        }
    }
}

pub(in crate::frequency_adaptive) fn finish(
    channels: [Vec<f64>; 2],
    target_length: usize,
    uncovered: usize,
    states: StateCounts,
) -> SharedRotationRender {
    let non_finite = channels
        .iter()
        .flatten()
        .filter(|sample| !sample.is_finite())
        .count();
    let boundary_failures = channels
        .iter()
        .map(|channel| {
            usize::from(channel.first().is_none_or(|sample| !sample.is_finite()))
                + usize::from(channel.last().is_none_or(|sample| !sample.is_finite()))
        })
        .sum();
    let mut hash = hash_samples(&channels[0]);
    hash = (hash ^ hash_samples(&channels[1])).wrapping_mul(0x100_0000_01b3);
    SharedRotationRender {
        channels,
        target_length,
        uncovered,
        non_finite,
        boundary_failures,
        states,
        hash,
    }
}

pub(in super::super) fn stereo_adapter(
    inputs: [&[f64]; 2],
    ratio: f64,
    sample_rate: usize,
) -> StereoRender {
    let rendered = render(inputs, ratio, sample_rate);
    StereoRender {
        channels: rendered.channels,
        uncovered: rendered.uncovered,
        non_finite: rendered.non_finite,
        boundary_failures: rendered.boundary_failures,
        shared_corrected: 0,
        shared_fallback: 0,
        unilateral_non_silent_completions: 0,
        reference_bins: [0; 2],
        active_reference_ties: 0,
        reference_switches: 0,
        maximum_projected_relation_error: 0.0,
        maximum_constrained_relation_error: 0.0,
        synthesis_relation_trace: None,
        coefficient_contribution_trace: None,
        peak_region_counts: [
            rendered.states.regions,
            rendered.states.tracked,
            rendered.states.reset,
            rendered.states.silent,
        ],
        tracked_peak_phase_trace: Default::default(),
        hash: rendered.hash,
    }
}
