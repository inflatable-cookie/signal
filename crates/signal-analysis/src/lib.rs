//! Shared analysis traits, result types, and confidence models for Signal.
//!
//! Analysis crates in the workspace implement [`AnalysisStage`] and return
//! confidence-scored result types through this shared contract layer.
//!
//! ```no_run
//! use signal_analysis::{AnalysisInputConfig, AnalysisMode, AnalysisStage, Confidence};
//! use signal_primitives::{AudioBuffer, ChannelLayout, FrameCount, SampleRate};
//!
//! struct EmptyStage;
//!
//! impl AnalysisStage<Confidence> for EmptyStage {
//!     fn mode(&self) -> AnalysisMode {
//!         AnalysisMode::Offline
//!     }
//!
//!     fn analyze(&mut self, _audio: &AudioBuffer) -> Confidence {
//!         Confidence::new(0.25)
//!     }
//! }
//!
//! let audio = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Mono, FrameCount(128));
//! let mut stage = EmptyStage;
//! assert_eq!(stage.analyze(&audio), Confidence::new(0.25));
//!
//! let prepared = signal_analysis::prepare_audio_analysis(&audio, AnalysisInputConfig::default());
//! assert_eq!(prepared.sample_rate, SampleRate(48_000));
//! ```

use signal_dsp_resample::{resample_mono, ResampleConfig, ResampleQuality};
use signal_primitives::AudioBuffer;
use signal_primitives::{Sample, SampleRate, Seconds};
use std::time::Instant;

/// Execution mode for an analysis stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisMode {
    Offline,
    Streaming,
}

/// Confidence score normalized to the inclusive range `0.0..=1.0`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Confidence(pub f32);

impl Confidence {
    /// Construct a confidence value, clamping it into `0.0..=1.0`.
    pub fn new(value: f32) -> Self {
        Self(value.clamp(0.0, 1.0))
    }
}

/// Shared trait for analysis stages that consume an [`AudioBuffer`].
pub trait AnalysisStage<Output> {
    /// Report whether the stage is intended for offline or streaming use.
    fn mode(&self) -> AnalysisMode;

    /// Analyze an audio buffer and return the stage-specific output.
    fn analyze(&mut self, audio: &AudioBuffer) -> Output;
}

/// Shared mono reduction policy for analyzer input preparation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisChannelPolicy {
    /// Average all input channels into a mono stream.
    MixToMono,
    /// Use only the first channel of the input stream.
    FirstChannel,
}

/// Shared preprocessing configuration for offline or chunk-owned analyzers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnalysisInputConfig {
    pub channel_policy: AnalysisChannelPolicy,
    pub max_duration: Option<Seconds>,
    pub target_sample_rate: Option<SampleRate>,
    pub resample_quality: ResampleQuality,
}

impl Default for AnalysisInputConfig {
    fn default() -> Self {
        Self {
            channel_policy: AnalysisChannelPolicy::MixToMono,
            max_duration: None,
            target_sample_rate: None,
            resample_quality: ResampleQuality::Linear,
        }
    }
}

/// Prepared mono analysis input after optional duration limiting and resampling.
#[derive(Clone, Debug, PartialEq)]
pub struct PreparedAnalysisBuffer {
    pub sample_rate: SampleRate,
    pub samples: Vec<Sample>,
}

/// Shared corpus family tags for regression-sensitive analysis fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisCorpusFamily {
    Tonal,
    Noise,
    Pulse,
    Sustained,
    Loudness,
    Semantic,
    Silence,
    RatePolicy,
}

/// Origin type for a shared corpus case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisCorpusSource {
    Synthetic,
    ExternalReference,
    LicensedCorpus,
}

/// Artifact-size class for a corpus case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisCorpusArtifactSize {
    InlineSynthetic,
    SmallLocal,
    LargeExternal,
}

/// Severity for acceptance and regression thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptanceSeverity {
    Warn,
    Fail,
}

/// Aggregated status for one metric, case, or report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptanceStatus {
    Pass,
    Warn,
    Fail,
}

/// Shared metadata for an analysis corpus case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisCorpusCaseMetadata {
    pub case_id: String,
    pub family: AnalysisCorpusFamily,
    pub source: AnalysisCorpusSource,
    pub artifact_size: AnalysisCorpusArtifactSize,
    pub description: String,
}

