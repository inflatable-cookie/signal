use rustfft::{num_complex::Complex, FftPlanner};

use super::types::{
    StretchDensePainlessDirection as Direction, StretchDensePainlessReview as Review,
};
use super::{build_bands, hash_u32, hash_usize, Band, HASH_OFFSET};

const FFT_FRAMES: usize = 65_536;
const SAMPLE_RATE_HZ: f64 = 48_000.0;
const LOCALIZATION_CAP: usize = 16_384;
const LOCALIZATION_THRESHOLD: f64 = 1.0e-12;

pub(crate) fn dense_painless_common_lattice_review() -> Review {
    let bands = build_bands(FFT_FRAMES, SAMPLE_RATE_HZ);
    let common_coefficients = bands
        .iter()
        .map(|band| band.coefficient_frames)
        .max()
        .unwrap_or(1);
    let common_hop = FFT_FRAMES / common_coefficients;
    let unequal_coefficients = bands
        .iter()
        .map(|band| band.coefficient_frames)
        .sum::<usize>();
    let dense_coefficients = common_coefficients * bands.len();

    let (frame_operator, coverage) = frame_operator(&bands);
    let frame_min = frame_operator.iter().copied().fold(f64::INFINITY, f64::min);
    let frame_max = frame_operator.iter().copied().fold(0.0_f64, f64::max);
    let filter_hash = filter_hash(&bands);
    let frame_hash = scalar_hash(&frame_operator);
    let dual_hash = dual_hash(&bands, &frame_operator);
    let closure_error = real_spectrum_closure(&bands);
    let support_violations = bands
        .iter()
        .filter(|band| band.taps.len() > common_coefficients)
        .count();

    let (reconstruction_errors, non_finite_reconstruction, reconstruction_hash) =
        reconstruct(&bands, &frame_operator, common_coefficients);
    let (localization_radii, localization_curves, required_radii, limiting_bands, atom_hash) =
        localization(&bands, &frame_operator, common_hop);

    let structural_failures = [
        coverage.iter().filter(|count| **count == 0).count(),
        support_violations,
        non_finite_reconstruction,
    ];
    let localization_pass = required_radii.iter().all(|radius| *radius != usize::MAX);
    let numerical_pass = structural_failures == [0; 3]
        && closure_error <= 1.0e-12
        && frame_max / frame_min <= 1.0 + 1.0e-6
        && reconstruction_errors[0] <= 1.0e-5
        && reconstruction_errors[1] <= 1.0e-6
        && reconstruction_errors[2] <= 1.0e-5
        && reconstruction_errors[3] <= 1.0e-5;

    let mut review = Review {
        geometry: [FFT_FRAMES, bands.len(), common_coefficients, common_hop],
        coefficient_counts: [unequal_coefficients, dense_coefficients],
        coefficient_cost: [
            dense_coefficients as f64 / unequal_coefficients as f64,
            dense_coefficients as f64 / FFT_FRAMES as f64,
        ],
        frame_values: [frame_min, frame_max, frame_max / frame_min],
        structural_failures,
        reconstruction_errors: [
            closure_error,
            reconstruction_errors[0],
            reconstruction_errors[1],
            reconstruction_errors[2],
            reconstruction_errors[3],
        ],
        localization_radii,
        localization_curves,
        required_radii,
        limiting_bands,
        hashes: [
            filter_hash,
            filter_hash,
            frame_hash,
            frame_hash,
            dual_hash,
            dual_hash,
            0,
        ],
        direction: if numerical_pass && localization_pass {
            Direction::PhaseTopologyContract
        } else {
            Direction::OperatorReview
        },
    };
    review.hashes[6] = evidence_hash(&review, reconstruction_hash, atom_hash);
    review
}

fn frame_operator(bands: &[Band]) -> (Vec<f64>, Vec<usize>) {
    let mut frame = vec![0.0_f64; FFT_FRAMES];
    let mut coverage = vec![0_usize; FFT_FRAMES];
    for band in bands {
        for &(bin, weight) in &band.taps {
            frame[bin] += f64::from(weight) * f64::from(weight);
            coverage[bin] += 1;
        }
    }
    (frame, coverage)
}

