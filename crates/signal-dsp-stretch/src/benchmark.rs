use signal_primitives::Sample;

/// Corpus family required by the Signal-native stretch benchmark program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCorpusFamily {
    /// Transient-heavy drum and percussion material.
    DrumsPercussion,
    /// Bass material with sustained notes and plucked attacks.
    Bass,
    /// Spoken or sung vocal material.
    Vocals,
    /// Sustained harmonic pads, piano tails, and reverberant material.
    PadsSustains,
    /// Full stereo mixes with dense cross-band interaction.
    FullMix,
    /// Material rendered against tempo ramps and dynamic ratio curves.
    TempoRamp,
    /// Looping material with boundary and warp-marker seam pressure.
    LoopSeam,
    /// Material that exercises wide stretch ratios and degradation policy.
    ExtremeRatio,
}

/// Source/provenance class for a stretch benchmark case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchCorpusSource {
    /// Generated in the test harness.
    Synthetic,
    /// Repository-local fixture with checked-in license/provenance.
    LocalFixture,
    /// External benchmark output used only as comparison evidence.
    ExternalBenchmark,
    /// Operator-provided licensed listening material.
    LicensedListening,
}

/// One required stretch benchmark case blueprint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchCorpusCase {
    /// Stable case identifier.
    pub case_id: &'static str,
    /// Required material family.
    pub family: StretchCorpusFamily,
    /// Provenance class.
    pub source: StretchCorpusSource,
    /// Output/input duration ratios this case must exercise.
    pub ratios: &'static [f64],
    /// What artifact the case is intended to expose.
    pub intent: &'static str,
}

const STANDARD_RATIOS: &[f64] = &[0.75, 1.25, 1.5];
const RAMP_RATIOS: &[f64] = &[0.75, 1.0, 1.5];
const LOOP_RATIOS: &[f64] = &[0.5, 1.0, 2.0];
const EXTREME_RATIOS: &[f64] = &[0.5, 0.75, 1.5, 2.0];

/// Required benchmark corpus blueprint for Signal-owned stretch promotion.
pub const STRETCH_BENCHMARK_CORPUS: [StretchCorpusCase; 8] = [
    StretchCorpusCase {
        case_id: "stretch:drums_percussion",
        family: StretchCorpusFamily::DrumsPercussion,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "transient preservation, cymbal texture, kick timing",
    },
    StretchCorpusCase {
        case_id: "stretch:bass",
        family: StretchCorpusFamily::Bass,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "low-frequency stability, pluck attack preservation",
    },
    StretchCorpusCase {
        case_id: "stretch:vocals",
        family: StretchCorpusFamily::Vocals,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "consonants, breath noise, vibrato, formant-adjacent artifacts",
    },
    StretchCorpusCase {
        case_id: "stretch:pads_sustains",
        family: StretchCorpusFamily::PadsSustains,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "phasiness, beating, reverb-tail stability",
    },
    StretchCorpusCase {
        case_id: "stretch:full_mix",
        family: StretchCorpusFamily::FullMix,
        source: StretchCorpusSource::LicensedListening,
        ratios: STANDARD_RATIOS,
        intent: "cross-band coherence and stereo image stability",
    },
    StretchCorpusCase {
        case_id: "stretch:tempo_ramp",
        family: StretchCorpusFamily::TempoRamp,
        source: StretchCorpusSource::Synthetic,
        ratios: RAMP_RATIOS,
        intent: "dynamic-ratio drift and automation alignment",
    },
    StretchCorpusCase {
        case_id: "stretch:loop_seam",
        family: StretchCorpusFamily::LoopSeam,
        source: StretchCorpusSource::Synthetic,
        ratios: LOOP_RATIOS,
        intent: "loop-boundary click and warp-marker seam behavior",
    },
    StretchCorpusCase {
        case_id: "stretch:extreme_ratio",
        family: StretchCorpusFamily::ExtremeRatio,
        source: StretchCorpusSource::Synthetic,
        ratios: EXTREME_RATIOS,
        intent: "wide-ratio quality and out-of-support degradation policy",
    },
];

/// Inline synthetic audio generated for stretch benchmark bootstrap cases.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchSyntheticAudio {
    /// Sample rate in hertz.
    pub sample_rate_hz: u32,
    /// Interleaved channel count.
    pub channels: u16,
    /// Interleaved sample frames.
    pub samples: Vec<Sample>,
}

impl StretchSyntheticAudio {
    /// Number of sample frames.
    pub fn frame_count(&self) -> usize {
        self.samples.len() / self.channels.max(1) as usize
    }
}