impl AnalysisCorpusCaseMetadata {
    pub fn synthetic(
        case_id: impl Into<String>,
        family: AnalysisCorpusFamily,
        description: impl Into<String>,
    ) -> Self {
        Self {
            case_id: case_id.into(),
            family,
            source: AnalysisCorpusSource::Synthetic,
            artifact_size: AnalysisCorpusArtifactSize::InlineSynthetic,
            description: description.into(),
        }
    }
}

/// One named analysis metric value.
#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisMetricValue {
    pub name: String,
    pub value: f32,
}

impl AnalysisMetricValue {
    pub fn new(name: impl Into<String>, value: f32) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}

/// Inclusive metric threshold for one corpus case.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceThreshold {
    pub metric: String,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub severity: AcceptanceSeverity,
}

impl AcceptanceThreshold {
    pub fn range(
        metric: impl Into<String>,
        min: Option<f32>,
        max: Option<f32>,
        severity: AcceptanceSeverity,
    ) -> Self {
        Self {
            metric: metric.into(),
            min,
            max,
            severity,
        }
    }
}

/// Absolute-delta drift limit for baseline-versus-candidate regression checks.
#[derive(Clone, Debug, PartialEq)]
pub struct RegressionDriftLimit {
    pub metric: String,
    pub max_abs_delta: f32,
    pub severity: AcceptanceSeverity,
}

impl RegressionDriftLimit {
    pub fn new(
        metric: impl Into<String>,
        max_abs_delta: f32,
        severity: AcceptanceSeverity,
    ) -> Self {
        Self {
            metric: metric.into(),
            max_abs_delta,
            severity,
        }
    }
}

/// One shared corpus case with inline expectations and drift limits.
#[derive(Clone, Debug, PartialEq)]
pub struct AnalysisCorpusCase {
    pub metadata: AnalysisCorpusCaseMetadata,
    pub audio: AudioBuffer,
    pub acceptance_thresholds: Vec<AcceptanceThreshold>,
    pub regression_limits: Vec<RegressionDriftLimit>,
}

impl AnalysisCorpusCase {
    pub fn new(metadata: AnalysisCorpusCaseMetadata, audio: AudioBuffer) -> Self {
        Self {
            metadata,
            audio,
            acceptance_thresholds: Vec::new(),
            regression_limits: Vec::new(),
        }
    }

    pub fn with_acceptance_thresholds(mut self, thresholds: Vec<AcceptanceThreshold>) -> Self {
        self.acceptance_thresholds = thresholds;
        self
    }

    pub fn with_regression_limits(mut self, limits: Vec<RegressionDriftLimit>) -> Self {
        self.regression_limits = limits;
        self
    }
}

/// Evaluation of one metric against one acceptance threshold.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceMetricAssessment {
    pub metric: String,
    pub value: f32,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub status: AcceptanceStatus,
}

/// Acceptance result for one corpus case.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceCaseReport {
    pub case_id: String,
    pub status: AcceptanceStatus,
    pub elapsed_ms: f32,
    pub metrics: Vec<AcceptanceMetricAssessment>,
}

/// Acceptance result over a shared corpus slice.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceHarnessReport {
    pub status: AcceptanceStatus,
    pub cases: Vec<AcceptanceCaseReport>,
}

/// Delta for one metric between baseline and candidate analyzers.
#[derive(Clone, Debug, PartialEq)]
pub struct RegressionMetricDelta {
    pub metric: String,
    pub baseline: f32,
    pub candidate: f32,
    pub abs_delta: f32,
    pub max_abs_delta: f32,
    pub status: AcceptanceStatus,
}

/// Regression result for one corpus case.
#[derive(Clone, Debug, PartialEq)]
pub struct RegressionCaseReport {
    pub case_id: String,
    pub status: AcceptanceStatus,
    pub baseline_elapsed_ms: f32,
    pub candidate_elapsed_ms: f32,
    pub metrics: Vec<RegressionMetricDelta>,
}

/// Regression comparison report across a shared corpus slice.
#[derive(Clone, Debug, PartialEq)]
pub struct RegressionHarnessReport {
    pub status: AcceptanceStatus,
    pub cases: Vec<RegressionCaseReport>,
}

