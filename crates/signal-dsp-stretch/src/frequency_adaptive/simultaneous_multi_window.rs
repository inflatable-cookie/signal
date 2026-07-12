use rustfft::{num_complex::Complex64, FftPlanner};

use super::HASH_OFFSET;
use support::{controls, hash, reflected, review_hash, window};

mod support;

const SOURCE_FRAMES: usize = 16_384;
const GUARD: isize = 8_192;
const LAYERS: [usize; 3] = [512, 2_048, 8_192];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Direction {
    StudyAndScheduleProof,
    UnionRedesign,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Review {
    pub layer_lengths: [usize; 3],
    pub layer_frame_counts: [usize; 3],
    pub coefficient_counts: [usize; 3],
    pub work_counts: [usize; 3],
    pub frame_bounds: [f64; 3],
    pub structural_failures: [usize; 3],
    pub maximum_errors: [f64; 4],
    pub non_finite_values: usize,
    pub reflected_reads: usize,
    pub empty_input_exact: bool,
    pub hashes: [u64; 6],
    pub direction: Direction,
}

#[derive(Clone, Copy)]
struct Frame {
    layer: usize,
    start: isize,
}

pub(super) fn review() -> Review {
    let domain_start = -GUARD;
    let domain_end = SOURCE_FRAMES as isize + GUARD;
    let domain_len = (domain_end - domain_start) as usize;
    let frames = frames(domain_start, domain_end);
    let mut operator = vec![0.0; domain_len];
    let mut layer_frame_counts = [0; 3];
    let mut reflected_reads = 0;
    let mut schedule_hash = HASH_OFFSET;
    let mut window_hash = HASH_OFFSET;
    for frame in &frames {
        let length = LAYERS[frame.layer];
        layer_frame_counts[frame.layer] += 1;
        hash(&mut schedule_hash, frame.layer as u64);
        hash(&mut schedule_hash, frame.start as i64 as u64);
        for (offset, value) in window(length).into_iter().enumerate() {
            hash(&mut window_hash, value.to_bits());
            let logical = frame.start + offset as isize;
            reflected_reads += usize::from(logical < 0 || logical >= SOURCE_FRAMES as isize);
            if (domain_start..domain_end).contains(&logical) {
                operator[(logical - domain_start) as usize] += value * value;
            }
        }
    }
    let source = GUARD as usize..GUARD as usize + SOURCE_FRAMES;
    let minimum = operator[source.clone()]
        .iter()
        .copied()
        .fold(f64::INFINITY, f64::min);
    let maximum = operator[source.clone()].iter().copied().fold(0.0, f64::max);
    let uncovered_domain = operator.iter().filter(|value| **value <= 0.0).count();
    let uncovered_source = operator[source]
        .iter()
        .filter(|value| **value <= 0.0)
        .count();
    let mut dual_hash = HASH_OFFSET;
    for value in &operator {
        hash(&mut dual_hash, value.recip().to_bits());
    }

    let mut maximum_errors = [0.0_f64; 4];
    let mut non_finite_values = 0;
    let mut coefficient_hash = HASH_OFFSET;
    let mut output_hash = HASH_OFFSET;
    for control in controls() {
        let (errors, non_finite) = reconstruct(
            &control,
            &frames,
            &operator,
            domain_start,
            &mut coefficient_hash,
            &mut output_hash,
        );
        for (maximum, error) in maximum_errors.iter_mut().zip(errors) {
            *maximum = maximum.max(error);
        }
        non_finite_values += non_finite;
    }
    let coefficient_counts = std::array::from_fn(|layer| layer_frame_counts[layer] * LAYERS[layer]);
    let structural_failures = [uncovered_domain, uncovered_source, 0];
    let pass = structural_failures == [0; 3]
        && maximum / minimum <= 1.000_001
        && maximum_errors[0] <= 2.0e-12
        && maximum_errors[1] <= 2.0e-12
        && maximum_errors[2] <= 2.0e-10
        && maximum_errors[3] <= 2.0e-10
        && non_finite_values == 0;
    let mut empty_coefficient_hash = HASH_OFFSET;
    let mut empty_output_hash = HASH_OFFSET;
    let empty_input_exact = reconstruct(
        &[],
        &frames,
        &operator,
        domain_start,
        &mut empty_coefficient_hash,
        &mut empty_output_hash,
    )
    .0 == [0.0; 4];
    let mut result = Review {
        layer_lengths: LAYERS,
        layer_frame_counts,
        coefficient_counts,
        work_counts: coefficient_counts,
        frame_bounds: [minimum, maximum, maximum / minimum],
        structural_failures,
        maximum_errors,
        non_finite_values,
        reflected_reads,
        empty_input_exact,
        hashes: [
            schedule_hash,
            window_hash,
            dual_hash,
            coefficient_hash,
            output_hash,
            0,
        ],
        direction: if pass {
            Direction::StudyAndScheduleProof
        } else {
            Direction::UnionRedesign
        },
    };
    result.hashes[5] = review_hash(&result);
    result
}

fn frames(domain_start: isize, domain_end: isize) -> Vec<Frame> {
    let mut result = Vec::new();
    for (layer, length) in LAYERS.into_iter().enumerate() {
        let hop = (length / 4) as isize;
        let mut start = domain_start - length as isize + hop;
        while start < domain_end {
            result.push(Frame { layer, start });
            start += hop;
        }
    }
    result
}

fn reconstruct(
    input: &[f64],
    frames: &[Frame],
    operator: &[f64],
    domain_start: isize,
    coefficient_hash: &mut u64,
    output_hash: &mut u64,
) -> ([f64; 4], usize) {
    let mut planner = FftPlanner::<f64>::new();
    let mut output = vec![Complex64::new(0.0, 0.0); operator.len()];
    let mut symmetry = 0.0_f64;
    let mut non_finite = 0;
    for frame in frames {
        let length = LAYERS[frame.layer];
        let window = window(length);
        let mut buffer = window
            .iter()
            .enumerate()
            .map(|(offset, value)| {
                Complex64::new(reflected(input, frame.start + offset as isize) * value, 0.0)
            })
            .collect::<Vec<_>>();
        planner.plan_fft_forward(length).process(&mut buffer);
        for bin in 0..length {
            let mirror = if bin == 0 { 0 } else { length - bin };
            symmetry = symmetry.max((buffer[bin] - buffer[mirror].conj()).norm());
            non_finite += usize::from(!buffer[bin].re.is_finite() || !buffer[bin].im.is_finite());
            hash(coefficient_hash, buffer[bin].re.to_bits());
            hash(coefficient_hash, buffer[bin].im.to_bits());
        }
        planner.plan_fft_inverse(length).process(&mut buffer);
        for (offset, (sample, value)) in buffer.into_iter().zip(window).enumerate() {
            let logical = frame.start + offset as isize;
            if let Some(index) = logical
                .checked_sub(domain_start)
                .and_then(|v| usize::try_from(v).ok())
                .filter(|v| *v < output.len())
            {
                output[index] += sample * (value / (length as f64 * operator[index]));
            }
        }
    }
    let mut peak = 0.0_f64;
    let mut rms = 0.0_f64;
    let mut imaginary = 0.0_f64;
    for (index, expected) in input.iter().copied().enumerate() {
        let actual = output[index + GUARD as usize];
        peak = peak.max((actual.re - expected).abs());
        rms += (actual.re - expected).powi(2);
        imaginary = imaginary.max(actual.im.abs());
        hash(output_hash, actual.re.to_bits());
    }
    let rms = if input.is_empty() {
        0.0
    } else {
        (rms / input.len() as f64).sqrt()
    };
    [peak, rms, symmetry, imaginary]
        .into_iter()
        .for_each(|v| non_finite += usize::from(!v.is_finite()));
    ([peak, rms, symmetry, imaginary], non_finite)
}
