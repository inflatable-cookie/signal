//! Corpus and test case types for analysis validation.
//!
//! This module provides types for defining analysis test corpora, including
//! metadata, thresholds, and drift limits for acceptance and regression testing.

/// Shared corpus family tags for regression-sensitive analysis fixtures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisCorpusFamily {
    /// Pitched or harmonic material (sine tones, instruments).
    Tonal,
    /// Broadband or stochastic noise.
    Noise,
    /// Transient-heavy or percussive content.
    Pulse,
    /// Long-held, slowly evolving content.
    Sustained,
    /// Material chosen to exercise loudness measurement paths.
    Loudness,
    /// Content tagged for semantic or high-level feature tests.
    Semantic,
    /// Digital silence or near-silence.
    Silence,
    /// Cases that exercise sample-rate-specific policy paths.
    RatePolicy,
}

/// Origin type for a shared corpus case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisCorpusSource {
    /// Audio generated programmatically within the test suite.
    Synthetic,
    /// Audio sourced from a publicly available external reference.
    ExternalReference,
    /// Audio from a commercially licensed corpus.
    LicensedCorpus,
}

/// Artifact-size class for a corpus case.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnalysisCorpusArtifactSize {
    /// Audio is generated inline — no file I/O required.
    InlineSynthetic,
    /// Small audio file stored locally in the repository.
    SmallLocal,
    /// Large file fetched from an external location at test time.
    LargeExternal,
}

/// Severity for acceptance and regression thresholds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptanceSeverity {
    /// A threshold violation is noteworthy but does not fail the run.
    Warn,
    /// A threshold violation fails the run.
    Fail,
}

/// Aggregated status for one metric, case, or report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AcceptanceStatus {
    /// All checks within the scope passed.
    Pass,
    /// At least one check triggered a warning; none failed.
    Warn,
    /// At least one check failed.
    Fail,
}

/// Shared metadata for an analysis corpus case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AnalysisCorpusCaseMetadata {
    /// Unique string identifier for the case (e.g. `"case:sine_440"`).
    pub case_id: String,
    /// Broad content category used for filtering and reporting.
    pub family: AnalysisCorpusFamily,
    /// Origin of the audio artifact.
    pub source: AnalysisCorpusSource,
    /// Artifact size class, used to decide whether the case can run offline.
    pub artifact_size: AnalysisCorpusArtifactSize,
    /// Human-readable description of what the case tests.
    pub description: String,
}

impl AnalysisCorpusCaseMetadata {
    /// Build metadata for an inline synthetic case.
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
    /// Name of the metric, matched against threshold and limit names.
    pub name: String,
    /// Computed metric value.
    pub value: f32,
}

impl AnalysisMetricValue {
    /// Construct a named metric value.
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
    /// Name of the metric this threshold applies to.
    pub metric: String,
    /// Inclusive lower bound; `None` means no lower bound.
    pub min: Option<f32>,
    /// Inclusive upper bound; `None` means no upper bound.
    pub max: Option<f32>,
    /// Severity triggered when the metric falls outside `[min, max]`.
    pub severity: AcceptanceSeverity,
}

impl AcceptanceThreshold {
    /// Create an inclusive-range threshold for the named metric.
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
    /// Name of the metric this limit applies to.
    pub metric: String,
    /// Maximum tolerated absolute difference between baseline and candidate.
    pub max_abs_delta: f32,
    /// Severity triggered when the delta exceeds `max_abs_delta`.
    pub severity: AcceptanceSeverity,
}

impl RegressionDriftLimit {
    /// Create a drift limit for the named metric.
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
