use std::sync::Arc;

use rustfft::Fft;

use crate::spectral_support::plan_forward_analysis;
use crate::Sample;

mod spectral;

use spectral::{
    added_sideband_ratio, normalized_spectral_distance, normalized_spectrum, window_fits,
};

const SPECTRAL_WINDOW_SIZE: usize = 4_096;
const SPECTRAL_SAMPLE_COUNT: usize = 24;
const MODULATION_CLUSTER_COUNT: usize = 4;
const MODULATION_FRAMES_PER_CLUSTER: usize = 8;
const MODULATION_SOURCE_HOP: usize = 256;
const ENVELOPE_WINDOW_SIZE: usize = 256;
const SIDEBAND_FLOOR_RATIO: f64 = 1.0e-3;

/// Source-relative tonal texture evidence for one stretched output.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTonalTextureMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Number of ratio-projected spectral windows measured.
    pub spectral_windows: usize,
    /// Mean normalized spectral residual from the source.
    pub mean_spectral_residual_ratio: f64,
    /// Largest normalized spectral residual from the source.
    pub max_spectral_residual_ratio: f64,
    /// Mean output energy added outside source-supported spectral bins.
    pub mean_added_sideband_ratio: f64,
    /// Largest output energy added outside source-supported spectral bins.
    pub max_added_sideband_ratio: f64,
    /// Mean source frame-to-frame normalized spectral movement.
    pub source_spectral_modulation: f64,
    /// Mean output frame-to-frame normalized spectral movement.
    pub output_spectral_modulation: f64,
    /// Output spectral movement minus source spectral movement.
    pub spectral_modulation_delta: f64,
    /// Mean source short-time RMS step, in decibels.
    pub source_envelope_modulation_db: f64,
    /// Mean output short-time RMS step, in decibels.
    pub output_envelope_modulation_db: f64,
    /// Output RMS-step modulation minus source modulation, in decibels.
    pub envelope_modulation_delta_db: f64,
}

/// Measure source-relative spectral residue, sidebands, and fast modulation.
///
/// Source windows are projected into output time through `ratio`. Spectra are
/// L1-normalized, so whole-render gain does not affect residual or sideband
/// evidence. Modulation is measured in short contiguous clusters rather than
/// across distant musical sections.
pub fn measure_tonal_texture(
    source: &[Sample],
    output: &[Sample],
    ratio: f64,
) -> StretchTonalTextureMeasurement {
    if !ratio.is_finite()
        || ratio <= 0.0
        || source.len() < SPECTRAL_WINDOW_SIZE
        || output.len() < SPECTRAL_WINDOW_SIZE
    {
        return invalid_measurement(ratio);
    }

    let (fft, window) = plan_forward_analysis(SPECTRAL_WINDOW_SIZE);
    let centers = evenly_spaced_centers(source.len(), ratio, output.len());
    let mut residual_sum = 0.0;
    let mut residual_max = 0.0f64;
    let mut sideband_sum = 0.0;
    let mut sideband_max = 0.0f64;
    let mut measured = 0usize;

    for source_center in centers {
        let output_center = (source_center as f64 * ratio).round() as usize;
        let source_spectrum = normalized_spectrum(source, source_center, &window, fft.clone());
        let output_spectrum = normalized_spectrum(output, output_center, &window, fft.clone());
        let residual = normalized_spectral_distance(&source_spectrum, &output_spectrum);
        let sideband = added_sideband_ratio(&source_spectrum, &output_spectrum);
        if residual.is_finite() && sideband.is_finite() {
            residual_sum += residual;
            residual_max = residual_max.max(residual);
            sideband_sum += sideband;
            sideband_max = sideband_max.max(sideband);
            measured += 1;
        }
    }

    let modulation = measure_modulation(source, output, ratio, &window, fft);
    StretchTonalTextureMeasurement {
        ratio,
        spectral_windows: measured,
        mean_spectral_residual_ratio: finite_ratio(residual_sum, measured),
        max_spectral_residual_ratio: if measured > 0 { residual_max } else { f64::NAN },
        mean_added_sideband_ratio: finite_ratio(sideband_sum, measured),
        max_added_sideband_ratio: if measured > 0 { sideband_max } else { f64::NAN },
        source_spectral_modulation: modulation.source_spectral,
        output_spectral_modulation: modulation.output_spectral,
        spectral_modulation_delta: modulation.output_spectral - modulation.source_spectral,
        source_envelope_modulation_db: modulation.source_envelope_db,
        output_envelope_modulation_db: modulation.output_envelope_db,
        envelope_modulation_delta_db: modulation.output_envelope_db - modulation.source_envelope_db,
    }
}

#[derive(Clone, Copy)]
struct ModulationMeasurement {
    source_spectral: f64,
    output_spectral: f64,
    source_envelope_db: f64,
    output_envelope_db: f64,
}

