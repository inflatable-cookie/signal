use super::*;

fn tone(frequency: f32, frames: usize) -> Vec<Sample> {
    (0..frames)
        .map(|frame| (std::f32::consts::TAU * frequency * frame as f32 / 48_000.0).sin())
        .collect()
}

fn vowel_like(formants: [f32; 3], frames: usize) -> Vec<Sample> {
    let mut samples = vec![0.0; frames];
    for harmonic in 1..=40 {
        let frequency = 120.0 * harmonic as f32;
        let weight = formants
            .iter()
            .map(|formant| {
                let distance = (frequency - formant) / 140.0;
                (-0.5 * distance * distance).exp()
            })
            .sum::<f32>()
            / harmonic as f32;
        for (frame, sample) in samples.iter_mut().enumerate() {
            *sample += weight * (std::f32::consts::TAU * frequency * frame as f32 / 48_000.0).sin();
        }
    }
    samples
}

#[test]
fn formant_boundary_identity_has_no_residual_or_boundary_growth() {
    let source = vowel_like([700.0, 1_200.0, 2_500.0], 48_000);
    let measurement = measure_formant_boundary(&source, &source, 1.0, 48_000);

    assert_eq!(measurement.envelope_windows, SPECTRAL_SAMPLE_COUNT);
    assert!(measurement.mean_envelope_residual_ratio < 1.0e-9);
    assert!(measurement.mean_envelope_centroid_shift_hz < 1.0e-9);
    assert_eq!(measurement.measured_boundary_count, 2);
    assert!(measurement.max_boundary_step_crest_growth_db < 1.0e-9);
}

#[test]
fn formant_boundary_ignores_whole_render_gain() {
    let source = vowel_like([700.0, 1_200.0, 2_500.0], 48_000);
    let output = source
        .iter()
        .map(|sample| sample * 0.25)
        .collect::<Vec<_>>();
    let measurement = measure_formant_boundary(&source, &output, 1.0, 48_000);

    assert!(measurement.mean_envelope_residual_ratio < 1.0e-7);
    assert!(measurement.mean_envelope_centroid_shift_hz < 1.0e-3);
    assert!(measurement.max_boundary_step_crest_growth_db < 1.0e-6);
}

#[test]
fn formant_boundary_detects_shifted_spectral_envelope() {
    let source = vowel_like([700.0, 1_200.0, 2_500.0], 48_000);
    let output = vowel_like([900.0, 1_500.0, 2_800.0], 48_000);
    let measurement = measure_formant_boundary(&source, &output, 1.0, 48_000);

    assert!(measurement.mean_envelope_residual_ratio > 0.1);
    assert!(measurement.mean_envelope_centroid_shift_hz > 100.0);
}

#[test]
fn formant_boundary_detects_added_edge_steps() {
    let source = tone(468.75, 48_000);
    let mut output = source.clone();
    output[0] = 4.0;
    *output.last_mut().expect("output tail") = -4.0;
    let measurement = measure_formant_boundary(&source, &output, 1.0, 48_000);

    assert!(measurement.head_boundary_step_crest_delta_db > 10.0);
    assert!(measurement.tail_boundary_step_crest_delta_db > 10.0);
    assert!(measurement.max_boundary_step_dbfs > 10.0);
}

#[test]
fn formant_boundary_ignores_inaudible_silent_edge_steps() {
    let source = vec![0.0; 48_000];
    let mut output = source.clone();
    output[0] = 1.0e-9;
    *output.last_mut().expect("output tail") = -1.0e-9;
    let measurement = measure_formant_boundary(&source, &output, 1.0, 48_000);

    assert_eq!(measurement.envelope_windows, 0);
    assert_eq!(measurement.measured_boundary_count, 0);
    assert!(measurement.max_boundary_step_crest_growth_db.is_nan());
    assert!(measurement.max_boundary_step_dbfs < -170.0);
}

#[test]
fn formant_boundary_projects_pitch_preserving_expansion() {
    let source = tone(468.75, 48_000);
    let output = tone(468.75, 72_000);
    let measurement = measure_formant_boundary(&source, &output, 1.5, 48_000);

    assert_eq!(measurement.envelope_windows, SPECTRAL_SAMPLE_COUNT);
    assert!(measurement.mean_envelope_residual_ratio < 2.0e-3);
    assert!(measurement.mean_envelope_centroid_shift_hz < 1.0);
}