/// Generate the synthetic benchmark audio for a corpus family.
pub fn generate_synthetic_stretch_audio(
    family: StretchCorpusFamily,
) -> Option<StretchSyntheticAudio> {
    match family {
        StretchCorpusFamily::TempoRamp => Some(synthetic_tempo_ramp()),
        StretchCorpusFamily::LoopSeam => Some(synthetic_loop_seam()),
        StretchCorpusFamily::ExtremeRatio => Some(synthetic_extreme_ratio()),
        _ => None,
    }
}

/// Generate all inline synthetic benchmark cases declared in the corpus
/// blueprint.
pub fn synthetic_stretch_corpus_cases() -> Vec<(StretchCorpusCase, StretchSyntheticAudio)> {
    STRETCH_BENCHMARK_CORPUS
        .iter()
        .filter_map(|case| {
            if case.source == StretchCorpusSource::Synthetic {
                generate_synthetic_stretch_audio(case.family).map(|audio| (*case, audio))
            } else {
                None
            }
        })
        .collect()
}

/// Objective metric family used by the stretch benchmark harness.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchMetric {
    /// Absolute output-length or segment-boundary drift in samples.
    TimingDriftSamples,
    /// Attack widening around detected transients, in frames.
    TransientSmearFrames,
    /// Inter-bin or peak-neighborhood phase-coherence delta.
    VerticalCoherenceDelta,
    /// Mid/side or channel-correlation image delta.
    StereoImageDelta,
    /// Highest click or discontinuity at a loop boundary, in dBFS.
    LoopBoundaryClickDbfs,
    /// CPU time relative to rendered audio duration.
    CpuRealtimeFactor,
    /// Reported algorithmic latency, in frames.
    LatencyFrames,
    /// Peak memory used by the render, in bytes.
    PeakMemoryBytes,
}

/// Severity for one stretch metric limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchAcceptanceSeverity {
    /// Limit breach should be visible but not fail the run.
    Warn,
    /// Limit breach fails the run.
    Fail,
}

/// Aggregated result for one metric or report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StretchAcceptanceStatus {
    /// All limits passed.
    Pass,
    /// At least one warning limit breached and no failure limit breached.
    Warn,
    /// At least one failure limit breached.
    Fail,
}

/// One measured stretch benchmark metric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchMetricValue {
    /// Metric identity.
    pub metric: StretchMetric,
    /// Metric value. Limits interpret the value as "lower is better".
    pub value: f64,
}

impl StretchMetricValue {
    /// Construct a metric value.
    pub fn new(metric: StretchMetric, value: f64) -> Self {
        Self { metric, value }
    }
}

/// Upper-bound limit for a stretch benchmark metric.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchMetricLimit {
    /// Metric identity.
    pub metric: StretchMetric,
    /// Inclusive upper bound.
    pub max: f64,
    /// Severity to report when the value exceeds `max` or is not finite.
    pub severity: StretchAcceptanceSeverity,
}

impl StretchMetricLimit {
    /// Construct a metric upper-bound limit.
    pub fn max(metric: StretchMetric, max: f64, severity: StretchAcceptanceSeverity) -> Self {
        Self {
            metric,
            max,
            severity,
        }
    }
}

/// Assessment for one metric limit.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StretchMetricAssessment {
    /// Metric identity.
    pub metric: StretchMetric,
    /// Measured value, or `NaN` when the metric was missing.
    pub value: f64,
    /// Inclusive upper bound.
    pub max: f64,
    /// Result for this metric.
    pub status: StretchAcceptanceStatus,
}

/// Assessment report for a stretch benchmark case or tier.
#[derive(Clone, Debug, PartialEq)]
pub struct StretchAcceptanceReport {
    /// Aggregated worst status.
    pub status: StretchAcceptanceStatus,
    /// Metric assessments in limit order.
    pub metrics: Vec<StretchMetricAssessment>,
}

/// Evaluate measured stretch metrics against upper-bound limits.
pub fn assess_stretch_metrics(
    measurements: &[StretchMetricValue],
    limits: &[StretchMetricLimit],
) -> StretchAcceptanceReport {
    let mut status = StretchAcceptanceStatus::Pass;
    let mut metrics = Vec::with_capacity(limits.len());
    for limit in limits {
        let value = measurements
            .iter()
            .find(|measurement| measurement.metric == limit.metric)
            .map(|measurement| measurement.value)
            .unwrap_or(f64::NAN);
        let metric_status = if value.is_finite() && value <= limit.max {
            StretchAcceptanceStatus::Pass
        } else {
            severity_to_stretch_status(limit.severity)
        };
        status = combine_stretch_status(status, metric_status);
        metrics.push(StretchMetricAssessment {
            metric: limit.metric,
            value,
            max: limit.max,
            status: metric_status,
        });
    }

    StretchAcceptanceReport { status, metrics }
}