fn reconstruct(
    bands: &[Band],
    frame_operator: &[f64],
    common_coefficients: usize,
) -> ([f64; 4], usize, u64) {
    let input = deterministic_probe();
    let mut planner = FftPlanner::<f64>::new();
    let forward_full = planner.plan_fft_forward(FFT_FRAMES);
    let inverse_full = planner.plan_fft_inverse(FFT_FRAMES);
    let forward_band = planner.plan_fft_forward(common_coefficients);
    let inverse_band = planner.plan_fft_inverse(common_coefficients);
    let mut spectrum = input
        .iter()
        .map(|sample| Complex::new(*sample, 0.0))
        .collect::<Vec<_>>();
    forward_full.process(&mut spectrum);
    let mut reconstructed = vec![Complex::new(0.0, 0.0); FFT_FRAMES];
    let mut non_finite = 0;

    for band in bands {
        let mut coefficients = vec![Complex::new(0.0, 0.0); common_coefficients];
        for &(bin, weight) in &band.taps {
            let local = circular_delta(bin, band.center, FFT_FRAMES)
                .rem_euclid(common_coefficients as isize) as usize;
            coefficients[local] = spectrum[bin] * f64::from(weight);
        }
        inverse_band.process(&mut coefficients);
        let scale = 1.0 / common_coefficients as f64;
        for value in &mut coefficients {
            *value *= scale;
            non_finite += usize::from(!value.re.is_finite() || !value.im.is_finite());
        }
        forward_band.process(&mut coefficients);
        for &(bin, weight) in &band.taps {
            let local = circular_delta(bin, band.center, FFT_FRAMES)
                .rem_euclid(common_coefficients as isize) as usize;
            reconstructed[bin] += coefficients[local] * f64::from(weight) / frame_operator[bin];
        }
    }

    inverse_full.process(&mut reconstructed);
    let scale = 1.0 / FFT_FRAMES as f64;
    let mut errors = Vec::with_capacity(FFT_FRAMES);
    let mut output_hash = HASH_OFFSET;
    for (source, output) in input.iter().zip(&reconstructed) {
        let output = *output * scale;
        non_finite += usize::from(!output.re.is_finite() || !output.im.is_finite());
        errors.push((source - output.re).abs());
        hash_u64(&mut output_hash, output.re.to_bits());
        hash_u64(&mut output_hash, output.im.to_bits());
    }
    let peak = errors.iter().copied().fold(0.0_f64, f64::max);
    let rms = (errors.iter().map(|error| error * error).sum::<f64>() / FFT_FRAMES as f64).sqrt();
    (
        [peak, rms, errors[0], errors[FFT_FRAMES - 1]],
        non_finite,
        output_hash,
    )
}

fn localization(
    bands: &[Band],
    frame_operator: &[f64],
    common_hop: usize,
) -> (Vec<usize>, Vec<[f64; 2]>, [usize; 2], [usize; 2], u64) {
    let radii = (common_hop..=LOCALIZATION_CAP)
        .step_by(common_hop)
        .collect::<Vec<_>>();
    let mut curves = vec![[0.0_f64; 2]; radii.len()];
    let mut required = [0_usize; 2];
    let mut limiting = [0_usize; 2];
    let mut maximum_cap_leakage = [0.0_f64; 2];
    let mut atom_hash = HASH_OFFSET;
    let mut planner = FftPlanner::<f64>::new();
    let inverse = planner.plan_fft_inverse(FFT_FRAMES);

    for (band_index, band) in bands.iter().enumerate() {
        for form in 0..2 {
            let mut atom = vec![Complex::new(0.0, 0.0); FFT_FRAMES];
            for &(bin, weight) in &band.taps {
                atom[bin].re = if form == 0 {
                    f64::from(weight)
                } else {
                    f64::from(weight) / frame_operator[bin]
                };
            }
            inverse.process(&mut atom);
            let mut energy_by_distance = vec![0.0_f64; FFT_FRAMES / 2 + 1];
            for (index, value) in atom.iter().enumerate() {
                energy_by_distance[circular_distance(index, FFT_FRAMES)] += value.norm_sqr();
            }
            let total = energy_by_distance.iter().sum::<f64>();
            let mut excluded_by_distance = vec![0.0_f64; energy_by_distance.len()];
            let mut excluded = 0.0;
            for distance in (0..energy_by_distance.len()).rev() {
                excluded_by_distance[distance] = excluded;
                excluded += energy_by_distance[distance];
            }
            let mut first = usize::MAX;
            for (radius_index, radius) in radii.iter().copied().enumerate() {
                let excluded = excluded_by_distance[radius] / total.max(f64::MIN_POSITIVE);
                curves[radius_index][form] = curves[radius_index][form].max(excluded);
                if first == usize::MAX && excluded <= LOCALIZATION_THRESHOLD {
                    first = radius;
                }
            }
            if first == usize::MAX {
                required[form] = usize::MAX;
            } else if required[form] != usize::MAX && first >= required[form] {
                required[form] = first;
            }
            let cap_leakage = excluded_by_distance[LOCALIZATION_CAP] / total.max(f64::MIN_POSITIVE);
            if cap_leakage >= maximum_cap_leakage[form] {
                maximum_cap_leakage[form] = cap_leakage;
                limiting[form] = band_index;
            }
            for value in &atom {
                hash_u64(&mut atom_hash, value.re.to_bits());
                hash_u64(&mut atom_hash, value.im.to_bits());
            }
        }
    }
    (radii, curves, required, limiting, atom_hash)
}

