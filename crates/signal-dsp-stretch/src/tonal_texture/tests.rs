use super::*;

fn tone(frequency: f32, frames: usize) -> Vec<Sample> {
    (0..frames)
        .map(|frame| (std::f32::consts::TAU * frequency * frame as f32 / 48_000.0).sin())
        .collect()
}

#[test]
fn tonal_texture_identity_has_no_residual_or_added_modulation() {
    let source = tone(440.0, 48_000);
    let measurement = measure_tonal_texture(&source, &source, 1.0);

    assert!(measurement.spectral_windows > 0);
    assert!(measurement.mean_spectral_residual_ratio < 1.0e-9);
    assert!(measurement.mean_added_sideband_ratio < 1.0e-9);
    assert!(measurement.spectral_modulation_delta.abs() < 1.0e-9);
    assert!(measurement.envelope_modulation_delta_db.abs() < 1.0e-9);
}

#[test]
fn tonal_texture_projects_source_windows_into_expanded_output() {
    let source = tone(468.75, 48_000);
    let output = tone(468.75, 72_000);
    let measurement = measure_tonal_texture(&source, &output, 1.5);

    assert_eq!(measurement.spectral_windows, SPECTRAL_SAMPLE_COUNT);
    assert!(
        measurement.mean_spectral_residual_ratio < 2.0e-3,
        "{measurement:?}"
    );
    assert!(
        measurement.mean_added_sideband_ratio < 2.0e-3,
        "{measurement:?}"
    );
}

#[test]
fn tonal_texture_detects_added_inharmonic_sideband() {
    let source = tone(440.0, 48_000);
    let sideband = tone(613.0, 48_000);
    let output = source
        .iter()
        .zip(sideband)
        .map(|(source, sideband)| source + sideband * 0.08)
        .collect::<Vec<_>>();
    let measurement = measure_tonal_texture(&source, &output, 1.0);

    assert!(measurement.mean_spectral_residual_ratio > 0.02);
    assert!(measurement.mean_added_sideband_ratio > 0.01);
}

#[test]
fn tonal_texture_detects_energy_added_over_silence() {
    let source = vec![0.0; 48_000];
    let output = tone(440.0, 48_000);
    let measurement = measure_tonal_texture(&source, &output, 1.0);

    assert!(measurement.mean_spectral_residual_ratio > 0.49);
    assert!(measurement.mean_added_sideband_ratio > 0.99);
}

#[test]
fn tonal_texture_detects_fast_envelope_modulation() {
    let source = tone(440.0, 48_000);
    let output = source
        .iter()
        .enumerate()
        .map(|(frame, sample)| {
            let modulation =
                0.75 + 0.25 * (std::f32::consts::TAU * 60.0 * frame as f32 / 48_000.0).sin();
            sample * modulation
        })
        .collect::<Vec<_>>();
    let measurement = measure_tonal_texture(&source, &output, 1.0);

    assert!(measurement.envelope_modulation_delta_db > 0.1);
}
