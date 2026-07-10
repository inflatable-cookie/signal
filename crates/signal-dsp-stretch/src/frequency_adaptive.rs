use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::{Sample, SampleRate};

mod types;
pub use types::{
    StretchCommonGridTonePhaseEvidence, StretchCommonGridWaveletEvidence,
    StretchCommonGridWaveletReview, StretchFrequencyAdaptiveBandEvidence,
    StretchFrequencyAdaptiveEvidence, StretchFrequencyAdaptiveReview,
};

mod common_grid;
pub(crate) use common_grid::common_grid_tone_phase_review_mono;
pub(crate) use common_grid::common_grid_wavelet_reconstruction_review_mono;

#[cfg(test)]
mod tests;

const BANDS_PER_OCTAVE: f64 = 48.0;
const MIN_FREQUENCY_HZ: f64 = 50.0;
const MAX_FREQUENCY_HZ: f64 = 20_000.0;
const MIN_FFT_FRAMES: usize = 4_096;
const HASH_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;

#[derive(Clone)]
struct Band {
    center: usize,
    taps: Vec<(usize, f32)>,
    coefficient_frames: usize,
}

pub(crate) fn frequency_adaptive_reconstruction_review_mono(
    input: &[Sample],
    sample_rate: SampleRate,
) -> StretchFrequencyAdaptiveReview {
    let sample_rate_hz = sample_rate.0.max(1) as f64;
    let fft_frames = input.len().max(MIN_FFT_FRAMES).next_power_of_two();
    let bands = build_bands(fft_frames, sample_rate_hz);
    let mut frame_operator = vec![0.0_f32; fft_frames];
    let mut coverage = vec![0_usize; fft_frames];
    let mut filter_hash = HASH_OFFSET;
    for band in &bands {
        hash_usize(&mut filter_hash, band.center);
        hash_usize(&mut filter_hash, band.coefficient_frames);
        for &(bin, weight) in &band.taps {
            frame_operator[bin] += weight * weight;
            coverage[bin] += 1;
            hash_usize(&mut filter_hash, bin);
            hash_u32(&mut filter_hash, weight.to_bits());
        }
    }

    let frame_operator_min = frame_operator.iter().copied().fold(f32::INFINITY, f32::min);
    let frame_operator_max = frame_operator.iter().copied().fold(0.0_f32, f32::max);
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(fft_frames);
    let inverse = planner.plan_fft_inverse(fft_frames);
    let mut spectrum = vec![Complex32::new(0.0, 0.0); fft_frames];
    for (slot, sample) in spectrum.iter_mut().zip(input) {
        slot.re = *sample;
    }
    forward.process(&mut spectrum);

    let mut reconstructed_spectrum = vec![Complex32::new(0.0, 0.0); fft_frames];
    let mut coefficient_hash = HASH_OFFSET;
    let mut non_finite_coefficients = 0;
    let mut band_evidence = Vec::with_capacity(bands.len());
    for band in &bands {
        let mut coefficients = vec![Complex32::new(0.0, 0.0); band.coefficient_frames];
        for &(bin, weight) in &band.taps {
            let local = circular_delta(bin, band.center, fft_frames)
                .rem_euclid(band.coefficient_frames as isize) as usize;
            coefficients[local] = spectrum[bin] * weight;
        }
        planner
            .plan_fft_inverse(band.coefficient_frames)
            .process(&mut coefficients);
        let scale = 1.0 / band.coefficient_frames as f32;
        for coefficient in &mut coefficients {
            *coefficient *= scale;
            non_finite_coefficients += usize::from(!coefficient.re.is_finite());
            non_finite_coefficients += usize::from(!coefficient.im.is_finite());
            hash_u32(&mut coefficient_hash, coefficient.re.to_bits());
            hash_u32(&mut coefficient_hash, coefficient.im.to_bits());
        }
        planner
            .plan_fft_forward(band.coefficient_frames)
            .process(&mut coefficients);
        for &(bin, weight) in &band.taps {
            let local = circular_delta(bin, band.center, fft_frames)
                .rem_euclid(band.coefficient_frames as isize) as usize;
            let dual = weight / frame_operator[bin];
            reconstructed_spectrum[bin] += coefficients[local] * dual;
        }
        band_evidence.push(StretchFrequencyAdaptiveBandEvidence {
            center_bin: band.center,
            center_frequency_hz: absolute_frequency(band.center, fft_frames, sample_rate_hz),
            support_bins: band.taps.len(),
            decimation_frames: fft_frames / band.coefficient_frames,
            coefficient_count: band.coefficient_frames,
            impulse_peak_frame: 0,
        });
    }

    inverse.process(&mut reconstructed_spectrum);
    let inverse_scale = 1.0 / fft_frames as f32;
    let samples = reconstructed_spectrum[..input.len()]
        .iter()
        .map(|sample| sample.re * inverse_scale)
        .collect::<Vec<_>>();
    let errors = input
        .iter()
        .zip(&samples)
        .map(|(source, output)| (*source as f64 - *output as f64).abs())
        .collect::<Vec<_>>();
    let peak_error = errors.iter().copied().fold(0.0_f64, f64::max);
    let rms_error = if errors.is_empty() {
        0.0
    } else {
        (errors.iter().map(|error| error * error).sum::<f64>() / errors.len() as f64).sqrt()
    };
    let evidence = StretchFrequencyAdaptiveEvidence {
        fft_frames,
        band_count: bands.len(),
        coefficient_count: bands.iter().map(|band| band.coefficient_frames).sum(),
        frame_operator_min: frame_operator_min as f64,
        frame_operator_max: frame_operator_max as f64,
        frame_condition_ratio: frame_operator_max as f64 / frame_operator_min as f64,
        uncovered_frequency_bins: coverage.iter().filter(|count| **count == 0).count(),
        multiply_covered_frequency_bins: coverage.iter().filter(|count| **count > 1).count(),
        painless_support_violations: bands
            .iter()
            .filter(|band| band.taps.len() > band.coefficient_frames)
            .count(),
        source_frames: input.len(),
        output_frames: samples.len(),
        reconstruction_peak_error: peak_error,
        reconstruction_rms_error: rms_error,
        reconstruction_head_error: errors.first().copied().unwrap_or(0.0),
        reconstruction_tail_error: errors.last().copied().unwrap_or(0.0),
        non_finite_coefficients,
        non_finite_output_samples: samples.iter().filter(|sample| !sample.is_finite()).count(),
        max_band_impulse_delay_frames: 0,
        filter_hash,
        coefficient_hash,
        reconstruction_hash: sample_hash(&samples),
        bands: band_evidence,
    };
    StretchFrequencyAdaptiveReview { samples, evidence }
}