/// Evaluate analyzer outputs against per-case metric thresholds.
pub fn run_audio_acceptance_harness<Output, Analyze, Measure>(
    cases: &[AnalysisCorpusCase],
    mut analyze: Analyze,
    mut measure: Measure,
) -> AcceptanceHarnessReport
where
    Analyze: FnMut(&AudioBuffer) -> Output,
    Measure: FnMut(&Output) -> Vec<AnalysisMetricValue>,
{
    let mut overall_status = AcceptanceStatus::Pass;
    let mut reports = Vec::with_capacity(cases.len());

    for case in cases {
        let started = Instant::now();
        let output = analyze(&case.audio);
        let elapsed_ms = started.elapsed().as_secs_f32() * 1_000.0;
        let measured = measure(&output);
        let mut case_status = AcceptanceStatus::Pass;
        let mut metric_reports = Vec::with_capacity(case.acceptance_thresholds.len());

        for threshold in &case.acceptance_thresholds {
            let value = measured
                .iter()
                .find(|metric| metric.name == threshold.metric)
                .map(|metric| metric.value)
                .unwrap_or(f32::NAN);
            let passes_min = threshold.min.map(|min| value >= min).unwrap_or(true);
            let passes_max = threshold.max.map(|max| value <= max).unwrap_or(true);
            let status = if value.is_nan() || !(passes_min && passes_max) {
                severity_to_status(threshold.severity)
            } else {
                AcceptanceStatus::Pass
            };

            case_status = combine_status(case_status, status);
            metric_reports.push(AcceptanceMetricAssessment {
                metric: threshold.metric.clone(),
                value,
                min: threshold.min,
                max: threshold.max,
                status,
            });
        }

        overall_status = combine_status(overall_status, case_status);
        reports.push(AcceptanceCaseReport {
            case_id: case.metadata.case_id.clone(),
            status: case_status,
            elapsed_ms,
            metrics: metric_reports,
        });
    }

    AcceptanceHarnessReport {
        status: overall_status,
        cases: reports,
    }
}

/// Compare baseline and candidate analyzer outputs against per-case drift limits.
pub fn compare_audio_analyzers<
    BaselineOutput,
    CandidateOutput,
    AnalyzeBaseline,
    AnalyzeCandidate,
    MeasureBaseline,
    MeasureCandidate,
>(
    cases: &[AnalysisCorpusCase],
    mut baseline_analyze: AnalyzeBaseline,
    mut candidate_analyze: AnalyzeCandidate,
    mut baseline_measure: MeasureBaseline,
    mut candidate_measure: MeasureCandidate,
) -> RegressionHarnessReport
where
    AnalyzeBaseline: FnMut(&AudioBuffer) -> BaselineOutput,
    AnalyzeCandidate: FnMut(&AudioBuffer) -> CandidateOutput,
    MeasureBaseline: FnMut(&BaselineOutput) -> Vec<AnalysisMetricValue>,
    MeasureCandidate: FnMut(&CandidateOutput) -> Vec<AnalysisMetricValue>,
{
    let mut overall_status = AcceptanceStatus::Pass;
    let mut reports = Vec::with_capacity(cases.len());

    for case in cases {
        let baseline_started = Instant::now();
        let baseline_output = baseline_analyze(&case.audio);
        let baseline_elapsed_ms = baseline_started.elapsed().as_secs_f32() * 1_000.0;
        let candidate_started = Instant::now();
        let candidate_output = candidate_analyze(&case.audio);
        let candidate_elapsed_ms = candidate_started.elapsed().as_secs_f32() * 1_000.0;

        let baseline_metrics = baseline_measure(&baseline_output);
        let candidate_metrics = candidate_measure(&candidate_output);
        let mut case_status = AcceptanceStatus::Pass;
        let mut metric_reports = Vec::with_capacity(case.regression_limits.len());

        for limit in &case.regression_limits {
            let baseline = baseline_metrics
                .iter()
                .find(|metric| metric.name == limit.metric)
                .map(|metric| metric.value)
                .unwrap_or(f32::NAN);
            let candidate = candidate_metrics
                .iter()
                .find(|metric| metric.name == limit.metric)
                .map(|metric| metric.value)
                .unwrap_or(f32::NAN);
            let abs_delta = if baseline.is_nan() || candidate.is_nan() {
                f32::NAN
            } else {
                (candidate - baseline).abs()
            };
            let status = if abs_delta.is_nan() || abs_delta > limit.max_abs_delta {
                severity_to_status(limit.severity)
            } else {
                AcceptanceStatus::Pass
            };

            case_status = combine_status(case_status, status);
            metric_reports.push(RegressionMetricDelta {
                metric: limit.metric.clone(),
                baseline,
                candidate,
                abs_delta,
                max_abs_delta: limit.max_abs_delta,
                status,
            });
        }

        overall_status = combine_status(overall_status, case_status);
        reports.push(RegressionCaseReport {
            case_id: case.metadata.case_id.clone(),
            status: case_status,
            baseline_elapsed_ms,
            candidate_elapsed_ms,
            metrics: metric_reports,
        });
    }

    RegressionHarnessReport {
        status: overall_status,
        cases: reports,
    }
}