fn measure_modulation(
    source: &[Sample],
    output: &[Sample],
    ratio: f64,
    window: &[f32],
    fft: Arc<dyn Fft<f32>>,
) -> ModulationMeasurement {
    let radius = SPECTRAL_WINDOW_SIZE / 2;
    let usable_start = radius + MODULATION_SOURCE_HOP * MODULATION_FRAMES_PER_CLUSTER;
    let usable_end = source.len().saturating_sub(usable_start);
    if usable_end <= usable_start {
        return modulation_nan();
    }

    let mut source_spectral_sum = 0.0;
    let mut output_spectral_sum = 0.0;
    let mut source_envelope_sum = 0.0;
    let mut output_envelope_sum = 0.0;
    let mut steps = 0usize;
    for cluster in 0..MODULATION_CLUSTER_COUNT {
        let span = usable_end - usable_start;
        let center = usable_start + span * (cluster * 2 + 1) / (MODULATION_CLUSTER_COUNT * 2);
        let cluster_start =
            center.saturating_sub(MODULATION_SOURCE_HOP * MODULATION_FRAMES_PER_CLUSTER / 2);
        let mut previous_source_spectrum: Option<Vec<f64>> = None;
        let mut previous_output_spectrum: Option<Vec<f64>> = None;
        let mut previous_source_rms_db: Option<f64> = None;
        let mut previous_output_rms_db: Option<f64> = None;
        for frame in 0..MODULATION_FRAMES_PER_CLUSTER {
            let source_center = cluster_start + frame * MODULATION_SOURCE_HOP;
            let output_center = (source_center as f64 * ratio).round() as usize;
            if !window_fits(source.len(), source_center, SPECTRAL_WINDOW_SIZE)
                || !window_fits(output.len(), output_center, SPECTRAL_WINDOW_SIZE)
            {
                continue;
            }
            let source_spectrum = normalized_spectrum(source, source_center, window, fft.clone());
            let output_spectrum = normalized_spectrum(output, output_center, window, fft.clone());
            let source_rms_db = rms_db_at(source, source_center);
            let output_rms_db = rms_db_at(output, output_center);
            if let (
                Some(previous_source),
                Some(previous_output),
                Some(previous_source_db),
                Some(previous_output_db),
            ) = (
                previous_source_spectrum.as_ref(),
                previous_output_spectrum.as_ref(),
                previous_source_rms_db,
                previous_output_rms_db,
            ) {
                source_spectral_sum +=
                    normalized_spectral_distance(previous_source, &source_spectrum);
                output_spectral_sum +=
                    normalized_spectral_distance(previous_output, &output_spectrum);
                source_envelope_sum += (source_rms_db - previous_source_db).abs();
                output_envelope_sum += (output_rms_db - previous_output_db).abs();
                steps += 1;
            }
            previous_source_spectrum = Some(source_spectrum);
            previous_output_spectrum = Some(output_spectrum);
            previous_source_rms_db = Some(source_rms_db);
            previous_output_rms_db = Some(output_rms_db);
        }
    }

    ModulationMeasurement {
        source_spectral: finite_ratio(source_spectral_sum, steps),
        output_spectral: finite_ratio(output_spectral_sum, steps),
        source_envelope_db: finite_ratio(source_envelope_sum, steps),
        output_envelope_db: finite_ratio(output_envelope_sum, steps),
    }
}

fn evenly_spaced_centers(source_len: usize, ratio: f64, output_len: usize) -> Vec<usize> {
    let radius = SPECTRAL_WINDOW_SIZE / 2;
    let source_end = source_len.saturating_sub(radius);
    (0..SPECTRAL_SAMPLE_COUNT)
        .map(|index| radius + (source_end - radius) * (index * 2 + 1) / (SPECTRAL_SAMPLE_COUNT * 2))
        .filter(|center| {
            window_fits(source_len, *center, SPECTRAL_WINDOW_SIZE)
                && window_fits(
                    output_len,
                    (*center as f64 * ratio).round() as usize,
                    SPECTRAL_WINDOW_SIZE,
                )
        })
        .collect()
}

fn rms_db_at(samples: &[Sample], center: usize) -> f64 {
    let radius = ENVELOPE_WINDOW_SIZE / 2;
    let square_mean = samples[center - radius..center + radius]
        .iter()
        .map(|sample| (*sample as f64).powi(2))
        .sum::<f64>()
        / ENVELOPE_WINDOW_SIZE as f64;
    20.0 * (square_mean.sqrt() + 1.0e-12).log10()
}

fn finite_ratio(sum: f64, count: usize) -> f64 {
    if count == 0 {
        f64::NAN
    } else {
        sum / count as f64
    }
}

fn modulation_nan() -> ModulationMeasurement {
    ModulationMeasurement {
        source_spectral: f64::NAN,
        output_spectral: f64::NAN,
        source_envelope_db: f64::NAN,
        output_envelope_db: f64::NAN,
    }
}

fn invalid_measurement(ratio: f64) -> StretchTonalTextureMeasurement {
    StretchTonalTextureMeasurement {
        ratio,
        spectral_windows: 0,
        mean_spectral_residual_ratio: f64::NAN,
        max_spectral_residual_ratio: f64::NAN,
        mean_added_sideband_ratio: f64::NAN,
        max_added_sideband_ratio: f64::NAN,
        source_spectral_modulation: f64::NAN,
        output_spectral_modulation: f64::NAN,
        spectral_modulation_delta: f64::NAN,
        source_envelope_modulation_db: f64::NAN,
        output_envelope_modulation_db: f64::NAN,
        envelope_modulation_delta_db: f64::NAN,
    }
}

#[cfg(test)]
#[path = "tonal_texture/tests.rs"]
mod tests;
