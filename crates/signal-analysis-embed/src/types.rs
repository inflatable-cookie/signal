use signal_analysis::Confidence;
use signal_analysis_character::{CharacterAnalysisResult, CharacterAnalyzerConfig};

pub(crate) const DEFAULT_MAX_TAG_COUNT: usize = 3;

/// Fallback behavior when the requested semantic model cannot be loaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelFallbackBehavior {
    /// Fail the constructor if the requested model cannot be resolved.
    FailClosed,
    /// Use Signal's built-in deterministic descriptor model instead.
    UseBuiltInDescriptorV1,
}

/// Source family for an inference model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticModelSource {
    /// Model is compiled into the crate and requires no external resources.
    BuiltIn,
}

/// Version triple for a semantic inference model contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticModelVersion {
    /// Major version; breaking changes increment this.
    pub major: u16,
    /// Minor version; backward-compatible additions increment this.
    pub minor: u16,
    /// Patch version; backward-compatible fixes increment this.
    pub patch: u16,
}

impl SemanticModelVersion {
    /// Construct a version triple.
    pub const fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

/// Resource and determinism expectations for an inference model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticModelResourceProfile {
    /// Number of dimensions in the embedding vector this model produces.
    pub embedding_dimensions: usize,
    /// Whether the model produces identical output for identical inputs.
    pub deterministic: bool,
    /// Whether the model requires a network connection at inference time.
    pub requires_network: bool,
    /// Approximate heap allocation for the model and inference buffers, in bytes.
    pub estimated_heap_bytes: usize,
    /// Maximum duration the model will analyse; `None` means no cap.
    pub analysis_duration_cap_seconds: Option<u32>,
}

/// Public semantic model contract surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticModelSpec {
    /// Stable string identifier for the resolved model.
    pub model_id: &'static str,
    /// Version of the resolved model.
    pub version: SemanticModelVersion,
    /// Source family of the resolved model.
    pub source: SemanticModelSource,
    /// Fallback policy that was in effect when the model was resolved.
    pub fallback_behavior: ModelFallbackBehavior,
    /// Resource and determinism profile for the resolved model.
    pub resources: SemanticModelResourceProfile,
    /// Human-readable notes about the model's design or limitations.
    pub notes: &'static str,
}

/// Error returned when the requested model cannot be resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelLoadError {
    /// The model ID that was requested but could not be resolved.
    pub requested_model_id: String,
    /// The fallback policy that was in effect; `FailClosed` is the typical cause.
    pub fallback_behavior: ModelFallbackBehavior,
}

/// Semantic labels produced by the built-in descriptor model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticTagLabel {
    /// Strong pitched/harmonic content dominates the signal.
    TonalFocus,
    /// Broad-spectrum noise or non-harmonic texture dominates.
    TexturalNoise,
    /// Rhythmic transient activity is the primary character.
    PulseDriven,
    /// Extended sustain with slow amplitude envelope.
    SustainedBody,
    /// High crest factor: sharp transients against a quiet background.
    DynamicPunch,
}

/// Explainable evidence carried with each emitted semantic tag.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticTagEvidence {
    /// Name of the descriptor dimension most responsible for this tag.
    pub primary_driver: &'static str,
    /// Normalized value of `primary_driver` at inference time.
    pub primary_value: f32,
    /// Name of the secondary descriptor dimension contributing to this tag.
    pub supporting_driver: &'static str,
    /// Normalized value of `supporting_driver` at inference time.
    pub supporting_value: f32,
    /// Composite evidence strength; higher means the tag is better supported.
    pub evidence_strength: f32,
}

/// Ranked semantic tag with score and confidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticTag {
    /// Semantic label assigned to this tag.
    pub label: SemanticTagLabel,
    /// Raw projection score from the embedding (higher means stronger match).
    pub score: f32,
    /// Confidence in this tag; accounts for score margin and embedding activity.
    pub confidence: Confidence,
    /// Explainable evidence backing this tag.
    pub evidence: SemanticTagEvidence,
}

