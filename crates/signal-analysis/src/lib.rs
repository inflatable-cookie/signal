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

pub mod corpus;
pub mod harness;
pub mod input;

use signal_primitives::AudioBuffer;

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

// Re-export corpus types.
pub use corpus::{
    AcceptanceSeverity, AcceptanceStatus, AcceptanceThreshold, AnalysisCorpusArtifactSize,
    AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisCorpusSource, AnalysisMetricValue,
    RegressionDriftLimit,
};

// Re-export input preparation types and functions.
pub use input::{
    prepare_audio_analysis, prepare_mono_analysis, AnalysisChannelPolicy, AnalysisInputConfig,
    PreparedAnalysisBuffer,
};

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

/// Acceptance result for one corpus case.
#[derive(Clone, Debug, PartialEq)]
pub struct AcceptanceCaseReport {
    pub case_id: String,
    pub status: AcceptanceStatus,
    pub elapsed_ms: f32,
    pub metrics: Vec<harness::AcceptanceMetricAssessment>,
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

/// Re-export harness functions at crate root for backward compatibility.
pub use harness::compare_audio_analyzers;
pub use harness::run_audio_acceptance_harness;

#[cfg(test)]
mod tests {
    use super::*;
    use signal_primitives::{ChannelLayout, SampleRate};

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