fn real_spectrum_closure(bands: &[Band]) -> f64 {
    let mut maximum = 0.0_f64;
    for band in bands {
        let mirror_center = if band.center == 0 {
            0
        } else {
            FFT_FRAMES - band.center
        };
        let mirror = bands
            .iter()
            .find(|candidate| candidate.center == mirror_center);
        let Some(mirror) = mirror else {
            return f64::INFINITY;
        };
        for &(bin, weight) in &band.taps {
            let mirror_bin = if bin == 0 { 0 } else { FFT_FRAMES - bin };
            let mirror_weight = mirror
                .taps
                .iter()
                .find(|(candidate, _)| *candidate == mirror_bin)
                .map(|(_, value)| *value)
                .unwrap_or(0.0);
            maximum = maximum.max(f64::from((weight - mirror_weight).abs()));
        }
    }
    maximum
}

fn deterministic_probe() -> Vec<f64> {
    let mut state = 0x243f_6a88_85a3_08d3_u64;
    (0..FFT_FRAMES)
        .map(|index| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let noise = ((state >> 40) as f64 / (1_u64 << 24) as f64 - 0.5) * 0.05;
            let time = index as f64 / SAMPLE_RATE_HZ;
            let impulse = if matches!(index, 4096 | 32768 | 61439) {
                0.75
            } else {
                0.0
            };
            0.35 * (std::f64::consts::TAU * 55.0 * time).sin()
                + 0.2 * (std::f64::consts::TAU * 440.0 * time).sin()
                + 0.1 * (std::f64::consts::TAU * 8_000.0 * time).sin()
                + impulse
                + noise
        })
        .collect()
}

fn filter_hash(bands: &[Band]) -> u64 {
    let mut hash = HASH_OFFSET;
    for band in bands {
        hash_usize(&mut hash, band.center);
        for &(bin, weight) in &band.taps {
            hash_usize(&mut hash, bin);
            hash_u32(&mut hash, weight.to_bits());
        }
    }
    hash
}

fn dual_hash(bands: &[Band], frame_operator: &[f64]) -> u64 {
    let mut hash = HASH_OFFSET;
    for band in bands {
        for &(bin, weight) in &band.taps {
            hash_u64(
                &mut hash,
                (f64::from(weight) / frame_operator[bin]).to_bits(),
            );
        }
    }
    hash
}

fn scalar_hash(values: &[f64]) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in values {
        hash_u64(&mut hash, value.to_bits());
    }
    hash
}

fn evidence_hash(review: &Review, reconstruction_hash: u64, atom_hash: u64) -> u64 {
    let mut hash = HASH_OFFSET;
    for value in review.geometry.into_iter().chain(review.coefficient_counts) {
        hash_usize(&mut hash, value);
    }
    for value in review
        .coefficient_cost
        .into_iter()
        .chain(review.frame_values)
        .chain(review.reconstruction_errors)
    {
        hash_u64(&mut hash, value.to_bits());
    }
    for curve in &review.localization_curves {
        hash_u64(&mut hash, curve[0].to_bits());
        hash_u64(&mut hash, curve[1].to_bits());
    }
    hash_u64(&mut hash, reconstruction_hash);
    hash_u64(&mut hash, atom_hash);
    hash
}

fn circular_delta(bin: usize, center: usize, length: usize) -> isize {
    let raw = bin as isize - center as isize;
    if raw > length as isize / 2 {
        raw - length as isize
    } else if raw < -(length as isize / 2) {
        raw + length as isize
    } else {
        raw
    }
}

fn circular_distance(index: usize, length: usize) -> usize {
    index.min(length - index)
}

fn hash_u64(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
