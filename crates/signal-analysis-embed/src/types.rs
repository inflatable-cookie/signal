use signal_analysis::Confidence;
use signal_analysis_character::{CharacterAnalysisResult, CharacterAnalyzerConfig};

pub(crate) const DEFAULT_MAX_TAG_COUNT: usize = 3;

/// Semantic labels produced by the descriptor projection.
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
    /// Projected embedding vector ([`EMBEDDING_DIMENSIONS`](crate::EMBEDDING_DIMENSIONS) values).
    pub embedding: Vec<f32>,
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
    /// Maximum number of semantic tags to include in results.
    pub max_tag_count: usize,
}

impl Default for SemanticEmbedderConfig {
    fn default() -> Self {
        Self {
            character: CharacterAnalyzerConfig::default(),
            max_tag_count: DEFAULT_MAX_TAG_COUNT,
        }
    }
}
