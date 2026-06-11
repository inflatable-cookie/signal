//! Descriptor-based semantic projection for Signal.
//!
//! The crate projects shared character descriptor packs into a small
//! deterministic descriptor vector and matches it against hand-written
//! semantic tag prototypes. There is no learned model here — just explicit,
//! explainable descriptor arithmetic.
//!
//! ```no_run
//! use signal_analysis::AnalysisStage;
//! use signal_analysis_embed::{SemanticEmbedder, SemanticEmbedderConfig, EMBEDDING_DIMENSIONS};
//! use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};
//!
//! let audio = AudioBuffer::from_interleaved(
//!     SampleRate(48_000),
//!     ChannelLayout::Mono,
//!     vec![0.0; 48_000],
//! );
//! let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default());
//! let result = embedder.analyze(&audio);
//!
//! assert_eq!(embedder.mode(), signal_analysis::AnalysisMode::Offline);
//! assert_eq!(result.embedding.len(), EMBEDDING_DIMENSIONS);
//! ```

#![warn(missing_docs)]

use signal_analysis::{AnalysisMode, AnalysisStage};
use signal_analysis_character::{CharacterAnalysisResult, CharacterAnalyzer};
use signal_primitives::AudioBuffer;

mod projection;
mod types;

pub use projection::{descriptor_embedding, semantic_tags, EMBEDDING_DIMENSIONS};
pub use types::{
    SemanticAnalysisDiagnostics, SemanticAnalysisResult, SemanticCalibrationCaseReport,
    SemanticCalibrationReport, SemanticConfidenceDiagnostics, SemanticEmbedderConfig, SemanticTag,
    SemanticTagEvidence, SemanticTagLabel,
};

/// Offline semantic embedder that projects shared descriptor packs into a
/// deterministic embedding and ranked semantic tags.
#[derive(Debug)]
pub struct SemanticEmbedder {
    config: SemanticEmbedderConfig,
    character_analyzer: CharacterAnalyzer,
}

impl SemanticEmbedder {
    /// Create an embedder with the provided config.
    pub fn new(config: SemanticEmbedderConfig) -> Self {
        Self {
            character_analyzer: CharacterAnalyzer::new(config.character),
            config,
        }
    }

    /// Project precomputed character descriptors into the embedding space.
    pub fn embed_descriptors(
        &self,
        descriptors: CharacterAnalysisResult,
    ) -> SemanticAnalysisResult {
        projection::build_analysis_result(descriptors, self.config.max_tag_count)
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
