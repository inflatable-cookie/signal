use signal_primitives::Sample;

const SILENCE_FLOOR_DB: f64 = -240.0;

/// Full-render correctness measurements independent of a comparator backend.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchRenderIntegrityMeasurement {
    /// Requested output/input duration ratio.
    pub ratio: f64,
    /// Input frame count.
    pub input_frames: usize,
    /// Output frame count.
    pub output_frames: usize,
    /// Absolute output-length error against the requested ratio.
    pub output_length_drift_frames: f64,
    /// Largest absolute head/tail RMS change, in decibels.
    pub endpoint_energy_delta_db: f64,
    /// Output silence beyond the ratio-scaled longest input silence run.
    pub added_silence_frames: usize,
    /// Positive output peak growth relative to the input, in decibels.
    pub peak_growth_db: f64,
    /// Input peak amplitude.
    pub input_peak: f64,
    /// Output peak amplitude.
    pub output_peak: f64,
    /// Input head RMS over the configured source endpoint span.
    pub input_head_rms: f64,
    /// Output head RMS over the ratio-scaled endpoint span.
    pub output_head_rms: f64,
    /// Input tail RMS over the configured source endpoint span.
    pub input_tail_rms: f64,
    /// Output tail RMS over the ratio-scaled endpoint span.
    pub output_tail_rms: f64,
    /// Longest input silence run at the configured threshold.
    pub longest_input_silence_frames: usize,
    /// Longest output silence run at the configured threshold.
    pub longest_output_silence_frames: usize,
}

/// Absolute limits for full-render stretch correctness.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchRenderIntegrityLimits {
    /// Maximum output-length error in frames.
    pub max_output_length_drift_frames: f64,
    /// Maximum absolute endpoint RMS change in decibels.
    pub max_endpoint_energy_delta_db: f64,
    /// Maximum added silence in frames.
    pub max_added_silence_frames: usize,
    /// Maximum positive peak growth in decibels.
    pub max_peak_growth_db: f64,
}

impl StretchRenderIntegrityLimits {
    /// Construct absolute full-render limits.
    pub const fn new(
        max_output_length_drift_frames: f64,
        max_endpoint_energy_delta_db: f64,
        max_added_silence_frames: usize,
        max_peak_growth_db: f64,
    ) -> Self {
        Self {
            max_output_length_drift_frames,
            max_endpoint_energy_delta_db,
            max_added_silence_frames,
            max_peak_growth_db,
        }
    }
}

/// Absolute full-render integrity assessment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StretchRenderIntegrityAssessment {
    /// Whether all configured limits passed.
    pub passed: bool,
    /// Whether output length passed.
    pub output_length_passed: bool,
    /// Whether endpoint energy passed.
    pub endpoint_energy_passed: bool,
    /// Whether added silence passed.
    pub added_silence_passed: bool,
    /// Whether peak growth passed.
    pub peak_growth_passed: bool,
}

/// Measure full-render boundary, silence, peak, and length integrity.
///
/// `endpoint_source_frames` selects the source-domain head/tail span. The
/// output span is scaled by `ratio`. `silence_threshold` is an absolute sample
/// amplitude used only for longest-run comparison.
pub fn measure_stretch_render_integrity(
    input: &[Sample],
    output: &[Sample],
    ratio: f64,
    endpoint_source_frames: usize,
    silence_threshold: Sample,
) -> StretchRenderIntegrityMeasurement {
    if input.is_empty() || output.is_empty() || !ratio.is_finite() || ratio <= 0.0 {
        return invalid_measurement(ratio, input.len(), output.len());
    }

    let input_endpoint_frames = endpoint_source_frames.max(1).min(input.len());
    let output_endpoint_frames = ((input_endpoint_frames as f64 * ratio).round() as usize)
        .max(1)
        .min(output.len());
    let input_head_rms = rms(&input[..input_endpoint_frames]);
    let input_tail_rms = rms(&input[input.len() - input_endpoint_frames..]);
    let output_head_rms = rms(&output[..output_endpoint_frames]);
    let output_tail_rms = rms(&output[output.len() - output_endpoint_frames..]);
    let head_delta_db = amplitude_delta_db(output_head_rms, input_head_rms);
    let tail_delta_db = amplitude_delta_db(output_tail_rms, input_tail_rms);
    let input_peak = peak(input);
    let output_peak = peak(output);
    let longest_input_silence_frames = longest_silence_run(input, silence_threshold);
    let longest_output_silence_frames = longest_silence_run(output, silence_threshold);
    let expected_output_silence = (longest_input_silence_frames as f64 * ratio).round() as usize;

    StretchRenderIntegrityMeasurement {
        ratio,
        input_frames: input.len(),
        output_frames: output.len(),
        output_length_drift_frames: (output.len() as f64 - input.len() as f64 * ratio).abs(),
        endpoint_energy_delta_db: head_delta_db.abs().max(tail_delta_db.abs()),
        added_silence_frames: longest_output_silence_frames.saturating_sub(expected_output_silence),
        peak_growth_db: amplitude_delta_db(output_peak, input_peak).max(0.0),
        input_peak,
        output_peak,
        input_head_rms,
        output_head_rms,
        input_tail_rms,
        output_tail_rms,
        longest_input_silence_frames,
        longest_output_silence_frames,
    }
}

