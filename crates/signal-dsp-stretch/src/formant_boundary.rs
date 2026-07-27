use crate::spectral_support::plan_forward_analysis;
use crate::Sample;

mod spectral;

use spectral::{envelope_centroid_hz, envelope_residual, smoothed_spectral_envelope, window_fits};

const SPECTRAL_WINDOW_SIZE: usize = 4_096;
const SPECTRAL_SAMPLE_COUNT: usize = 24;
const FORMANT_LOW_HZ: f64 = 80.0;
const FORMANT_HIGH_HZ: f64 = 5_000.0;
const FORMANT_SMOOTHING_HZ: f64 = 300.0;
const BOUNDARY_SOURCE_FRAMES: usize = 2_048;
const BOUNDARY_ACTIVITY_FLOOR: f64 = 1.0e-6;

/// Source-relative formant-envelope and render-boundary evidence.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchFormantBoundaryMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Sample rate used to define the formant band and smoothing width.
    pub sample_rate_hz: u32,
    /// Number of ratio-projected spectral-envelope windows measured.
    pub envelope_windows: usize,
    /// Mean normalized residual between source and output spectral envelopes.
    pub mean_envelope_residual_ratio: f64,
    /// Largest normalized residual between source and output spectral envelopes.
    pub max_envelope_residual_ratio: f64,
    /// Mean absolute spectral-envelope centroid shift, in hertz.
    pub mean_envelope_centroid_shift_hz: f64,
    /// Largest absolute spectral-envelope centroid shift, in hertz.
    pub max_envelope_centroid_shift_hz: f64,
    /// Number of active source/output boundaries measured.
    pub measured_boundary_count: u8,
    /// Output head step crest minus source head step crest, in decibels.
    pub head_boundary_step_crest_delta_db: f64,
    /// Output tail step crest minus source tail step crest, in decibels.
    pub tail_boundary_step_crest_delta_db: f64,
    /// Largest positive head/tail step-crest growth, in decibels.
    pub max_boundary_step_crest_growth_db: f64,
    /// Output transition from digital silence into the first sample, in dBFS.
    pub head_boundary_step_dbfs: f64,
    /// Output transition from the final sample into digital silence, in dBFS.
    pub tail_boundary_step_dbfs: f64,
    /// Largest output head/tail sample step, in dBFS.
    pub max_boundary_step_dbfs: f64,
}