/// Absolute output-length drift from the fixed-ratio length contract.
pub fn output_length_drift_samples(input_frames: usize, output_frames: usize, ratio: f64) -> f64 {
    if !ratio.is_finite() || ratio <= 0.0 {
        return f64::NAN;
    }
    let expected = (input_frames as f64 * ratio).round();
    (output_frames as f64 - expected).abs()
}

/// Deterministic line-oriented report for one stretch acceptance result.
pub fn format_stretch_acceptance_report(case_id: &str, report: &StretchAcceptanceReport) -> String {
    let mut lines = Vec::with_capacity(report.metrics.len() + 1);
    lines.push(format!("case={case_id} status={:?}", report.status));
    for metric in &report.metrics {
        lines.push(format!(
            "metric={:?} value={:.6} max={:.6} status={:?}",
            metric.metric, metric.value, metric.max, metric.status
        ));
    }
    lines.join("\n")
}

fn synthetic_tempo_ramp() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize * 2;
    let mut samples = Vec::with_capacity(FRAMES * 2);
    for frame in 0..FRAMES {
        let progress = frame as f32 / FRAMES as f32;
        let frequency = 220.0 + 220.0 * progress;
        let carrier = (std::f32::consts::TAU * frequency * frame as f32 / SAMPLE_RATE as f32).sin();
        let pulse = if frame % 12_000 < 96 { 0.7 } else { 0.0 };
        let sample = (carrier * 0.25 + pulse) * (1.0 - 0.25 * progress);
        samples.push(sample);
        samples.push(sample);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 2,
        samples,
    }
}

fn synthetic_loop_seam() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize;
    let mut samples = Vec::with_capacity(FRAMES * 2);
    for frame in 0..FRAMES {
        let phase = frame as f32 / FRAMES as f32;
        let body = (std::f32::consts::TAU * 110.0 * frame as f32 / SAMPLE_RATE as f32).sin() * 0.2;
        let boundary_probe = if !(128..FRAMES - 128).contains(&frame) {
            0.8 * (1.0 - frame.min(FRAMES - 1 - frame) as f32 / 128.0)
        } else {
            0.0
        };
        let left = body + boundary_probe;
        let right = body * (0.95 + 0.05 * phase) + boundary_probe;
        samples.push(left);
        samples.push(right);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 2,
        samples,
    }
}

fn synthetic_extreme_ratio() -> StretchSyntheticAudio {
    const SAMPLE_RATE: u32 = 48_000;
    const FRAMES: usize = SAMPLE_RATE as usize * 2;
    let mut samples = Vec::with_capacity(FRAMES);
    for frame in 0..FRAMES {
        let tonal =
            (std::f32::consts::TAU * 330.0 * frame as f32 / SAMPLE_RATE as f32).sin() * 0.25;
        let transient = if frame % 8_000 < 64 {
            0.9 * (1.0 - (frame % 8_000) as f32 / 64.0)
        } else {
            0.0
        };
        samples.push(tonal + transient);
    }
    StretchSyntheticAudio {
        sample_rate_hz: SAMPLE_RATE,
        channels: 1,
        samples,
    }
}

fn severity_to_stretch_status(severity: StretchAcceptanceSeverity) -> StretchAcceptanceStatus {
    match severity {
        StretchAcceptanceSeverity::Warn => StretchAcceptanceStatus::Warn,
        StretchAcceptanceSeverity::Fail => StretchAcceptanceStatus::Fail,
    }
}

fn combine_stretch_status(
    left: StretchAcceptanceStatus,
    right: StretchAcceptanceStatus,
) -> StretchAcceptanceStatus {
    match (left, right) {
        (StretchAcceptanceStatus::Fail, _) | (_, StretchAcceptanceStatus::Fail) => {
            StretchAcceptanceStatus::Fail
        }
        (StretchAcceptanceStatus::Warn, _) | (_, StretchAcceptanceStatus::Warn) => {
            StretchAcceptanceStatus::Warn
        }
        _ => StretchAcceptanceStatus::Pass,
    }
}
