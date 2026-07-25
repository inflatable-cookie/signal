use rustfft::{num_complex::Complex32, FftPlanner};
use signal_primitives::Sample;

#[cfg(any(test, feature = "evidence"))]
use crate::benchmark::{StretchMetric, StretchMetricValue};

/// One detected transient candidate in stretch audio.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientEvent {
    /// Sample-frame index at the beginning of the detected analysis frame.
    pub frame_index: usize,
    /// Normalized positive frame-energy rise score.
    pub energy_score: f64,
    /// Normalized positive spectral-flux score.
    pub spectral_flux_score: f64,
    /// Combined detector score. Higher means a stronger transient candidate.
    pub combined_score: f64,
}

/// Threshold policy for stretch transient detection.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientDetectorPolicy {
    /// Minimum combined normalized energy-rise plus spectral-flux score.
    pub minimum_combined_score: f64,
    /// Minimum normalized spectral-flux score.
    pub minimum_spectral_flux_score: f64,
}

impl StretchTransientDetectorPolicy {
    /// Current production selector and metric policy.
    pub const fn production() -> Self {
        Self {
            minimum_combined_score: 3.0,
            minimum_spectral_flux_score: 2.0,
        }
    }

    /// Recovery policy used when the production output detector misses.
    pub const fn candidate_review() -> Self {
        Self {
            minimum_combined_score: 2.0,
            minimum_spectral_flux_score: 1.5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct SelectorTransientSmearMeasurement {
    pub(crate) missed_transients: usize,
    pub(crate) max_smear_frames: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct RawTransientSmearMeasurement {
    ratio: f64,
    input_transients: usize,
    output_transients: usize,
    matched_transients: usize,
    missed_transients: usize,
    mean_smear_frames: f64,
    max_smear_frames: f64,
    max_matched_smear_frames: f64,
    max_matched_input_frame: f64,
    max_matched_output_frame: f64,
    max_matched_input_width_frames: f64,
    max_matched_output_width_frames: f64,
}

/// Transient smear measurement for one rendered stretch output.
#[cfg(any(test, feature = "evidence"))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchTransientSmearMeasurement {
    /// Output/input duration ratio measured.
    pub ratio: f64,
    /// Number of detected input transient candidates.
    pub input_transients: usize,
    /// Number of detected output transient candidates.
    pub output_transients: usize,
    /// Number of input transients matched to stretched-output transients.
    pub matched_transients: usize,
    /// Number of input transients with no output transient match.
    pub missed_transients: usize,
    /// Mean positive attack widening in sample frames.
    pub mean_smear_frames: f64,
    /// Worst positive attack widening in sample frames.
    pub max_smear_frames: f64,
    /// Worst positive attack widening among matched transient events only.
    pub max_matched_smear_frames: f64,
    /// Input frame for the worst matched transient smear event.
    pub max_matched_input_frame: f64,
    /// Output frame for the worst matched transient smear event.
    pub max_matched_output_frame: f64,
    /// Input attack width for the worst matched transient smear event.
    pub max_matched_input_width_frames: f64,
    /// Output attack width for the worst matched transient smear event.
    pub max_matched_output_width_frames: f64,
    /// Metric reported to the acceptance harness.
    pub metric: StretchMetricValue,
}

pub(crate) fn measure_selector_transient_smear(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
) -> SelectorTransientSmearMeasurement {
    let measurement = measure_raw_transient_smear(
        input,
        output,
        ratio,
        window_size,
        hop_size,
        StretchTransientDetectorPolicy::production(),
        StretchTransientDetectorPolicy::production(),
        Some(StretchTransientDetectorPolicy::candidate_review()),
    );
    SelectorTransientSmearMeasurement {
        missed_transients: measurement.missed_transients,
        max_smear_frames: measurement.max_smear_frames,
    }
}

/// Detect transient candidates from frame energy rise and positive spectral
/// flux. This is a measurement primitive only; it does not change synthesis.
#[cfg(any(test, feature = "evidence"))]
pub fn detect_stretch_transients(
    samples: &[Sample],
    window_size: usize,
    hop_size: usize,
) -> Vec<StretchTransientEvent> {
    detect_stretch_transients_with_policy(
        samples,
        window_size,
        hop_size,
        StretchTransientDetectorPolicy::production(),
    )
}

/// Detect transient candidates using an explicit threshold policy.
///
/// This is a measurement primitive only. Candidate policies are for corpus
/// evidence and review gates; they do not change synthesis.
pub fn detect_stretch_transients_with_policy(
    samples: &[Sample],
    window_size: usize,
    hop_size: usize,
    policy: StretchTransientDetectorPolicy,
) -> Vec<StretchTransientEvent> {
    if samples.len() < window_size || window_size < 16 || hop_size == 0 {
        return Vec::new();
    }

    let frame_features = transient_frame_features(samples, window_size, hop_size);
    if frame_features.len() < 3 {
        return Vec::new();
    }

    let mut energy_rises = Vec::with_capacity(frame_features.len());
    let mut fluxes = Vec::with_capacity(frame_features.len());
    energy_rises.push(0.0);
    fluxes.push(0.0);
    for pair in frame_features.windows(2) {
        energy_rises.push((pair[1].energy - pair[0].energy).max(0.0));
        fluxes.push(pair[1].spectral_flux);
    }

    let energy_scale = mean_plus_stddev(&energy_rises).max(1.0e-12);
    let flux_scale = mean_plus_stddev(&fluxes).max(1.0e-12);
    let mut events = Vec::new();

    for index in 1..frame_features.len() - 1 {
        let energy_score = energy_rises[index] / energy_scale;
        let flux_score = fluxes[index] / flux_scale;
        let combined_score = energy_score + flux_score;
        let previous_score =
            energy_rises[index - 1] / energy_scale + fluxes[index - 1] / flux_scale;
        let next_score = energy_rises[index + 1] / energy_scale + fluxes[index + 1] / flux_scale;
        if combined_score >= policy.minimum_combined_score
            && combined_score >= previous_score
            && combined_score > next_score
            && flux_score >= policy.minimum_spectral_flux_score
        {
            events.push(StretchTransientEvent {
                frame_index: frame_features[index].frame_index,
                energy_score,
                spectral_flux_score: flux_score,
                combined_score,
            });
        }
    }

    merge_nearby_transients(events, hop_size * 2)
}

/// Measure transient attack widening between input and stretched output.
#[cfg(any(test, feature = "evidence"))]
pub fn measure_transient_smear(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
) -> StretchTransientSmearMeasurement {
    measure_transient_smear_with_output_recovery_policy(
        input,
        output,
        ratio,
        window_size,
        hop_size,
        StretchTransientDetectorPolicy::production(),
        StretchTransientDetectorPolicy::production(),
        StretchTransientDetectorPolicy::candidate_review(),
    )
}

/// Measure transient attack widening with an explicit detector policy.
#[cfg(any(test, feature = "evidence"))]
pub fn measure_transient_smear_with_policy(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
    policy: StretchTransientDetectorPolicy,
) -> StretchTransientSmearMeasurement {
    measure_transient_smear_with_policies(
        input,
        output,
        ratio,
        window_size,
        hop_size,
        policy,
        policy,
    )
}

/// Measure transient attack widening with separate input and output detector
/// policies.
#[cfg(any(test, feature = "evidence"))]
pub fn measure_transient_smear_with_policies(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
    input_policy: StretchTransientDetectorPolicy,
    output_policy: StretchTransientDetectorPolicy,
) -> StretchTransientSmearMeasurement {
    evidence_measurement(measure_raw_transient_smear(
        input,
        output,
        ratio,
        window_size,
        hop_size,
        input_policy,
        output_policy,
        None,
    ))
}

/// Measure transient smear with a fallback output detector for primary misses.
#[cfg(any(test, feature = "evidence"))]
#[allow(clippy::too_many_arguments)]
pub fn measure_transient_smear_with_output_recovery_policy(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
    input_policy: StretchTransientDetectorPolicy,
    output_policy: StretchTransientDetectorPolicy,
    recovery_output_policy: StretchTransientDetectorPolicy,
) -> StretchTransientSmearMeasurement {
    evidence_measurement(measure_raw_transient_smear(
        input,
        output,
        ratio,
        window_size,
        hop_size,
        input_policy,
        output_policy,
        Some(recovery_output_policy),
    ))
}

#[allow(clippy::too_many_arguments)]
fn measure_raw_transient_smear(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
    input_policy: StretchTransientDetectorPolicy,
    output_policy: StretchTransientDetectorPolicy,
    recovery_output_policy: Option<StretchTransientDetectorPolicy>,
) -> RawTransientSmearMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 || input.is_empty() || output.is_empty() {
        return raw_transient_smear_nan(ratio);
    }

    let input_events =
        detect_stretch_transients_with_policy(input, window_size, hop_size, input_policy);
    let output_events =
        detect_stretch_transients_with_policy(output, window_size, hop_size, output_policy);
    let recovery_output_events = recovery_output_policy
        .map(|policy| detect_stretch_transients_with_policy(output, window_size, hop_size, policy));
    let mut matched = 0usize;
    let mut smear_sum = 0.0f64;
    let mut max_smear = 0.0f64;
    let mut max_matched_smear = f64::NAN;
    let mut max_matched_input_frame = f64::NAN;
    let mut max_matched_output_frame = f64::NAN;
    let mut max_matched_input_width = f64::NAN;
    let mut max_matched_output_width = f64::NAN;
    let tolerance = window_size.max(hop_size * 4) as f64;

    for input_event in &input_events {
        let expected_output_frame = input_event.frame_index as f64 * ratio;
        let output_event = nearest_transient(&output_events, expected_output_frame, tolerance)
            .or_else(|| {
                recovery_output_events
                    .as_ref()
                    .and_then(|events| nearest_transient(events, expected_output_frame, tolerance))
            });
        let Some(output_event) = output_event else {
            continue;
        };
        let input_width = transient_attack_width(input, input_event.frame_index, window_size);
        let output_width = transient_attack_width(output, output_event.frame_index, window_size);
        if !input_width.is_finite() || !output_width.is_finite() {
            continue;
        }
        let smear = (output_width - input_width).max(0.0);
        matched += 1;
        smear_sum += smear;
        if !max_matched_smear.is_finite() || smear > max_matched_smear {
            max_matched_smear = smear;
            max_matched_input_frame = input_event.frame_index as f64;
            max_matched_output_frame = output_event.frame_index as f64;
            max_matched_input_width = input_width;
            max_matched_output_width = output_width;
        }
        max_smear = max_smear.max(smear);
    }

    let missed = input_events.len().saturating_sub(matched);
    let missed_penalty = window_size as f64;
    let total_measured = matched + missed;
    let mean_smear = if total_measured > 0 {
        (smear_sum + missed_penalty * missed as f64) / total_measured as f64
    } else {
        f64::NAN
    };
    let max_smear = if total_measured > 0 {
        if missed > 0 {
            max_smear.max(missed_penalty)
        } else {
            max_smear
        }
    } else {
        f64::NAN
    };

    RawTransientSmearMeasurement {
        ratio,
        input_transients: input_events.len(),
        output_transients: output_events.len(),
        matched_transients: matched,
        missed_transients: missed,
        mean_smear_frames: mean_smear,
        max_smear_frames: max_smear,
        max_matched_smear_frames: max_matched_smear,
        max_matched_input_frame,
        max_matched_output_frame,
        max_matched_input_width_frames: max_matched_input_width,
        max_matched_output_width_frames: max_matched_output_width,
    }
}

#[cfg(any(test, feature = "evidence"))]
fn evidence_measurement(raw: RawTransientSmearMeasurement) -> StretchTransientSmearMeasurement {
    StretchTransientSmearMeasurement {
        ratio: raw.ratio,
        input_transients: raw.input_transients,
        output_transients: raw.output_transients,
        matched_transients: raw.matched_transients,
        missed_transients: raw.missed_transients,
        mean_smear_frames: raw.mean_smear_frames,
        max_smear_frames: raw.max_smear_frames,
        max_matched_smear_frames: raw.max_matched_smear_frames,
        max_matched_input_frame: raw.max_matched_input_frame,
        max_matched_output_frame: raw.max_matched_output_frame,
        max_matched_input_width_frames: raw.max_matched_input_width_frames,
        max_matched_output_width_frames: raw.max_matched_output_width_frames,
        metric: StretchMetricValue::new(StretchMetric::TransientSmearFrames, raw.max_smear_frames),
    }
}

#[cfg(any(test, feature = "evidence"))]
pub(crate) fn transient_smear_nan(ratio: f64) -> StretchTransientSmearMeasurement {
    evidence_measurement(raw_transient_smear_nan(ratio))
}

fn raw_transient_smear_nan(ratio: f64) -> RawTransientSmearMeasurement {
    RawTransientSmearMeasurement {
        ratio,
        input_transients: 0,
        output_transients: 0,
        matched_transients: 0,
        missed_transients: 0,
        mean_smear_frames: f64::NAN,
        max_smear_frames: f64::NAN,
        max_matched_smear_frames: f64::NAN,
        max_matched_input_frame: f64::NAN,
        max_matched_output_frame: f64::NAN,
        max_matched_input_width_frames: f64::NAN,
        max_matched_output_width_frames: f64::NAN,
    }
}

#[derive(Clone, Copy, Debug)]
struct TransientFrameFeature {
    frame_index: usize,
    energy: f64,
    spectral_flux: f64,
}

fn transient_frame_features(
    samples: &[Sample],
    window_size: usize,
    hop_size: usize,
) -> Vec<TransientFrameFeature> {
    let bins = window_size / 2 + 1;
    let window: Vec<f32> = (0..window_size)
        .map(|index| 0.5 - 0.5 * (std::f32::consts::TAU * index as f32 / window_size as f32).cos())
        .collect();
    let mut planner = FftPlanner::<f32>::new();
    let forward = planner.plan_fft_forward(window_size);
    let mut buffer = vec![Complex32::new(0.0, 0.0); window_size];
    let mut previous_magnitudes = vec![0.0f32; bins];
    let mut magnitudes = vec![0.0f32; bins];
    let mut features = Vec::new();

    for start in (0..=samples.len() - window_size).step_by(hop_size) {
        let mut energy = 0.0f64;
        for (slot, (sample, weight)) in buffer.iter_mut().zip(
            samples[start..start + window_size]
                .iter()
                .zip(window.iter()),
        ) {
            let windowed = sample * weight;
            energy += (windowed * windowed) as f64;
            *slot = Complex32::new(windowed, 0.0);
        }
        forward.process(&mut buffer);

        let mut flux = 0.0f64;
        for bin in 0..bins {
            let magnitude = buffer[bin].norm();
            magnitudes[bin] = magnitude;
            flux += (magnitude - previous_magnitudes[bin]).max(0.0) as f64;
        }
        previous_magnitudes.copy_from_slice(&magnitudes);

        features.push(TransientFrameFeature {
            frame_index: start,
            energy: energy / window_size as f64,
            spectral_flux: flux / bins as f64,
        });
    }

    features
}

fn mean_plus_stddev(values: &[f64]) -> f64 {
    if values.is_empty() {
        return f64::NAN;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance = values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / values.len() as f64;
    mean + variance.sqrt()
}

fn merge_nearby_transients(
    events: Vec<StretchTransientEvent>,
    merge_distance_frames: usize,
) -> Vec<StretchTransientEvent> {
    let mut merged = Vec::<StretchTransientEvent>::new();
    for event in events {
        if let Some(last) = merged.last_mut() {
            if event.frame_index.saturating_sub(last.frame_index) <= merge_distance_frames {
                if event.combined_score > last.combined_score {
                    *last = event;
                }
                continue;
            }
        }
        merged.push(event);
    }
    merged
}

fn nearest_transient(
    events: &[StretchTransientEvent],
    expected_frame: f64,
    tolerance_frames: f64,
) -> Option<StretchTransientEvent> {
    events
        .iter()
        .copied()
        .filter_map(|event| {
            let distance = (event.frame_index as f64 - expected_frame).abs();
            if distance <= tolerance_frames {
                Some((distance, event))
            } else {
                None
            }
        })
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(_, event)| event)
}

fn transient_attack_width(samples: &[Sample], event_frame: usize, search_radius: usize) -> f64 {
    if samples.is_empty() {
        return f64::NAN;
    }

    let start = event_frame.saturating_sub(search_radius);
    let end = (event_frame + search_radius).min(samples.len().saturating_sub(1));
    if start >= end {
        return f64::NAN;
    }

    let mut peak_index = start;
    let mut peak = 0.0f32;
    for (offset, sample) in samples[start..=end].iter().enumerate() {
        let magnitude = sample.abs();
        if magnitude > peak {
            peak = magnitude;
            peak_index = start + offset;
        }
    }
    if peak <= 1.0e-6 {
        return f64::NAN;
    }

    let threshold = peak * 0.5;
    let mut left = peak_index;
    while left > start && samples[left - 1].abs() >= threshold {
        left -= 1;
    }
    let mut right = peak_index;
    while right < end && samples[right + 1].abs() >= threshold {
        right += 1;
    }

    (right - left + 1) as f64
}
