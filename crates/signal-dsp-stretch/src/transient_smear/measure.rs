use signal_primitives::Sample;

use super::detect::detect_stretch_transients_with_policy;
use super::features::{nearest_transient, transient_attack_width};
use super::types::{
    RawTransientSmearMeasurement, SelectorTransientSmearMeasurement,
    StretchTransientDetectorPolicy, StretchTransientEvent,
};

#[cfg(any(test, feature = "evidence"))]
use super::types::{StretchTransientSmearMeasurement, StretchTransientSmearPolicies};
#[cfg(any(test, feature = "evidence"))]
use crate::benchmark::{StretchMetric, StretchMetricValue};

/// Selector measurement that reuses already-detected source transients.
///
/// The expansion selector measures the current output and a draft baseline
/// against the same source with the same policy and geometry, so detecting
/// source transients twice was pure duplication.
pub(crate) fn measure_selector_transient_smear_with_input_events(
    input: &[Sample],
    input_events: &[StretchTransientEvent],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
) -> SelectorTransientSmearMeasurement {
    let measurement = measure_raw_transient_smear_with_input_events(
        input,
        input_events,
        output,
        ratio,
        window_size,
        hop_size,
        StretchTransientDetectorPolicy::production(),
        Some(StretchTransientDetectorPolicy::candidate_review()),
    );
    SelectorTransientSmearMeasurement {
        missed_transients: measurement.missed_transients,
        max_smear_frames: measurement.max_smear_frames,
    }
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

/// Measure transient attack widening between input and stretched output.
#[cfg(any(test, feature = "evidence"))]
pub fn measure_transient_smear(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
    policies: StretchTransientSmearPolicies,
) -> StretchTransientSmearMeasurement {
    evidence_measurement(measure_raw_transient_smear(
        input,
        output,
        ratio,
        window_size,
        hop_size,
        policies.input,
        policies.output,
        policies.output_recovery,
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
    measure_raw_transient_smear_with_input_events(
        input,
        &input_events,
        output,
        ratio,
        window_size,
        hop_size,
        output_policy,
        recovery_output_policy,
    )
}

#[allow(clippy::too_many_arguments)]
fn measure_raw_transient_smear_with_input_events(
    input: &[Sample],
    input_events: &[StretchTransientEvent],
    output: &[Sample],
    ratio: f64,
    window_size: usize,
    hop_size: usize,
    output_policy: StretchTransientDetectorPolicy,
    recovery_output_policy: Option<StretchTransientDetectorPolicy>,
) -> RawTransientSmearMeasurement {
    if !ratio.is_finite() || ratio <= 0.0 || input.is_empty() || output.is_empty() {
        return raw_transient_smear_nan(ratio);
    }
    let input_events = input_events.to_vec();
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
