use super::*;

const SAMPLE_RATE: f32 = 48_000.0;
const CONTROL_LEN: usize = 24_000;
const RATIO: f64 = 1.5;

#[test]
fn phase_gradient_identity_is_bit_exact() {
    let input = steady_sine(440.0);
    let render = stretch_phase_gradient_review_mono(&input, 1.0);
    assert_eq!(render.samples, input);
}

#[test]
fn phase_gradient_sine_is_exact_bounded_and_horizontal() {
    assert_kernel_gate(&steady_sine(440.0), true, false);
}

#[test]
fn phase_gradient_two_tone_is_exact_bounded_and_horizontal() {
    let input = (0..CONTROL_LEN)
        .map(|index| {
            let time = index as f32 / SAMPLE_RATE;
            0.25 * (std::f32::consts::TAU * 330.0 * time).sin()
                + 0.25 * (std::f32::consts::TAU * 880.0 * time).sin()
        })
        .collect::<Vec<_>>();
    assert_kernel_gate(&input, true, false);
}

#[test]
fn phase_gradient_chirp_uses_vertical_integration() {
    let input = (0..CONTROL_LEN)
        .map(|index| {
            let time = index as f32 / SAMPLE_RATE;
            let phase = std::f32::consts::TAU * (180.0 * time + 0.5 * 2_400.0 * time * time);
            0.5 * phase.sin()
        })
        .collect::<Vec<_>>();
    assert_kernel_gate(&input, false, true);
}

#[test]
fn phase_gradient_impulse_uses_vertical_integration() {
    let mut input = vec![0.0; CONTROL_LEN];
    input[CONTROL_LEN / 2] = 1.0;
    assert_kernel_gate(&input, false, true);
}

#[test]
fn phase_gradient_silence_is_finite_covered_and_exact() {
    let input = vec![0.0; CONTROL_LEN];
    let render = stretch_phase_gradient_review_mono(&input, RATIO);
    assert_eq!(
        render.samples.len(),
        (CONTROL_LEN as f64 * RATIO).round() as usize
    );
    assert!(render.samples.iter().all(|sample| *sample == 0.0));
    assert_eq!(render.evidence.significant_bins, 0);
    assert_eq!(render.evidence.missing_assignments, 0);
    assert_eq!(render.evidence.uncovered_output_samples, 0);
    assert!(render.evidence.derivatives_finite);
    assert!(render.evidence.all_samples_finite);
}

#[test]
fn phase_gradient_trace_and_samples_are_deterministic() {
    let input = steady_sine(523.25);
    let first = stretch_phase_gradient_review_mono(&input, RATIO);
    let repeated = stretch_phase_gradient_review_mono(&input, RATIO);
    assert_eq!(first, repeated);
}

#[test]
fn phase_gradient_compression_is_exact_covered_and_deterministic() {
    let input = steady_sine(261.63);
    let first = stretch_phase_gradient_review_mono(&input, 0.75);
    let repeated = stretch_phase_gradient_review_mono(&input, 0.75);
    assert_eq!(first, repeated);
    assert_eq!(first.samples.len(), 18_000);
    assert_eq!(first.evidence.uncovered_output_samples, 0);
    assert_eq!(first.evidence.duplicate_assignments, 0);
    assert_eq!(first.evidence.missing_assignments, 0);
    assert!(first.evidence.all_samples_finite);
}

fn assert_kernel_gate(input: &[Sample], require_horizontal: bool, require_vertical: bool) {
    let render = stretch_phase_gradient_review_mono(input, RATIO);
    let evidence = &render.evidence;
    assert_eq!(
        render.samples.len(),
        (input.len() as f64 * RATIO).round() as usize
    );
    assert!(evidence.derivatives_finite);
    assert!(evidence.all_samples_finite);
    assert!(evidence.analysis_positions_monotonic);
    assert!(evidence.max_analysis_mapping_error_frames <= 0.5);
    assert!(evidence.final_analysis_mapping_error_frames.abs() <= 0.5);
    assert_eq!(
        evidence.analysis_interval_floor_count + evidence.analysis_interval_ceiling_count,
        evidence.synthesis_frames - 1
    );
    assert!(evidence.synthesis_positions_monotonic);
    assert_eq!(evidence.duplicate_assignments, 0);
    assert_eq!(evidence.missing_assignments, 0);
    assert!(evidence.heap_high_water <= evidence.heap_capacity_bound);
    assert!(evidence.max_conjugate_symmetry_error <= 1.0e-6);
    assert_eq!(evidence.uncovered_output_samples, 0);
    if require_horizontal {
        assert!(evidence.horizontal_assignments > 0);
    }
    if require_vertical {
        assert!(evidence.vertical_assignments > 0);
    }
    eprintln!(
        "phase_gradient h={} v={} significant={} insignificant={} heap={}/{} hash={:016x}",
        evidence.horizontal_assignments,
        evidence.vertical_assignments,
        evidence.significant_bins,
        evidence.insignificant_bins,
        evidence.heap_high_water,
        evidence.heap_capacity_bound,
        evidence.sample_hash,
    );
}

fn steady_sine(frequency_hz: f32) -> Vec<Sample> {
    (0..CONTROL_LEN)
        .map(|index| {
            (std::f32::consts::TAU * frequency_hz * index as f32 / SAMPLE_RATE).sin() * 0.5
        })
        .collect()
}
