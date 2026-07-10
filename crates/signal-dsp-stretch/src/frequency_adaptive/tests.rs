use super::*;

const SAMPLE_RATE: SampleRate = SampleRate(48_000);
const CONTROL_LEN: usize = 4_096;

#[test]
fn frequency_adaptive_reconstruction_controls_pass() {
    let mut controls = vec![
        sine(55.0),
        sine(440.0),
        sine(4_000.0),
        sine(19_500.0),
        sine(23_500.0),
        deterministic_noise(),
        mixed_control(),
        vec![0.0; CONTROL_LEN],
    ];
    let mut impulse = vec![0.0; CONTROL_LEN];
    impulse[CONTROL_LEN / 2] = 1.0;
    controls.push(impulse);
    for input in controls {
        assert_reconstruction_gate(&input);
    }
}

#[test]
fn frequency_adaptive_reconstruction_is_deterministic() {
    let input = mixed_control();
    let first = frequency_adaptive_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = frequency_adaptive_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
    eprintln!(
        "frequency_adaptive fft={} bands={} coefficients={} frame=[{:.9},{:.9}] condition={:.9} overlap={} peak={:.9e} rms={:.9e} filters={:016x} coefficients_hash={:016x} reconstruction={:016x}",
        first.evidence.fft_frames,
        first.evidence.band_count,
        first.evidence.coefficient_count,
        first.evidence.frame_operator_min,
        first.evidence.frame_operator_max,
        first.evidence.frame_condition_ratio,
        first.evidence.multiply_covered_frequency_bins,
        first.evidence.reconstruction_peak_error,
        first.evidence.reconstruction_rms_error,
        first.evidence.filter_hash,
        first.evidence.coefficient_hash,
        first.evidence.reconstruction_hash,
    );
}

#[test]
fn frequency_adaptive_reconstruction_empty_input_is_exact() {
    let review = frequency_adaptive_reconstruction_review_mono(&[], SAMPLE_RATE);
    assert!(review.samples.is_empty());
    assert_eq!(review.evidence.source_frames, 0);
    assert_eq!(review.evidence.output_frames, 0);
    assert_eq!(review.evidence.reconstruction_peak_error, 0.0);
}

fn assert_reconstruction_gate(input: &[Sample]) {
    let review = frequency_adaptive_reconstruction_review_mono(input, SAMPLE_RATE);
    let evidence = &review.evidence;
    assert_eq!(review.samples.len(), input.len());
    assert_eq!(evidence.source_frames, input.len());
    assert_eq!(evidence.output_frames, input.len());
    assert!(evidence.band_count > 2);
    assert!(evidence.coefficient_count > 0);
    assert!(evidence.frame_operator_min.is_finite());
    assert!(evidence.frame_operator_min > 0.0);
    assert!(evidence.frame_operator_max.is_finite());
    assert!(evidence.frame_condition_ratio.is_finite());
    assert_eq!(evidence.uncovered_frequency_bins, 0);
    assert!(evidence.multiply_covered_frequency_bins > 0);
    assert_eq!(evidence.painless_support_violations, 0);
    assert!(evidence.reconstruction_peak_error <= 1.0e-5);
    assert!(evidence.reconstruction_rms_error <= 1.0e-6);
    assert!(evidence.reconstruction_head_error <= 1.0e-5);
    assert!(evidence.reconstruction_tail_error <= 1.0e-5);
    assert_eq!(evidence.non_finite_coefficients, 0);
    assert_eq!(evidence.non_finite_output_samples, 0);
    assert!(evidence.max_band_impulse_delay_frames <= 1);
    assert!(evidence.bands.iter().all(|band| {
        band.support_bins <= band.coefficient_count
            && band.decimation_frames > 0
            && band.impulse_peak_frame == 0
    }));
}

fn sine(frequency_hz: f32) -> Vec<Sample> {
    (0..CONTROL_LEN)
        .map(|index| {
            0.5 * (std::f32::consts::TAU * frequency_hz * index as f32 / SAMPLE_RATE.0 as f32).sin()
        })
        .collect()
}

fn deterministic_noise() -> Vec<Sample> {
    let mut state = 0x1234_5678_u32;
    (0..CONTROL_LEN)
        .map(|_| {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (state as f32 / u32::MAX as f32 - 0.5) * 0.5
        })
        .collect()
}

fn mixed_control() -> Vec<Sample> {
    let mut samples = (0..CONTROL_LEN)
        .map(|index| {
            let time = index as f32 / SAMPLE_RATE.0 as f32;
            0.3 * (std::f32::consts::TAU * 110.0 * time).sin()
                + 0.2 * (std::f32::consts::TAU * 3_200.0 * time).sin()
        })
        .collect::<Vec<_>>();
    samples[CONTROL_LEN / 3] += 0.5;
    samples
}
