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
    impulse[192] = 1.0;
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

#[test]
fn common_grid_wavelet_reconstruction_meets_frame_and_dual_gate() {
    let input = mixed_control();
    let review = common_grid_wavelet_reconstruction_review_mono(&input, SAMPLE_RATE);
    let evidence = &review.evidence;
    assert_eq!(review.samples.len(), input.len());
    assert_eq!(evidence.channel_count, 1_536);
    assert_eq!(evidence.lowpass_channel_count, 16);
    assert_eq!(evidence.hop_frames, 384);
    assert_eq!(evidence.redundancy, 8.0);
    assert!(evidence.frame_condition_ratio <= 1.25, "{evidence:?}");
    assert!(evidence.canonical_dual_residual <= 1.0e-8, "{evidence:?}");
    assert!(evidence.reconstruction_peak_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_rms_error <= 1.0e-6, "{evidence:?}");
    assert_eq!(evidence.non_finite_values, 0);
    eprintln!("common_grid_wavelet {evidence:?}");
}

#[test]
fn common_grid_wavelet_reconstruction_is_deterministic() {
    let input = sine(440.0)[..384].to_vec();
    let first = common_grid_wavelet_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = common_grid_wavelet_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
}

#[test]
fn common_grid_wavelet_reconstruction_controls_pass() {
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
    impulse[192] = 1.0;
    controls.push(impulse);
    controls.push(Vec::new());
    for input in controls {
        let short = &input[..input.len().min(384)];
        let review = common_grid_wavelet_reconstruction_review_mono(short, SAMPLE_RATE);
        assert_eq!(review.samples.len(), short.len());
        assert!(review.evidence.frame_condition_ratio <= 1.25);
        assert!(review.evidence.canonical_dual_residual <= 1.0e-8);
        assert!(review.evidence.reconstruction_peak_error <= 1.0e-5);
        assert!(review.evidence.reconstruction_rms_error <= 1.0e-6);
        assert!(review.evidence.reconstruction_head_error <= 1.0e-5);
        assert!(review.evidence.reconstruction_tail_error <= 1.0e-5);
        assert_eq!(review.evidence.non_finite_values, 0);
    }
}

#[test]
fn common_grid_phase_transport_rejects_high_band_phase_aliasing() {
    for frequency in [312.5_f32, 1_000.0] {
        let input = (0..24_576)
            .map(|index| {
                0.5 * (std::f32::consts::TAU * frequency * index as f32 / SAMPLE_RATE.0 as f32)
                    .sin()
            })
            .collect::<Vec<_>>();
        let evidence =
            common_grid_tone_phase_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
        assert!(evidence.horizontal_measurements > 0, "{evidence:?}");
        assert!(evidence.vertical_measurements > 0, "{evidence:?}");
        assert!(
            evidence.max_angular_frequency_error <= 1.0e-6,
            "{evidence:?}"
        );
        assert!(
            evidence.max_compensated_phase_residual <= 2.0e-5,
            "{evidence:?}"
        );
        assert!(evidence.all_values_finite);
        eprintln!("common_grid_phase frequency={frequency} {evidence:?}");
    }
    let frequency = 8_000.0_f32;
    let input = (0..24_576)
        .map(|index| {
            (0.5 * (std::f64::consts::TAU * f64::from(frequency) * index as f64
                / f64::from(SAMPLE_RATE.0))
            .sin()) as f32
        })
        .collect::<Vec<_>>();
    let evidence = common_grid_tone_phase_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
    assert!(evidence.max_angular_frequency_error > 1.0e-3);
    assert!(evidence.max_compensated_phase_residual > 0.1);
}

#[test]
fn common_grid_derivative_estimator_is_alias_free_and_deterministic() {
    for frequency in [312.5_f32, 1_000.0, 8_000.0, 19_500.0] {
        let input = periodic_tone(frequency);
        let first =
            common_grid_derivative_tone_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
        let repeated =
            common_grid_derivative_tone_review_mono(&input, SAMPLE_RATE, f64::from(frequency));
        assert_eq!(first, repeated);
        assert!(first.horizontal_measurements > 0, "{first:?}");
        assert!(first.vertical_measurements > 0, "{first:?}");
        assert!(first.max_angular_frequency_error <= 1.0e-6, "{first:?}");
        assert!(first.max_compensated_phase_residual <= 2.0e-5, "{first:?}");
        assert!(first.all_values_finite);
        eprintln!("derivative frequency={frequency} {first:?}");
    }
}

#[test]
fn common_grid_derivative_estimator_handles_silence_and_noise() {
    let silence = common_grid_derivative_tone_review_mono(&vec![0.0; 384], SAMPLE_RATE, 0.0);
    assert_eq!(silence.horizontal_measurements, 0);
    assert_eq!(silence.vertical_measurements, 0);
    assert!(silence.zero_energy_skips > 0);
    assert!(silence.all_values_finite);

    let noise =
        common_grid_derivative_tone_review_mono(&deterministic_noise()[..384], SAMPLE_RATE, 0.0);
    assert!(noise.horizontal_measurements > 0);
    assert!(noise.all_values_finite);
}

fn periodic_tone(frequency: f32) -> Vec<Sample> {
    (0..24_576)
        .map(|index| {
            (0.5 * (std::f64::consts::TAU * f64::from(frequency) * index as f64
                / f64::from(SAMPLE_RATE.0))
            .sin()) as f32
        })
        .collect()
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