/// Prepare an [`AudioBuffer`] for mono analyzer consumption.
pub fn prepare_audio_analysis(
    audio: &AudioBuffer,
    config: AnalysisInputConfig,
) -> PreparedAnalysisBuffer {
    let mono_samples = match config.channel_policy {
        AnalysisChannelPolicy::MixToMono => audio.to_mono(),
        AnalysisChannelPolicy::FirstChannel => first_channel_samples(audio),
    };

    prepare_mono_analysis(audio.sample_rate(), &mono_samples, config)
}

/// Prepare a mono slice for analyzer consumption.
pub fn prepare_mono_analysis(
    sample_rate: SampleRate,
    mono_samples: &[Sample],
    config: AnalysisInputConfig,
) -> PreparedAnalysisBuffer {
    let mut samples = mono_samples.to_vec();

    if let Some(max_duration) = config.max_duration {
        let max_frames = sample_rate.seconds_to_frames(max_duration).0;
        if max_frames > 0 && samples.len() > max_frames {
            let start = (samples.len() - max_frames) / 2;
            samples = samples[start..start + max_frames].to_vec();
        }
    }

    let output_rate = config.target_sample_rate.unwrap_or(sample_rate);
    if output_rate != sample_rate && !samples.is_empty() {
        samples = resample_mono(
            ResampleConfig::new(sample_rate, output_rate, config.resample_quality),
            &samples,
        );
    }

    PreparedAnalysisBuffer {
        sample_rate: output_rate,
        samples,
    }
}

fn first_channel_samples(audio: &AudioBuffer) -> Vec<Sample> {
    let channels = audio.channel_count().0;
    if channels == 0 || audio.is_empty() {
        return Vec::new();
    }

    if channels == 1 {
        return audio.samples().to_vec();
    }

    audio
        .samples()
        .chunks_exact(channels)
        .map(|frame| frame[0])
        .collect()
}

fn severity_to_status(severity: AcceptanceSeverity) -> AcceptanceStatus {
    match severity {
        AcceptanceSeverity::Warn => AcceptanceStatus::Warn,
        AcceptanceSeverity::Fail => AcceptanceStatus::Fail,
    }
}

