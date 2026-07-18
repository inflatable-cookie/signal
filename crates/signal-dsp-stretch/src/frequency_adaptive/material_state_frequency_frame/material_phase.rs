use std::{fs, path::PathBuf};

use rustfft::{num_complex::Complex64, FftPlanner};

use super::{
    absolute_bin, build_representation_for, local_coefficient, reflected_sample, Band,
    Representation, Scale, CROSSOVER_HZ, HASH_OFFSET, PAD_FRAMES, SUPPORT_FRAMES,
};
use crate::frequency_adaptive::source_studied::faithful_predictor::linked_stereo::{
    quality::gate_calibration::{
        peak_region_feasibility::{self, PeakRegionReview},
        shared_rotation_region_locked_proof::{
            corpus, mechanics, SharedRotationCorpusReview, SharedRotationMechanicsReview,
        },
    },
    render::StereoRender,
    shared_rotation_region_locked::{
        output::finish, phase::regions, SharedRotationRender, StateCounts,
    },
};

const COMMON_HOP: usize = 512;
const RATIOS: [f64; 3] = [0.75, 1.5, 2.0];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Material {
    tonalness: f64,
    noisiness: f64,
    transientness: f64,
}

#[derive(Clone)]
struct Analysis {
    representation: Representation,
    coefficients: [Vec<Vec<Complex64>>; 2],
    material: Vec<Vec<Material>>,
    transient_centers: Vec<bool>,
}

#[derive(Clone)]
struct RegionMemory {
    first: usize,
    end: usize,
    owner: usize,
    rotation: f64,
    phase: [f64; 2],
    energy: [f64; 2],
    frequency: f64,
}

mod analysis;
mod phase;
mod report;
mod synthesis;

use analysis::analyse;
use phase::{analysis_state_counts, transport};
use synthesis::synthesise;

fn render(inputs: [&[f64]; 2], ratio: f64, sample_rate: usize) -> SharedRotationRender {
    assert_eq!(inputs[0].len(), inputs[1].len(), "linked channel lengths");
    assert!(!inputs[0].is_empty(), "non-empty linked input");
    assert!(ratio.is_finite() && ratio > 0.0, "positive finite ratio");
    if ratio == 1.0 {
        return finish(
            [inputs[0].to_vec(), inputs[1].to_vec()],
            inputs[0].len(),
            0,
            StateCounts::default(),
        );
    }

    let target_length = (inputs[0].len() as f64 * ratio).round() as usize;
    let analysis = analyse(inputs, sample_rate);
    let output_fft_frames = padded_frames(target_length);
    let output_representation =
        build_representation_for(output_fft_frames, sample_rate, COMMON_HOP);
    assert_compatible(&analysis.representation, &output_representation);
    let output_coefficients = transport(&analysis, &output_representation, ratio, target_length);
    let (channels, non_finite) =
        synthesise(&output_representation, output_coefficients, target_length);
    let mut rendered = finish(channels, target_length, 0, analysis_state_counts(&analysis));
    rendered.non_finite += non_finite;
    rendered
}

fn padded_frames(content_frames: usize) -> usize {
    (content_frames + PAD_FRAMES * 2)
        .next_power_of_two()
        .max(SUPPORT_FRAMES[0] * 4)
}

fn assert_compatible(input: &Representation, output: &Representation) {
    assert_eq!(
        input.bands.len(),
        output.bands.len(),
        "frequency atom count"
    );
    assert!(
        input.bands.iter().zip(&output.bands).all(|(left, right)| {
            left.scale == right.scale
                && absolute_bin(left.center, input.fft_frames) * output.fft_frames
                    == absolute_bin(right.center, output.fft_frames) * input.fft_frames
        }),
        "frequency atom layout"
    );
}

fn deterministic_unit(time: usize, scale: Scale, band: usize) -> f64 {
    let mut hash = HASH_OFFSET;
    for value in [time as u64, scale.index() as u64, band as u64] {
        hash = (hash ^ value).wrapping_mul(0x100_0000_01b3);
        hash ^= hash >> 32;
    }
    (hash as f64 / u64::MAX as f64) * 2.0 - 1.0
}

fn wrap(value: f64) -> f64 {
    (value + std::f64::consts::PI).rem_euclid(std::f64::consts::TAU) - std::f64::consts::PI
}