/// Measure broad spectral-envelope preservation and boundary-step growth.
///
/// Spectral envelopes are smoothed over `300 Hz`, restricted to `80-5000 Hz`,
/// and L1-normalized before comparison. This makes the result insensitive to
/// whole-render gain and fine harmonic-bin motion. Boundary steps include the
/// transition from and to digital silence and are normalized by local RMS.
/// This is an allocating offline diagnostic, not an audio-thread surface.
pub fn measure_formant_boundary(
    source: &[Sample],
    output: &[Sample],
    ratio: f64,
    sample_rate_hz: u32,
) -> StretchFormantBoundaryMeasurement {
    if !ratio.is_finite()
        || ratio <= 0.0
        || sample_rate_hz == 0
        || sample_rate_hz as f64 * 0.5 <= FORMANT_LOW_HZ
        || source.len() < SPECTRAL_WINDOW_SIZE
        || output.len() < SPECTRAL_WINDOW_SIZE
    {
        return invalid_measurement(ratio, sample_rate_hz);
    }

    let (fft, window) = plan_forward_analysis(SPECTRAL_WINDOW_SIZE);
    let mut residual_sum = 0.0;
    let mut residual_max = 0.0f64;
    let mut centroid_shift_sum = 0.0;
    let mut centroid_shift_max = 0.0f64;
    let mut measured = 0usize;

    for source_center in evenly_spaced_centers(source.len(), ratio, output.len()) {
        let output_center = (source_center as f64 * ratio).round() as usize;
        let source_envelope =
            smoothed_spectral_envelope(source, source_center, &window, fft.clone(), sample_rate_hz);
        let output_envelope =
            smoothed_spectral_envelope(output, output_center, &window, fft.clone(), sample_rate_hz);
        let residual = envelope_residual(&source_envelope, &output_envelope);
        let source_centroid = envelope_centroid_hz(&source_envelope, sample_rate_hz);
        let output_centroid = envelope_centroid_hz(&output_envelope, sample_rate_hz);
        let centroid_shift = (output_centroid - source_centroid).abs();
        let source_supported = source_envelope.iter().sum::<f64>() > 0.5;
        if source_supported && residual.is_finite() && centroid_shift.is_finite() {
            residual_sum += residual;
            residual_max = residual_max.max(residual);
            centroid_shift_sum += centroid_shift;
            centroid_shift_max = centroid_shift_max.max(centroid_shift);
            measured += 1;
        }
    }

    let source_boundary_frames = BOUNDARY_SOURCE_FRAMES.min(source.len());
    let output_boundary_frames = ((source_boundary_frames as f64 * ratio).round() as usize)
        .max(1)
        .min(output.len());
    let source_head = boundary_step(source, source_boundary_frames, true);
    let source_tail = boundary_step(source, source_boundary_frames, false);
    let output_head = boundary_step(output, output_boundary_frames, true);
    let output_tail = boundary_step(output, output_boundary_frames, false);
    let head_delta = active_boundary_delta(source_head, output_head);
    let tail_delta = active_boundary_delta(source_tail, output_tail);
    let measured_boundary_count = head_delta.is_finite() as u8 + tail_delta.is_finite() as u8;
    let max_boundary_growth = [head_delta, tail_delta]
        .into_iter()
        .filter(|delta| delta.is_finite())
        .fold(f64::NAN, |current, delta| {
            if current.is_nan() {
                delta.max(0.0)
            } else {
                current.max(delta)
            }
        });

    StretchFormantBoundaryMeasurement {
        ratio,
        sample_rate_hz,
        envelope_windows: measured,
        mean_envelope_residual_ratio: finite_mean(residual_sum, measured),
        max_envelope_residual_ratio: measured_value(residual_max, measured),
        mean_envelope_centroid_shift_hz: finite_mean(centroid_shift_sum, measured),
        max_envelope_centroid_shift_hz: measured_value(centroid_shift_max, measured),
        measured_boundary_count,
        head_boundary_step_crest_delta_db: head_delta,
        tail_boundary_step_crest_delta_db: tail_delta,
        max_boundary_step_crest_growth_db: max_boundary_growth,
        head_boundary_step_dbfs: amplitude_dbfs(output_head.peak_step),
        tail_boundary_step_dbfs: amplitude_dbfs(output_tail.peak_step),
        max_boundary_step_dbfs: amplitude_dbfs(output_head.peak_step.max(output_tail.peak_step)),
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

#[derive(Clone, Copy)]
struct BoundaryStep {
    crest_db: f64,
    rms: f64,
    peak_step: f64,
}

fn boundary_step(samples: &[Sample], frames: usize, head: bool) -> BoundaryStep {
    let slice = if head {
        &samples[..frames]
    } else {
        &samples[samples.len() - frames..]
    };
    let rms = (slice
        .iter()
        .map(|sample| (*sample as f64).powi(2))
        .sum::<f64>()
        / slice.len() as f64)
        .sqrt();
    let peak_step = if head {
        slice[0].abs() as f64
    } else {
        slice[slice.len() - 1].abs() as f64
    };
    BoundaryStep {
        crest_db: if peak_step > 0.0 && rms > 0.0 {
            20.0 * (peak_step / rms).log10()
        } else {
            -240.0
        },
        rms,
        peak_step,
    }
}

fn active_boundary_delta(source: BoundaryStep, output: BoundaryStep) -> f64 {
    if source.rms > BOUNDARY_ACTIVITY_FLOOR || output.peak_step > BOUNDARY_ACTIVITY_FLOOR {
        output.crest_db - source.crest_db
    } else {
        f64::NAN
    }
}

fn amplitude_dbfs(amplitude: f64) -> f64 {
    if amplitude > 0.0 {
        20.0 * amplitude.log10()
    } else {
        -240.0
    }
}

fn finite_mean(sum: f64, count: usize) -> f64 {
    if count == 0 {
        f64::NAN
    } else {
        sum / count as f64
    }
}

fn measured_value(value: f64, count: usize) -> f64 {
    if count == 0 {
        f64::NAN
    } else {
        value
    }
}

fn invalid_measurement(ratio: f64, sample_rate_hz: u32) -> StretchFormantBoundaryMeasurement {
    StretchFormantBoundaryMeasurement {
        ratio,
        sample_rate_hz,
        envelope_windows: 0,
        mean_envelope_residual_ratio: f64::NAN,
        max_envelope_residual_ratio: f64::NAN,
        mean_envelope_centroid_shift_hz: f64::NAN,
        max_envelope_centroid_shift_hz: f64::NAN,
        measured_boundary_count: 0,
        head_boundary_step_crest_delta_db: f64::NAN,
        tail_boundary_step_crest_delta_db: f64::NAN,
        max_boundary_step_crest_growth_db: f64::NAN,
        head_boundary_step_dbfs: f64::NAN,
        tail_boundary_step_dbfs: f64::NAN,
        max_boundary_step_dbfs: f64::NAN,
    }
}

#[cfg(test)]
#[path = "formant_boundary/tests.rs"]
mod tests;
