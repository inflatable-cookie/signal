use std::collections::VecDeque;

use rustfft::num_complex::Complex64;

use super::*;
use crate::frequency_adaptive::{
    material_state_frequency_frame::guided_frequency_partitioned_linked_phase::{
        Decision, StateCounts,
    },
    source_studied::faithful_predictor::linked_stereo::render::{
        StereoRender, TrackedPeakPhaseTrace,
    },
};

mod analysis;
mod guidance;
mod render;
mod report;

use analysis::SourceCache;
use guidance::{GuidanceState, Material};

type Frame = [Vec<Complex64>; CHANNEL_CAPACITY];

#[derive(Clone, Debug)]
struct CandidateRender {
    channels: [Vec<f64>; CHANNEL_CAPACITY],
    target_length: usize,
    uncovered: usize,
    non_finite: usize,
    boundary_failures: usize,
    states: StateCounts,
    maximum_live_source_slices: usize,
    maximum_live_output_slices: usize,
    maximum_guidance_frames: usize,
    hash: u64,
}

fn stereo_adapter(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> StereoRender {
    let rendered = render::render(inputs, ratio, sample_rate);
    StereoRender {
        channels: rendered.channels,
        uncovered: rendered.uncovered,
        non_finite: rendered.non_finite,
        boundary_failures: rendered.boundary_failures,
        shared_corrected: rendered.states.linked_regions,
        shared_fallback: rendered.states.unlinked_regions,
        unilateral_non_silent_completions: 0,
        reference_bins: [0; 2],
        active_reference_ties: 0,
        reference_switches: rendered.states.owner_switches,
        maximum_projected_relation_error: 0.0,
        maximum_constrained_relation_error: 0.0,
        synthesis_relation_trace: None,
        coefficient_contribution_trace: None,
        peak_region_counts: [
            rendered.states.linked_regions + rendered.states.unlinked_regions,
            rendered.states.states[Decision::Locked.index()],
            rendered.states.states[Decision::Reset.index()]
                + rendered.states.states[Decision::Attack.index()],
            0,
        ],
        tracked_peak_phase_trace: TrackedPeakPhaseTrace::default(),
        hash: rendered.hash,
    }
}
