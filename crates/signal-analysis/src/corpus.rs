//! Corpus and test case types for analysis validation.
//!
//! This module provides types for defining analysis test corpora, including
//! metadata, thresholds, and drift limits for acceptance and regression testing.

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