fn build_bands(fft_frames: usize, sample_rate_hz: f64) -> Vec<Band> {
    let nyquist_bin = fft_frames / 2;
    let max_frequency = MAX_FREQUENCY_HZ.min(sample_rate_hz * 0.5);
    let mut positive = Vec::new();
    let mut frequency = MIN_FREQUENCY_HZ;
    while frequency < max_frequency {
        let bin = (frequency * fft_frames as f64 / sample_rate_hz).round() as usize;
        if bin > 0 && bin < nyquist_bin && positive.last().copied() != Some(bin) {
            positive.push(bin);
        }
        frequency *= 2.0_f64.powf(1.0 / BANDS_PER_OCTAVE);
    }
    let mut centers = Vec::with_capacity(positive.len() * 2 + 2);
    centers.push(0);
    centers.extend(positive.iter().copied());
    centers.push(nyquist_bin);
    centers.extend(positive.iter().rev().map(|bin| fft_frames - bin));
    centers.sort_unstable();
    centers.dedup();

    let mut taps = vec![Vec::<(usize, f32)>::new(); centers.len()];
    for bin in 0..fft_frames {
        let right_index = centers.partition_point(|center| *center <= bin) % centers.len();
        let left_index = (right_index + centers.len() - 1) % centers.len();
        let left = centers[left_index];
        let right = centers[right_index];
        let span = if right > left {
            right - left
        } else {
            fft_frames - left + right
        };
        let offset = if bin >= left {
            bin - left
        } else {
            fft_frames - left + bin
        };
        let phase = std::f32::consts::FRAC_PI_2 * offset as f32 / span as f32;
        let left_weight = phase.cos();
        let right_weight = phase.sin();
        if left_weight > f32::EPSILON {
            taps[left_index].push((bin, left_weight));
        }
        if right_weight > f32::EPSILON {
            taps[right_index].push((bin, right_weight));
        }
    }
    centers
        .into_iter()
        .zip(taps)
        .map(|(center, taps)| Band {
            center,
            coefficient_frames: taps.len().max(1).next_power_of_two(),
            taps,
        })
        .collect()
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

fn absolute_frequency(bin: usize, length: usize, sample_rate_hz: f64) -> f64 {
    let signed_bin = if bin <= length / 2 { bin } else { length - bin };
    signed_bin as f64 * sample_rate_hz / length as f64
}

fn sample_hash(samples: &[Sample]) -> u64 {
    let mut hash = HASH_OFFSET;
    for sample in samples {
        hash_u32(&mut hash, sample.to_bits());
    }
    hash
}

fn hash_usize(hash: &mut u64, value: usize) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}

fn hash_u32(hash: &mut u64, value: u32) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
