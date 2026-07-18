use std::collections::VecDeque;

use rustfft::{num_complex::Complex64, FftPlanner};

use super::{
    build_representation_for, hash_u64, local_coefficient, reflected_sample, Representation,
    FFT_FRAMES, HASH_OFFSET, SUPPORT_FRAMES,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    render::{StereoRender, TrackedPeakPhaseTrace},
    shared_rotation_region_locked::{output::finish, SharedRotationRender, StateCounts},
};

mod analysis;
mod phase;
mod relation;
mod render;
mod report;

const COMMON_HOP: usize = 512;
const OUTER_ADVANCE: usize = FFT_FRAMES / 2;
const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct RelationCounts {
    two_defined: usize,
    one_defined: usize,
    undefined: usize,
    zero_peer: usize,
    silent: usize,
}

impl RelationCounts {
    fn add(&mut self, other: Self) {
        self.two_defined += other.two_defined;
        self.one_defined += other.one_defined;
        self.undefined += other.undefined;
        self.zero_peer += other.zero_peer;
        self.silent += other.silent;
    }

    fn as_array(self) -> [usize; 5] {
        [
            self.two_defined,
            self.one_defined,
            self.undefined,
            self.zero_peer,
            self.silent,
        ]
    }
}

#[derive(Clone, Debug)]
struct CandidateRender {
    render: SharedRotationRender,
    relations: RelationCounts,
    maximum_relation_error: f64,
    maximum_live_source_slices: usize,
    maximum_live_output_slices: usize,
}

fn render(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> SharedRotationRender {
    render::render_detailed(inputs, ratio, sample_rate).render
}

fn stereo_adapter(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> StereoRender {
    let result = render::render_detailed(inputs, ratio, sample_rate);
    let counts = result.relations.as_array();
    let rendered = result.render;
    StereoRender {
        channels: rendered.channels,
        uncovered: rendered.uncovered,
        non_finite: rendered.non_finite,
        boundary_failures: rendered.boundary_failures,
        shared_corrected: counts[0],
        shared_fallback: counts[1],
        unilateral_non_silent_completions: counts[2],
        reference_bins: [counts[3], counts[4]],
        active_reference_ties: 0,
        reference_switches: 0,
        maximum_projected_relation_error: result.maximum_relation_error,
        maximum_constrained_relation_error: result.maximum_relation_error,
        synthesis_relation_trace: None,
        coefficient_contribution_trace: None,
        peak_region_counts: [
            rendered.states.regions,
            rendered.states.tracked,
            rendered.states.reset,
            rendered.states.silent,
        ],
        tracked_peak_phase_trace: TrackedPeakPhaseTrace::default(),
        hash: rendered.hash,
    }
}

fn outer_window() -> Vec<f64> {
    (0..FFT_FRAMES)
        .map(|index| (std::f64::consts::PI * (index as f64 + 0.5) / FFT_FRAMES as f64).sin())
        .collect()
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
