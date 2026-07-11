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
fn common_grid_boundary_candidate_rejects_frame_conditioning() {
    let input = mixed_control();
    let first = common_grid_boundary_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = common_grid_boundary_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
    let evidence = &first.reconstruction.evidence;
    assert_eq!(first.reconstruction.samples.len(), input.len());
    assert!(evidence.frame_condition_ratio > 1.25, "{evidence:?}");
    assert!(evidence.canonical_dual_residual <= 1.0e-8, "{evidence:?}");
    assert!(evidence.reconstruction_peak_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_rms_error <= 1.0e-6, "{evidence:?}");
    assert!(evidence.reconstruction_head_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_tail_error <= 1.0e-5, "{evidence:?}");
    assert_eq!(evidence.non_finite_values, 0);
    assert_ne!(first.preserved_filter_hash, 0);
    assert_ne!(first.nyquist_completion_hash, 0);
    assert_ne!(first.raw_filter_hash, 0);
}

#[test]
fn common_grid_preconditioned_candidate_rejects_frame_conditioning() {
    let input = mixed_control();
    let raw = common_grid_boundary_reconstruction_review_mono(&input, SAMPLE_RATE);
    let first = common_grid_preconditioned_reconstruction_review_mono(&input, SAMPLE_RATE);
    let repeated = common_grid_preconditioned_reconstruction_review_mono(&input, SAMPLE_RATE);
    assert_eq!(first, repeated);
    let evidence = &first.reconstruction.evidence;
    assert_eq!(first.reconstruction.samples.len(), input.len());
    assert!(evidence.frame_condition_ratio > 1.25, "{evidence:?}");
    assert!(evidence.canonical_dual_residual <= 1.0e-8, "{evidence:?}");
    assert!(evidence.reconstruction_peak_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_rms_error <= 1.0e-6, "{evidence:?}");
    assert!(evidence.reconstruction_head_error <= 1.0e-5, "{evidence:?}");
    assert!(evidence.reconstruction_tail_error <= 1.0e-5, "{evidence:?}");
    assert_eq!(evidence.non_finite_values, 0);
    assert_eq!(first.raw_filter_hash, raw.raw_filter_hash);
    assert_ne!(first.multiplier_hash, 0);
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

#[test]
fn common_grid_projected_phase_fields_are_exact_finite_and_deterministic() {
    let input = mixed_control()[..768].to_vec();
    for ratio in [0.75, 1.0, 1.5] {
        let first = common_grid_projected_phase_review_mono(&input, ratio);
        let repeated = common_grid_projected_phase_review_mono(&input, ratio);
        assert_eq!(first, repeated);
        assert_eq!(
            first.target_frames,
            (input.len() as f64 * ratio).round() as usize
        );
        assert_eq!(first.output_columns, first.target_frames.div_ceil(384) + 1);
        assert_eq!(
            first.projected_field_values,
            first.output_columns * 1_536 * 3
        );
        assert!(first.max_coordinate_error <= 1.0e-9, "{first:?}");
        assert!(first.coordinates_monotonic);
        assert!(first.boundary_pad_reads > 0, "{first:?}");
        assert_eq!(first.missing_assignments, 0, "{first:?}");
        assert_eq!(first.duplicate_assignments, 0, "{first:?}");
        assert!(first.heap_high_water <= first.heap_capacity, "{first:?}");
        assert_eq!(first.non_finite_values, 0, "{first:?}");
        if ratio != 1.0 {
            assert!(first.fractional_columns > 0, "{first:?}");
        }
        eprintln!("projected ratio={ratio} {first:?}");
    }
}

#[test]
fn common_grid_projected_phase_heap_uses_both_directions_and_handles_silence() {
    let mixed = common_grid_projected_phase_review_mono(&mixed_control()[..1_536], 1.5);
    assert!(mixed.seed_assignments > 0, "{mixed:?}");
    assert!(mixed.horizontal_assignments > 0, "{mixed:?}");
    assert!(mixed.vertical_assignments > 0, "{mixed:?}");
    assert_eq!(mixed.missing_assignments, 0, "{mixed:?}");
    assert!(mixed.heap_high_water <= mixed.heap_capacity, "{mixed:?}");

    let silence = common_grid_projected_phase_review_mono(&vec![0.0; 768], 0.75);
    assert_eq!(silence.seed_assignments, 0);
    assert_eq!(silence.horizontal_assignments, 0);
    assert_eq!(silence.vertical_assignments, 0);
    assert_eq!(silence.missing_assignments, 0);
    assert_eq!(silence.non_finite_values, 0);
}

#[test]
fn common_grid_projected_phase_contract_controls_pass() {
    let mut impulse = vec![0.0; 768];
    impulse[192] = 1.0;
    let controls = [
        sine(312.5)[..768].to_vec(),
        sine(1_000.0)[..768].to_vec(),
        sine(8_000.0)[..768].to_vec(),
        two_tone_control(),
        chirp_control(false),
        chirp_control(true),
        impulse,
        deterministic_noise()[..768].to_vec(),
        mixed_control()[..768].to_vec(),
        vec![0.0; 768],
    ];
    let mut horizontal = 0;
    let mut vertical = 0;
    let mut max_heap_high_water = 0;
    for input in controls {
        for ratio in [0.75, 1.0, 1.5] {
            let evidence = common_grid_projected_phase_review_mono(&input, ratio);
            assert!(evidence.max_coordinate_error <= 1.0e-9, "{evidence:?}");
            assert!(evidence.coordinates_monotonic);
            assert_eq!(evidence.missing_assignments, 0, "{evidence:?}");
            assert_eq!(evidence.duplicate_assignments, 0, "{evidence:?}");
            assert!(
                evidence.heap_high_water <= evidence.heap_capacity,
                "{evidence:?}"
            );
            assert_eq!(evidence.non_finite_values, 0, "{evidence:?}");
            horizontal += evidence.horizontal_assignments;
            vertical += evidence.vertical_assignments;
            max_heap_high_water = max_heap_high_water.max(evidence.heap_high_water);
        }
    }
    assert!(horizontal > 0);
    assert!(vertical > 0);
    eprintln!(
        "projected controls horizontal={horizontal} vertical={vertical} heap={max_heap_high_water}/3072"
    );
}

#[test]
fn common_grid_dual_guard_is_exact_bounded_and_deterministic() {
    let first = common_grid_dual_guard_review(384);
    let repeated = common_grid_dual_guard_review(384);
    assert_eq!(first, repeated);
    assert!(first.evaluated_channels > 0, "{first:?}");
    assert!(first.max_dual_residual <= 1.0e-8, "{first:?}");
    assert_eq!(first.non_finite_values, 0, "{first:?}");
    if first.passed {
        assert_eq!(first.evaluated_channels, first.channel_count);
        assert!(first.required_guard_lower_bound_frames <= first.guard_cap_frames);
        assert!(first.max_tail_energy_ratio <= 1.0e-12, "{first:?}");
    } else {
        assert!(first.required_guard_lower_bound_frames > first.guard_cap_frames);
    }
    eprintln!("dual_guard {first:?}");
}

#[test]
fn common_grid_tail_attribution_matrix_is_complete_and_deterministic() {
    let first = common_grid_tail_attribution_review();
    let repeated = common_grid_tail_attribution_review();
    assert_eq!(first, repeated);
    assert_eq!(first.probe_fft_frames, 34_176);
    assert_eq!(
        first.radii_frames,
        [384, 1_536, 4_096, 8_192, 12_288, 16_000]
    );
    assert_eq!(first.thresholds, [1.0e-6, 1.0e-8, 1.0e-10, 1.0e-12]);
    assert_eq!(first.atoms.len(), 30);
    assert_eq!(first.tightening_ratios.len(), 5);
    assert_eq!(first.dualization_ratios.len(), 5);
    assert_eq!(first.mirroring_ratios.len(), 15);
    assert!(first.max_dual_residual <= 1.0e-8, "{first:?}");
    assert_eq!(first.non_finite_values, 0, "{first:?}");
    assert!(first.atoms.iter().all(|atom| {
        atom.total_energy.is_finite()
            && atom.total_energy > 0.0
            && atom.tail_energy_ratios.len() == 6
            && atom.guard_lower_bounds.len() == 4
            && atom
                .tail_energy_ratios
                .iter()
                .all(|value| value.is_finite())
    }));
    assert!(first
        .tightening_ratios
        .iter()
        .chain(&first.dualization_ratios)
        .chain(&first.mirroring_ratios)
        .all(|value| value.is_finite() || value.is_infinite()));
    eprintln!("tail_attribution {first:?}");
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

fn two_tone_control() -> Vec<Sample> {
    (0..768)
        .map(|index| {
            let time = index as f64 / f64::from(SAMPLE_RATE.0);
            (0.3 * (std::f64::consts::TAU * 440.0 * time).sin()
                + 0.2 * (std::f64::consts::TAU * 4_000.0 * time).sin()) as f32
        })
        .collect()
}

fn chirp_control(exponential: bool) -> Vec<Sample> {
    let mut phase = 0.0_f64;
    (0..768)
        .map(|index| {
            let position = index as f64 / 767.0;
            let frequency = if exponential {
                200.0_f64 * (8_000.0_f64 / 200.0).powf(position)
            } else {
                200.0 + (8_000.0 - 200.0) * position
            };
            phase += std::f64::consts::TAU * frequency / f64::from(SAMPLE_RATE.0);
            (0.5 * phase.sin()) as f32
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
