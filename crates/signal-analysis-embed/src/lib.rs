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

use signal_analysis::{AnalysisMode, AnalysisStage, Confidence};
use signal_analysis_character::{
    CharacterAnalysisResult, CharacterAnalyzer, CharacterAnalyzerConfig,
};
use signal_primitives::AudioBuffer;

const BUILTIN_DESCRIPTOR_MODEL_ID: &str = "signal:descriptor-embed:v1";
const BUILTIN_DESCRIPTOR_MODEL_NOTES: &str =
    "Built-in deterministic descriptor projection over shared character packs.";
const EMBEDDING_DIMENSIONS: usize = 8;
const DEFAULT_MAX_TAG_COUNT: usize = 3;

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
        let embedding_values = self.model.embedding_from_descriptors(&descriptors);
        let semantic_tags = self
            .model
            .semantic_tags_from_descriptors(&descriptors, self.config.max_tag_count);
        let descriptor_confidence = descriptors.confidence;
        let semantic_confidence = self.model.semantic_confidence(
            &semantic_tags,
            descriptor_confidence,
            &embedding_values,
        );
        let embedding_l2_norm = embedding_values
            .iter()
            .copied()
            .map(|value| value * value)
            .sum::<f32>()
            .sqrt();
        let top_tag_margin = if semantic_tags.len() >= 2 {
            (semantic_tags[0].score - semantic_tags[1].score).max(0.0)
        } else {
            semantic_tags.first().map(|tag| tag.score).unwrap_or(0.0)
        };
        let active_embedding_dimensions = embedding_values
            .iter()
            .filter(|value| value.abs() > 1e-6)
            .count();

        SemanticAnalysisResult {
            source_descriptors: descriptors,
            embedding: DescriptorEmbedding {
                model_id: BUILTIN_DESCRIPTOR_MODEL_ID,
                version: self.model.version,
                values: embedding_values,
            },
            semantic_tags,
            diagnostics: SemanticAnalysisDiagnostics {
                descriptor_confidence,
                semantic_confidence,
                top_tag_margin,
                embedding_l2_norm,
                active_embedding_dimensions,
                fallback_used: self.fallback_used,
            },
        }
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

#[derive(Clone, Copy, Debug)]
struct BuiltInDescriptorSemanticModel {
    version: SemanticModelVersion,
}

impl BuiltInDescriptorSemanticModel {
    fn resolve(
        requested_model_id: &str,
        config: &SemanticEmbedderConfig,
    ) -> Result<(Self, bool), ModelLoadError> {
        if requested_model_id == BUILTIN_DESCRIPTOR_MODEL_ID {
            return Ok((Self::builtin_v1(), false));
        }

        match config.fallback_behavior {
            ModelFallbackBehavior::UseBuiltInDescriptorV1 => Ok((Self::builtin_v1(), true)),
            ModelFallbackBehavior::FailClosed => Err(ModelLoadError {
                requested_model_id: requested_model_id.to_string(),
                fallback_behavior: config.fallback_behavior,
            }),
        }
    }

    const fn builtin_v1() -> Self {
        Self {
            version: SemanticModelVersion::new(1, 0, 0),
        }
    }

    fn spec(
        &self,
        character_config: CharacterAnalyzerConfig,
        fallback_behavior: ModelFallbackBehavior,
    ) -> SemanticModelSpec {
        SemanticModelSpec {
            model_id: BUILTIN_DESCRIPTOR_MODEL_ID,
            version: self.version,
            source: SemanticModelSource::BuiltIn,
            fallback_behavior,
            resources: SemanticModelResourceProfile {
                embedding_dimensions: EMBEDDING_DIMENSIONS,
                deterministic: true,
                requires_network: false,
                estimated_heap_bytes: EMBEDDING_DIMENSIONS * core::mem::size_of::<f32>(),
                analysis_duration_cap_seconds: character_config.analysis_duration_seconds,
            },
            notes: BUILTIN_DESCRIPTOR_MODEL_NOTES,
        }
    }

    fn embedding_from_descriptors(&self, descriptors: &CharacterAnalysisResult) -> Vec<f32> {
        let spectral_shape = &descriptors.spectral_shape;
        let spectral_contrast = &descriptors.spectral_contrast;
        let spectral_profile = &descriptors.spectral_profile.normalized_mel_band_profile;
        let temporal = &descriptors.temporal;
        let temporal_shape = &descriptors.temporal_shape;
        let dynamics = &descriptors.dynamics;

        let brightness = normalize_log_hz(spectral_shape.centroid_hz, 20.0, 12_000.0);
        let spectral_complexity = normalize_unit(spectral_shape.spread_hz / 4_000.0);
        let noisiness = normalize_unit((spectral_shape.flatness * 10.0).sqrt());
        let harmonic_focus = normalize_unit(
            spectral_profile[0] * 0.55
                + spectral_profile[1] * 0.25
                + normalize_unit(spectral_contrast.contrast_db / 40.0) * 0.20,
        );
        let rhythmic_activity = normalize_unit(
            normalize_unit(temporal.onset_density / 4.0) * 0.65
                + temporal_shape.peak_transient_strength * 0.35,
        );
        let sustain_body = normalize_unit(
            temporal.sustain_ratio * 0.55 + temporal_shape.sustain_plateau_ratio * 0.45,
        );
        let dynamic_punch = normalize_unit(
            normalize_unit(dynamics.dynamic_range / 0.7) * 0.45
                + temporal_shape.peak_transient_strength * 0.35
                + (1.0 - sustain_body) * 0.20,
        );
        let low_band_weight = normalize_unit(spectral_profile[0] + spectral_profile[1] * 0.5);

        vec![
            brightness,
            spectral_complexity,
            noisiness,
            harmonic_focus,
            rhythmic_activity,
            sustain_body,
            dynamic_punch,
            low_band_weight,
        ]
    }