fn combine_status(left: AcceptanceStatus, right: AcceptanceStatus) -> AcceptanceStatus {
    match (left, right) {
        (AcceptanceStatus::Fail, _) | (_, AcceptanceStatus::Fail) => AcceptanceStatus::Fail,
        (AcceptanceStatus::Warn, _) | (_, AcceptanceStatus::Warn) => AcceptanceStatus::Warn,
        _ => AcceptanceStatus::Pass,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use signal_primitives::{ChannelLayout, FrameCount};

    #[test]
    fn prepare_audio_analysis_mixes_channels_by_default() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.25, 0.75],
        );

        let prepared = prepare_audio_analysis(&audio, AnalysisInputConfig::default());

        assert_eq!(prepared.sample_rate, SampleRate(48_000));
        assert_eq!(prepared.samples, vec![0.0, 0.5]);
    }

    #[test]
    fn prepare_audio_analysis_can_take_first_channel() {
        let audio = AudioBuffer::from_interleaved(
            SampleRate(48_000),
            ChannelLayout::Stereo,
            vec![1.0, -1.0, 0.25, 0.75],
        );
        let config = AnalysisInputConfig {
            channel_policy: AnalysisChannelPolicy::FirstChannel,
            ..AnalysisInputConfig::default()
        };

        let prepared = prepare_audio_analysis(&audio, config);

        assert_eq!(prepared.samples, vec![1.0, 0.25]);
    }

    #[test]
    fn prepare_mono_analysis_center_trims_to_duration() {
        let samples: Vec<f32> = (0..10).map(|index| index as f32).collect();
        let prepared = prepare_mono_analysis(
            SampleRate(10),
            &samples,
            AnalysisInputConfig {
                max_duration: Some(Seconds(0.4)),
                ..AnalysisInputConfig::default()
            },
        );

        assert_eq!(prepared.samples, vec![3.0, 4.0, 5.0, 6.0]);
    }

    #[test]
    fn prepare_mono_analysis_resamples_to_target_rate() {
        let prepared = prepare_mono_analysis(
            SampleRate(8),
            &[0.0, 1.0, 0.0, -1.0],
            AnalysisInputConfig {
                target_sample_rate: Some(SampleRate(4)),
                ..AnalysisInputConfig::default()
            },
        );

        assert_eq!(prepared.sample_rate, SampleRate(4));
        assert_eq!(prepared.samples.len(), 2);
    }

    #[test]
    fn prepare_audio_analysis_handles_empty_buffers() {
        let audio = AudioBuffer::new(SampleRate(48_000), ChannelLayout::Mono, FrameCount(0));
        let prepared = prepare_audio_analysis(&audio, AnalysisInputConfig::default());

        assert!(prepared.samples.is_empty());
    }

    fn rms_metric(audio: &AudioBuffer) -> Vec<AnalysisMetricValue> {
        let samples = audio.samples();
        let rms = if samples.is_empty() {
            0.0
        } else {
            let sum_squares: f32 = samples.iter().map(|sample| sample * sample).sum();
            (sum_squares / samples.len() as f32).sqrt()
        };
        vec![AnalysisMetricValue::new("rms_energy", rms)]
    }

    #[test]
    fn acceptance_harness_passes_in_range_case() {
        let case = AnalysisCorpusCase::new(
            AnalysisCorpusCaseMetadata::synthetic(
                "case:sine",
                AnalysisCorpusFamily::Tonal,
                "Low-amplitude tonal reference",
            ),
            AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.25, -0.25, 0.25, -0.25],
            ),
        )
        .with_acceptance_thresholds(vec![AcceptanceThreshold::range(
            "rms_energy",
            Some(0.24),
            Some(0.26),
            AcceptanceSeverity::Fail,
        )]);

        let report = run_audio_acceptance_harness(&[case], |audio| audio.clone(), rms_metric);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 1);
        assert_eq!(report.cases[0].metrics[0].status, AcceptanceStatus::Pass);
    }

    #[test]
    fn acceptance_harness_reports_failures() {
        let case = AnalysisCorpusCase::new(
            AnalysisCorpusCaseMetadata::synthetic(
                "case:loud",
                AnalysisCorpusFamily::Loudness,
                "Deliberately failing acceptance bound",
            ),
            AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![1.0, -1.0, 1.0, -1.0],
            ),
        )
        .with_acceptance_thresholds(vec![AcceptanceThreshold::range(
            "rms_energy",
            Some(0.1),
            Some(0.5),
            AcceptanceSeverity::Fail,
        )]);

        let report = run_audio_acceptance_harness(&[case], |audio| audio.clone(), rms_metric);

        assert_eq!(report.status, AcceptanceStatus::Fail);
        assert_eq!(report.cases[0].metrics[0].status, AcceptanceStatus::Fail);
    }

    #[test]
    fn regression_harness_reports_drift() {
        let case = AnalysisCorpusCase::new(
            AnalysisCorpusCaseMetadata::synthetic(
                "case:regression",
                AnalysisCorpusFamily::Tonal,
                "Simple regression check",
            ),
            AudioBuffer::from_interleaved(
                SampleRate(48_000),
                ChannelLayout::Mono,
                vec![0.5, -0.5, 0.5, -0.5],
            ),
        )
        .with_regression_limits(vec![RegressionDriftLimit::new(
            "rms_energy",
            0.05,
            AcceptanceSeverity::Fail,
        )]);

        let report = compare_audio_analyzers(
            &[case],
            |audio| audio.clone(),
            |_audio| {
                AudioBuffer::from_interleaved(
                    SampleRate(48_000),
                    ChannelLayout::Mono,
                    vec![1.0, -1.0, 1.0, -1.0],
                )
            },
            rms_metric,
            rms_metric,
        );

        assert_eq!(report.status, AcceptanceStatus::Fail);
        assert!(report.cases[0].metrics[0].abs_delta > 0.05);
    }
}