/// Assess full-render measurements against absolute limits.
pub fn assess_stretch_render_integrity(
    measurement: StretchRenderIntegrityMeasurement,
    limits: StretchRenderIntegrityLimits,
) -> StretchRenderIntegrityAssessment {
    let output_length_passed = measurement.output_length_drift_frames.is_finite()
        && measurement.output_length_drift_frames <= limits.max_output_length_drift_frames;
    let endpoint_energy_passed = measurement.endpoint_energy_delta_db.is_finite()
        && measurement.endpoint_energy_delta_db <= limits.max_endpoint_energy_delta_db;
    let added_silence_passed = measurement.added_silence_frames <= limits.max_added_silence_frames;
    let peak_growth_passed = measurement.peak_growth_db.is_finite()
        && measurement.peak_growth_db <= limits.max_peak_growth_db;

    StretchRenderIntegrityAssessment {
        passed: output_length_passed
            && endpoint_energy_passed
            && added_silence_passed
            && peak_growth_passed,
        output_length_passed,
        endpoint_energy_passed,
        added_silence_passed,
        peak_growth_passed,
    }
}

fn invalid_measurement(
    ratio: f64,
    input_frames: usize,
    output_frames: usize,
) -> StretchRenderIntegrityMeasurement {
    StretchRenderIntegrityMeasurement {
        ratio,
        input_frames,
        output_frames,
        output_length_drift_frames: f64::NAN,
        endpoint_energy_delta_db: f64::NAN,
        added_silence_frames: usize::MAX,
        peak_growth_db: f64::NAN,
        input_peak: f64::NAN,
        output_peak: f64::NAN,
        input_head_rms: f64::NAN,
        output_head_rms: f64::NAN,
        input_tail_rms: f64::NAN,
        output_tail_rms: f64::NAN,
        longest_input_silence_frames: usize::MAX,
        longest_output_silence_frames: usize::MAX,
    }
}

fn rms(samples: &[Sample]) -> f64 {
    (samples
        .iter()
        .map(|sample| (*sample as f64) * (*sample as f64))
        .sum::<f64>()
        / samples.len().max(1) as f64)
        .sqrt()
}

fn peak(samples: &[Sample]) -> f64 {
    samples
        .iter()
        .map(|sample| sample.abs() as f64)
        .fold(0.0, f64::max)
}

fn amplitude_delta_db(value: f64, reference: f64) -> f64 {
    match (value > 0.0, reference > 0.0) {
        (true, true) => 20.0 * (value / reference).log10(),
        (false, false) => 0.0,
        (false, true) => SILENCE_FLOOR_DB,
        (true, false) => -SILENCE_FLOOR_DB,
    }
}

fn longest_silence_run(samples: &[Sample], threshold: Sample) -> usize {
    let threshold = threshold.abs();
    let mut longest = 0usize;
    let mut current = 0usize;
    for sample in samples {
        if sample.abs() <= threshold {
            current = current.saturating_add(1);
            longest = longest.max(current);
        } else {
            current = 0;
        }
    }
    longest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_integrity_identity_passes_zero_tolerance() {
        let input = vec![0.25; 4_096];
        let measurement = measure_stretch_render_integrity(&input, &input, 1.0, 256, 1.0e-6);
        let assessment = assess_stretch_render_integrity(
            measurement,
            StretchRenderIntegrityLimits::new(0.0, 0.0, 0, 0.0),
        );

        assert!(assessment.passed);
        assert_eq!(measurement.endpoint_energy_delta_db, 0.0);
        assert_eq!(measurement.added_silence_frames, 0);
        assert_eq!(measurement.peak_growth_db, 0.0);
    }

    #[test]
    fn render_integrity_rejects_zero_filled_tail() {
        let input = vec![0.25; 4_096];
        let mut output = vec![0.25; 8_192];
        output[7_168..].fill(0.0);
        let measurement = measure_stretch_render_integrity(&input, &output, 2.0, 512, 1.0e-6);
        let assessment = assess_stretch_render_integrity(
            measurement,
            StretchRenderIntegrityLimits::new(0.0, 6.0, 0, 1.0),
        );

        assert!(!assessment.passed);
        assert!(!assessment.endpoint_energy_passed);
        assert!(!assessment.added_silence_passed);
        assert_eq!(measurement.added_silence_frames, 1_024);
    }

    #[test]
    fn render_integrity_reports_positive_peak_growth() {
        let input = vec![0.25; 1_024];
        let output = vec![0.5; 1_024];
        let measurement = measure_stretch_render_integrity(&input, &output, 1.0, 128, 1.0e-6);

        assert!((measurement.peak_growth_db - 6.020_599_913).abs() < 1.0e-6);
    }

    #[test]
    fn render_integrity_invalid_input_fails_assessment() {
        let measurement = measure_stretch_render_integrity(&[], &[], 1.0, 128, 1.0e-6);
        let assessment = assess_stretch_render_integrity(
            measurement,
            StretchRenderIntegrityLimits::new(0.0, 0.0, 0, 0.0),
        );

        assert!(!assessment.passed);
    }
}