    fn semantic_tags_from_descriptors(
        &self,
        descriptors: &CharacterAnalysisResult,
        max_tag_count: usize,
    ) -> Vec<SemanticTag> {
        let embedding = self.embedding_from_descriptors(descriptors);
        let descriptor_confidence = descriptors.confidence;
        let brightness = embedding[0];
        let spectral_complexity = embedding[1];
        let noisiness = embedding[2];
        let harmonic_focus = embedding[3];
        let rhythmic_activity = embedding[4];
        let sustain_body = embedding[5];
        let dynamic_punch = embedding[6];
        let contrast_norm = normalize_unit(descriptors.spectral_contrast.contrast_db / 40.0);

        let mut tags = vec![
            SemanticTag {
                label: SemanticTagLabel::TonalFocus,
                score: normalize_unit(
                    harmonic_focus * 0.35
                        + contrast_norm * 0.30
                        + brightness * 0.20
                        + (1.0 - spectral_complexity) * 0.15,
                ),
                confidence: descriptor_scaled_confidence(descriptor_confidence, harmonic_focus),
            },
            SemanticTag {
                label: SemanticTagLabel::TexturalNoise,
                score: normalize_unit(
                    spectral_complexity * 0.45
                        + (1.0 - contrast_norm) * 0.35
                        + normalize_unit((noisiness + (1.0 - harmonic_focus)) * 0.5) * 0.20,
                ),
                confidence: descriptor_scaled_confidence(
                    descriptor_confidence,
                    normalize_unit((spectral_complexity + (1.0 - contrast_norm)) * 0.5),
                ),
            },
            SemanticTag {
                label: SemanticTagLabel::PulseDriven,
                score: normalize_unit(
                    rhythmic_activity * 0.45
                        + descriptors.temporal_shape.peak_transient_strength * 0.35
                        + (1.0 - sustain_body) * 0.20,
                ),
                confidence: descriptor_scaled_confidence(
                    descriptor_confidence,
                    normalize_unit((rhythmic_activity + dynamic_punch) * 0.5),
                ),
            },
            SemanticTag {
                label: SemanticTagLabel::SustainedBody,
                score: normalize_unit(
                    sustain_body * 0.55
                        + normalize_unit(descriptors.temporal_shape.decay_time_ms / 300.0) * 0.25
                        + (1.0 - rhythmic_activity) * 0.20,
                ),
                confidence: descriptor_scaled_confidence(descriptor_confidence, sustain_body),
            },
            SemanticTag {
                label: SemanticTagLabel::DynamicPunch,
                score: normalize_unit(
                    dynamic_punch * 0.45
                        + descriptors.temporal_shape.peak_transient_strength * 0.30
                        + normalize_unit(descriptors.dynamics.dynamic_range / 0.7) * 0.25,
                ),
                confidence: descriptor_scaled_confidence(descriptor_confidence, dynamic_punch),
            },
        ];

        tags.sort_by(|left, right| {
            right
                .score
                .partial_cmp(&left.score)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        tags.truncate(max_tag_count.max(1).min(tags.len()));
        tags
    }

    fn semantic_confidence(
        &self,
        semantic_tags: &[SemanticTag],
        descriptor_confidence: Confidence,
        embedding_values: &[f32],
    ) -> Confidence {
        let top_margin = if semantic_tags.len() >= 2 {
            (semantic_tags[0].score - semantic_tags[1].score).max(0.0)
        } else {
            semantic_tags.first().map(|tag| tag.score).unwrap_or(0.0)
        };
        let embedding_activity =
            embedding_values.iter().copied().sum::<f32>() / embedding_values.len().max(1) as f32;
        Confidence::new(
            descriptor_confidence.0 * normalize_unit(0.55 * top_margin + 0.45 * embedding_activity),
        )
    }
}

fn normalize_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn normalize_log_hz(value_hz: f32, low_hz: f32, high_hz: f32) -> f32 {
    if value_hz <= 0.0 || low_hz <= 0.0 || high_hz <= low_hz {
        return 0.0;
    }

    let low = low_hz.log2();
    let high = high_hz.log2();
    normalize_unit((value_hz.log2() - low) / (high - low))
}

fn descriptor_scaled_confidence(base: Confidence, evidence_strength: f32) -> Confidence {
    Confidence::new(base.0 * (0.45 + 0.55 * normalize_unit(evidence_strength)))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