/// Deterministic embedding projected from the descriptor packs.
#[derive(Clone, Debug, PartialEq)]
pub struct DescriptorEmbedding {
    /// Stable identifier of the model that produced this embedding.
    pub model_id: &'static str,
    /// Version of the model that produced this embedding.
    pub version: SemanticModelVersion,
    /// Embedding vector; length matches `SemanticModelResourceProfile::embedding_dimensions`.
    pub values: Vec<f32>,
}

/// Component breakdown of the semantic confidence score.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticConfidenceDiagnostics {
    /// Contribution from the margin between the top and second-best tag scores.
    pub top_margin_component: f32,
    /// Contribution from the L2 norm of the embedding vector.
    pub embedding_activity_component: f32,
    /// Contribution from the upstream descriptor-pack confidence.
    pub descriptor_confidence_component: f32,
}

/// Diagnostics for the current semantic inference run.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticAnalysisDiagnostics {
    /// Confidence from the upstream character descriptor pack.
    pub descriptor_confidence: Confidence,
    /// Overall semantic confidence after combining all components.
    pub semantic_confidence: Confidence,
    /// Score margin between the top and second-best semantic tags.
    pub top_tag_margin: f32,
    /// Label of the highest-scoring semantic tag, if any tags were emitted.
    pub top_tag_label: Option<SemanticTagLabel>,
    /// Component breakdown of the semantic confidence score.
    pub confidence_components: SemanticConfidenceDiagnostics,
    /// L2 norm of the embedding vector.
    pub embedding_l2_norm: f32,
    /// Number of embedding dimensions with non-zero values.
    pub active_embedding_dimensions: usize,
    /// Whether the built-in fallback model was used instead of the requested one.
    pub fallback_used: bool,
}

/// Stable calibration evidence for one frozen semantic corpus case.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticCalibrationCaseReport {
    /// Stable identifier for this calibration case.
    pub case_id: &'static str,
    /// Highest-scoring semantic tag for this case.
    pub top_tag: SemanticTagLabel,
    /// Score of the top tag for this case.
    pub top_score: f32,
    /// Confidence in the top tag for this case.
    pub top_confidence: Confidence,
    /// Primary evidence driver for the top tag in this case.
    pub primary_driver: &'static str,
    /// Supporting evidence driver for the top tag in this case.
    pub supporting_driver: &'static str,
    /// Score margin between the top tag and the runner-up for this case.
    pub top_tag_margin: f32,
}

/// Machine-readable frozen semantic calibration surface.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticCalibrationReport {
    /// Per-case calibration reports for the frozen corpus.
    pub case_reports: Vec<SemanticCalibrationCaseReport>,
}

/// Semantic inference result built on top of shared character descriptors.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticAnalysisResult {
    /// Character descriptor packs used as input to embedding.
    pub source_descriptors: CharacterAnalysisResult,
    /// Projected embedding vector.
    pub embedding: DescriptorEmbedding,
    /// Ranked semantic tags, up to `SemanticEmbedderConfig::max_tag_count`.
    pub semantic_tags: Vec<SemanticTag>,
    /// Inference diagnostics and confidence breakdown.
    pub diagnostics: SemanticAnalysisDiagnostics,
}

/// Configuration for the semantic embedder.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticEmbedderConfig {
    /// Configuration forwarded to the internal character analyzer.
    pub character: CharacterAnalyzerConfig,
    /// Model identifier to resolve; `None` selects the built-in default.
    pub requested_model_id: Option<String>,
    /// Behavior when the requested model cannot be resolved.
    pub fallback_behavior: ModelFallbackBehavior,
    /// Maximum number of semantic tags to include in results.
    pub max_tag_count: usize,
}

impl Default for SemanticEmbedderConfig {
    fn default() -> Self {
        Self {
            character: CharacterAnalyzerConfig::default(),
            requested_model_id: None,
            fallback_behavior: ModelFallbackBehavior::UseBuiltInDescriptorV1,
            max_tag_count: DEFAULT_MAX_TAG_COUNT,
        }
    }
}
