use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::Sample;

mod analysis;
mod integration;
mod types;

use analysis::{analyze, frequency_phase_derivative, hann_window, time_phase_derivatives};
use integration::integrate_frame;
pub use types::{StretchPhaseGradientEvidence, StretchPhaseGradientRender};

#[cfg(test)]
mod tests;

pub(super) const WINDOW_FRAMES: usize = 4_092;
pub(super) const FFT_FRAMES: usize = 8_192;
pub(super) const SYNTHESIS_HOP: usize = 1_024;
pub(super) const RELATIVE_TOLERANCE: f32 = 1.0e-6;
pub(super) const BINS: usize = FFT_FRAMES / 2 + 1;

#[derive(Default)]
pub(super) struct EvidenceAccumulator {
    pub(super) significant_bins: usize,
    pub(super) insignificant_bins: usize,
    pub(super) horizontal_assignments: usize,
    pub(super) vertical_assignments: usize,
    pub(super) duplicate_assignments: usize,
    pub(super) missing_assignments: usize,
    pub(super) heap_high_water: usize,
    pub(super) derivatives_finite: bool,
    pub(super) max_conjugate_symmetry_error: f64,
    pub(super) trace_hash: u64,
}

pub(crate) fn stretch_phase_gradient_review_mono(
    input: &[Sample],
    ratio: f64,
) -> StretchPhaseGradientRender {
    let ratio = ratio.clamp(0.25, 4.0);
    let target_len = (input.len() as f64 * ratio).round() as usize;
    let analysis_hop = (SYNTHESIS_HOP as f64 / ratio).round().max(1.0) as usize;
    if input.is_empty() || target_len == 0 || (ratio - 1.0).abs() < 1.0e-9 {
        let samples = input[..target_len.min(input.len())].to_vec();
        return StretchPhaseGradientRender {
            evidence: empty_evidence(&samples, analysis_hop),
            samples,
        };
    }

    let window = hann_window();
    let output_crop_start = (WINDOW_FRAMES as f64 * 0.5 * ratio).round() as usize;
    let source_frame_count = (WINDOW_FRAMES / 2 + input.len() + WINDOW_FRAMES / 2)
        .saturating_sub(WINDOW_FRAMES)
        .div_ceil(analysis_hop)
        + 1;
    let output_frame_count = (output_crop_start + target_len)
        .saturating_sub(WINDOW_FRAMES)
        .div_ceil(SYNTHESIS_HOP)
        + 1;
    let frame_count = source_frame_count.max(output_frame_count).max(2);
    let analysis = analyze(input, analysis_hop, frame_count + 2, &window);
    let time_derivatives = time_phase_derivatives(&analysis.phases, analysis_hop);
    let mut accumulator = EvidenceAccumulator {
        derivatives_finite: time_derivatives
            .iter()
            .flatten()
            .all(|value| value.is_finite()),
        trace_hash: 0xcbf2_9ce4_8422_2325,
        ..EvidenceAccumulator::default()
    };

    let mut synthesis_phases = analysis.phases[1].clone();
    let ola_len = (frame_count - 1) * SYNTHESIS_HOP + WINDOW_FRAMES;
    let mut output = vec![0.0_f32; ola_len];
    let mut normalization = vec![0.0_f32; ola_len];
    let mut planner = FftPlanner::<f32>::new();
    let inverse = planner.plan_fft_inverse(FFT_FRAMES);

    synthesize_frame(
        &analysis.spectra[1],
        &synthesis_phases,
        0,
        &window,
        &inverse,
        &mut output,
        &mut normalization,
        &mut accumulator,
    );

    for frame in 1..frame_count {
        let analysis_frame = frame + 1;
        let frequency_derivative = frequency_phase_derivative(&analysis.phases[analysis_frame]);
        accumulator.derivatives_finite &=
            frequency_derivative.iter().all(|value| value.is_finite());
        synthesis_phases = integrate_frame(
            &analysis,
            &time_derivatives,
            &frequency_derivative,
            analysis_frame,
            &synthesis_phases,
            ratio,
            &mut accumulator,
        );
        synthesize_frame(
            &analysis.spectra[analysis_frame],
            &synthesis_phases,
            frame * SYNTHESIS_HOP,
            &window,
            &inverse,
            &mut output,
            &mut normalization,
            &mut accumulator,
        );
    }

    let crop_end = output_crop_start + target_len;
    let uncovered_output_samples = normalization[output_crop_start..crop_end]
        .iter()
        .filter(|weight| **weight <= f32::EPSILON)
        .count();
    for (sample, weight) in output.iter_mut().zip(&normalization) {
        if *weight > f32::EPSILON {
            *sample /= *weight;
        }
    }
    let samples = output[output_crop_start..crop_end].to_vec();
    StretchPhaseGradientRender {
        evidence: StretchPhaseGradientEvidence {
            window_frames: WINDOW_FRAMES,
            fft_frames: FFT_FRAMES,
            analysis_hop_frames: analysis_hop,
            synthesis_hop_frames: SYNTHESIS_HOP,
            synthesis_frames: frame_count,
            significant_bins: accumulator.significant_bins,
            insignificant_bins: accumulator.insignificant_bins,
            horizontal_assignments: accumulator.horizontal_assignments,
            vertical_assignments: accumulator.vertical_assignments,
            duplicate_assignments: accumulator.duplicate_assignments,
            missing_assignments: accumulator.missing_assignments,
            heap_high_water: accumulator.heap_high_water,
            heap_capacity_bound: BINS * 2,
            max_conjugate_symmetry_error: accumulator.max_conjugate_symmetry_error,
            uncovered_output_samples,
            derivatives_finite: accumulator.derivatives_finite,
            all_samples_finite: samples.iter().all(|sample| sample.is_finite()),
            synthesis_positions_monotonic: true,
            sample_hash: sample_hash(&samples),
            trace_hash: accumulator.trace_hash,
        },
        samples,
    }
}

