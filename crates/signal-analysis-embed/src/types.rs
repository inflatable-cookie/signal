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
    BuiltIn,
}

/// Version triple for a semantic inference model contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticModelVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl SemanticModelVersion {
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
    pub embedding_dimensions: usize,
    pub deterministic: bool,
    pub requires_network: bool,
    pub estimated_heap_bytes: usize,
    pub analysis_duration_cap_seconds: Option<u32>,
}

/// Public semantic model contract surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SemanticModelSpec {
    pub model_id: &'static str,
    pub version: SemanticModelVersion,
    pub source: SemanticModelSource,
    pub fallback_behavior: ModelFallbackBehavior,
    pub resources: SemanticModelResourceProfile,
    pub notes: &'static str,
}

/// Error returned when the requested model cannot be resolved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ModelLoadError {
    pub requested_model_id: String,
    pub fallback_behavior: ModelFallbackBehavior,
}

/// Semantic labels produced by the built-in descriptor model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticTagLabel {
    TonalFocus,
    TexturalNoise,
    PulseDriven,
    SustainedBody,
    DynamicPunch,
}

/// Ranked semantic tag with score and confidence.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticTag {
    pub label: SemanticTagLabel,
    pub score: f32,
    pub confidence: Confidence,
}

/// Deterministic embedding projected from the descriptor packs.
#[derive(Clone, Debug, PartialEq)]
pub struct DescriptorEmbedding {
    pub model_id: &'static str,
    pub version: SemanticModelVersion,
    pub values: Vec<f32>,
}

/// Diagnostics for the current semantic inference run.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticAnalysisDiagnostics {
    pub descriptor_confidence: Confidence,
    pub semantic_confidence: Confidence,
    pub top_tag_margin: f32,
    pub embedding_l2_norm: f32,
    pub active_embedding_dimensions: usize,
    pub fallback_used: bool,
}

/// Semantic inference result built on top of shared character descriptors.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticAnalysisResult {
    pub source_descriptors: CharacterAnalysisResult,
    pub embedding: DescriptorEmbedding,
    pub semantic_tags: Vec<SemanticTag>,
    pub diagnostics: SemanticAnalysisDiagnostics,
}

/// Configuration for the semantic embedder.
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticEmbedderConfig {
    pub character: CharacterAnalyzerConfig,
    pub requested_model_id: Option<String>,
    pub fallback_behavior: ModelFallbackBehavior,
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
