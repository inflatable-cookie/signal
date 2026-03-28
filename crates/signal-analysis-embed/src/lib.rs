//! Descriptor-based embedding and semantic inference for Signal.
//!
//! The crate owns a first host-neutral semantic-analysis boundary built on top
//! of shared descriptor packs rather than app-local feature extraction.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_embed::{ModelFallbackBehavior, SemanticEmbedder, SemanticEmbedderConfig};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
//!     fallback_behavior: ModelFallbackBehavior::UseBuiltInDescriptorV1,
//!     ..SemanticEmbedderConfig::default()
//! })
//! .expect("built-in semantic model should load");
//! let result = embedder.analyze(&audio);
//!
//! assert_eq!(embedder.mode(), signal_analysis::AnalysisMode::Offline);
//! assert_eq!(result.embedding.values.len(), 8);
//! ```

use signal_analysis::{AnalysisMode, AnalysisStage};
use signal_analysis_character::{CharacterAnalysisResult, CharacterAnalyzer};
use signal_primitives::AudioBuffer;

mod builtin_model;
mod types;

pub use types::{
    DescriptorEmbedding, ModelFallbackBehavior, ModelLoadError, SemanticAnalysisDiagnostics,
    SemanticAnalysisResult, SemanticEmbedderConfig, SemanticModelResourceProfile,
    SemanticModelSource, SemanticModelSpec, SemanticModelVersion, SemanticTag, SemanticTagLabel,
};

use builtin_model::{BuiltInDescriptorSemanticModel, BUILTIN_DESCRIPTOR_MODEL_ID};

/// Offline semantic embedder that projects shared descriptor packs into a
/// deterministic embedding and ranked semantic tags.
#[derive(Debug)]
pub struct SemanticEmbedder {
    config: SemanticEmbedderConfig,
    character_analyzer: CharacterAnalyzer,
    model: BuiltInDescriptorSemanticModel,
    fallback_used: bool,
}

impl SemanticEmbedder {
    /// Resolve the requested model or configured fallback.
    pub fn new(config: SemanticEmbedderConfig) -> Result<Self, ModelLoadError> {
        let requested_model_id = config
            .requested_model_id
            .as_deref()
            .unwrap_or(BUILTIN_DESCRIPTOR_MODEL_ID);
        let (model, fallback_used) =
            BuiltInDescriptorSemanticModel::resolve(requested_model_id, &config)?;

        Ok(Self {
            character_analyzer: CharacterAnalyzer::new(config.character),
            config,
            model,
            fallback_used,
        })
    }

    /// Return the resolved semantic model contract.
    pub fn model_spec(&self) -> SemanticModelSpec {
        self.model
            .spec(self.config.character, self.config.fallback_behavior)
    }

    /// Project precomputed character descriptors into the embedding space.
    pub fn embed_descriptors(
        &self,
        descriptors: CharacterAnalysisResult,
    ) -> SemanticAnalysisResult {
        self.model
            .build_analysis_result(descriptors, self.config.max_tag_count, self.fallback_used)
    }
}

impl AnalysisStage<SemanticAnalysisResult> for SemanticEmbedder {
    fn mode(&self) -> AnalysisMode {
        AnalysisMode::Offline
    }

    fn analyze(&mut self, audio: &AudioBuffer) -> SemanticAnalysisResult {
        let descriptors = self.character_analyzer.analyze(audio);
        self.embed_descriptors(descriptors)
    }
}
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