fn empty_evidence(samples: &[Sample], analysis_hop: usize) -> StretchPhaseGradientEvidence {
    StretchPhaseGradientEvidence {
        window_frames: WINDOW_FRAMES,
        fft_frames: FFT_FRAMES,
        analysis_hop_frames: analysis_hop,
        synthesis_hop_frames: SYNTHESIS_HOP,
        synthesis_frames: usize::from(!samples.is_empty()),
        significant_bins: 0,
        insignificant_bins: 0,
        horizontal_assignments: 0,
        vertical_assignments: 0,
        duplicate_assignments: 0,
        missing_assignments: 0,
        heap_high_water: 0,
        heap_capacity_bound: BINS * 2,
        max_conjugate_symmetry_error: 0.0,
        uncovered_output_samples: 0,
        derivatives_finite: true,
        all_samples_finite: samples.iter().all(|sample| sample.is_finite()),
        synthesis_positions_monotonic: true,
        sample_hash: sample_hash(samples),
        trace_hash: 0xcbf2_9ce4_8422_2325,
    }
}

#[allow(clippy::too_many_arguments)]
fn synthesize_frame(
    analyzed_spectrum: &[Complex32],
    phase: &[f32],
    start: usize,
    window: &[f32],
    inverse: &std::sync::Arc<dyn rustfft::Fft<f32>>,
    output: &mut [f32],
    normalization: &mut [f32],
    accumulator: &mut EvidenceAccumulator,
) {
    let mut spectrum = vec![Complex32::new(0.0, 0.0); FFT_FRAMES];
    for bin in 0..BINS {
        spectrum[bin] = Complex32::from_polar(analyzed_spectrum[bin].norm(), phase[bin]);
        if bin > 0 && bin < BINS - 1 {
            spectrum[FFT_FRAMES - bin] = spectrum[bin].conj();
            let error = (spectrum[FFT_FRAMES - bin] - spectrum[bin].conj()).norm() as f64;
            accumulator.max_conjugate_symmetry_error =
                accumulator.max_conjugate_symmetry_error.max(error);
        }
    }
    spectrum[0].im = 0.0;
    spectrum[BINS - 1].im = 0.0;
    inverse.process(&mut spectrum);
    for index in 0..WINDOW_FRAMES {
        let weight = window[index];
        output[start + index] += spectrum[index].re * weight / FFT_FRAMES as f32;
        normalization[start + index] += weight * weight;
    }
}

fn sample_hash(samples: &[Sample]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for sample in samples {
        trace_hash_value(&mut hash, sample.to_bits() as u64);
    }
    hash
}

pub(super) fn trace_hash_value(hash: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *hash ^= u64::from(byte);
        *hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
}
