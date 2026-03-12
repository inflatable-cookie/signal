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
mod tests {
    use super::*;
    use signal_analysis::{
        run_audio_acceptance_harness, AcceptanceSeverity, AcceptanceStatus, AnalysisCorpusCase,
        AnalysisCorpusCaseMetadata, AnalysisCorpusFamily, AnalysisMetricValue,
    };
    use signal_primitives::{ChannelLayout, SampleRate};

    fn sine_audio(
        frequency_hz: f32,
        duration_seconds: f32,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        for (index, sample) in data.iter_mut().enumerate() {
            let time = index as f32 / sample_rate_hz as f32;
            *sample = amplitude * (core::f32::consts::TAU * frequency_hz * time).sin();
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn noise_audio(duration_seconds: f32, sample_rate_hz: u32, amplitude: f32) -> AudioBuffer {
        let count = (duration_seconds * sample_rate_hz as f32) as usize;
        let mut data = vec![0.0f32; count];
        let mut state = 0x1234_5678u32;
        for sample in &mut data {
            state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let unit = ((state >> 8) as f32 / u32::MAX as f32) * 2.0 - 1.0;
            *sample = amplitude * unit;
        }
        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn adsr_pulse_audio(
        attack_ms: u32,
        sustain_ms: u32,
        decay_ms: u32,
        interval_ms: u32,
        event_count: usize,
        sample_rate_hz: u32,
        amplitude: f32,
    ) -> AudioBuffer {
        let interval_samples = (interval_ms as usize * sample_rate_hz as usize) / 1_000;
        let attack_samples = (attack_ms as usize * sample_rate_hz as usize) / 1_000;
        let sustain_samples = (sustain_ms as usize * sample_rate_hz as usize) / 1_000;
        let decay_samples = (decay_ms as usize * sample_rate_hz as usize) / 1_000;
        let total_samples = interval_samples * event_count.max(1);
        let mut data = vec![0.0f32; total_samples.max(1)];

        for event_index in 0..event_count {
            let start = event_index * interval_samples;

            for offset in 0..attack_samples {
                let index = start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = (offset + 1) as f32 / attack_samples.max(1) as f32;
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }

            let sustain_start = start + attack_samples;
            for offset in 0..sustain_samples {
                let index = sustain_start + offset;
                if index >= data.len() {
                    break;
                }
                data[index] = amplitude;
            }

            let decay_start = sustain_start + sustain_samples;
            for offset in 0..decay_samples {
                let index = decay_start + offset;
                if index >= data.len() {
                    break;
                }
                let progress = 1.0 - ((offset + 1) as f32 / decay_samples.max(1) as f32);
                data[index] = amplitude * progress.clamp(0.0, 1.0);
            }
        }

        AudioBuffer::from_interleaved(SampleRate(sample_rate_hz), ChannelLayout::Mono, data)
    }

    fn top_label(result: &SemanticAnalysisResult) -> SemanticTagLabel {
        result.semantic_tags.first().map(|tag| tag.label).unwrap()
    }

    fn semantic_score(result: &SemanticAnalysisResult, label: SemanticTagLabel) -> f32 {
        result
            .semantic_tags
            .iter()
            .find(|tag| tag.label == label)
            .map(|tag| tag.score)
            .unwrap_or(0.0)
    }

    fn semantic_metrics(result: &SemanticAnalysisResult) -> Vec<AnalysisMetricValue> {
        vec![
            AnalysisMetricValue::new(
                "tonal_focus_score",
                semantic_score(result, SemanticTagLabel::TonalFocus),
            ),
            AnalysisMetricValue::new(
                "textural_noise_score",
                semantic_score(result, SemanticTagLabel::TexturalNoise),
            ),
            AnalysisMetricValue::new(
                "pulse_driven_score",
                semantic_score(result, SemanticTagLabel::PulseDriven),
            ),
            AnalysisMetricValue::new(
                "dynamic_punch_score",
                semantic_score(result, SemanticTagLabel::DynamicPunch),
            ),
            AnalysisMetricValue::new(
                "semantic_confidence",
                result.diagnostics.semantic_confidence.0,
            ),
            AnalysisMetricValue::new(
                "descriptor_confidence",
                result.diagnostics.descriptor_confidence.0,
            ),
        ]
    }

    fn semantic_acceptance_cases() -> Vec<AnalysisCorpusCase> {
        vec![
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "semantic:tone:sine440",
                    AnalysisCorpusFamily::Semantic,
                    "Tonal semantic reference",
                ),
                sine_audio(440.0, 2.0, 48_000, 1.0),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "tonal_focus_score",
                    Some(0.60),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "semantic_confidence",
                    Some(0.03),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "semantic:noise:deterministic",
                    AnalysisCorpusFamily::Semantic,
                    "Noise semantic reference",
                ),
                noise_audio(2.0, 48_000, 0.5),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "textural_noise_score",
                    Some(0.50),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "semantic_confidence",
                    Some(0.03),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
            AnalysisCorpusCase::new(
                AnalysisCorpusCaseMetadata::synthetic(
                    "semantic:pulse:adsr",
                    AnalysisCorpusFamily::Semantic,
                    "Pulse semantic reference",
                ),
                adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9),
            )
            .with_acceptance_thresholds(vec![
                signal_analysis::AcceptanceThreshold::range(
                    "pulse_driven_score",
                    Some(0.40),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "dynamic_punch_score",
                    Some(0.40),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "semantic_confidence",
                    Some(0.03),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
                signal_analysis::AcceptanceThreshold::range(
                    "descriptor_confidence",
                    Some(0.25),
                    Some(1.0),
                    AcceptanceSeverity::Fail,
                ),
            ]),
        ]
    }

    #[test]
    fn built_in_model_spec_is_explicit() {
        let embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
        let spec = embedder.model_spec();

        assert_eq!(spec.model_id, BUILTIN_DESCRIPTOR_MODEL_ID);
        assert_eq!(spec.version, SemanticModelVersion::new(1, 0, 0));
        assert_eq!(spec.resources.embedding_dimensions, EMBEDDING_DIMENSIONS);
        assert!(spec.resources.deterministic);
        assert!(!spec.resources.requires_network);
    }

    #[test]
    fn unknown_model_fails_closed_when_requested() {
        let error = SemanticEmbedder::new(SemanticEmbedderConfig {
            requested_model_id: Some("signal:missing-model".to_string()),
            fallback_behavior: ModelFallbackBehavior::FailClosed,
            ..SemanticEmbedderConfig::default()
        })
        .unwrap_err();

        assert_eq!(error.requested_model_id, "signal:missing-model");
        assert_eq!(error.fallback_behavior, ModelFallbackBehavior::FailClosed);
    }

    #[test]
    fn unknown_model_can_fallback_to_builtin() {
        let embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
            requested_model_id: Some("signal:missing-model".to_string()),
            fallback_behavior: ModelFallbackBehavior::UseBuiltInDescriptorV1,
            ..SemanticEmbedderConfig::default()
        })
        .unwrap();

        assert_eq!(embedder.model_spec().model_id, BUILTIN_DESCRIPTOR_MODEL_ID);
    }

    #[test]
    fn tonal_audio_prefers_tonal_focus() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
        let result = embedder.analyze(&audio);

        assert_eq!(top_label(&result), SemanticTagLabel::TonalFocus);
        assert_eq!(result.embedding.values.len(), EMBEDDING_DIMENSIONS);
        assert!(result.source_descriptors.spectral_shape.flatness < 1e-4);
    }

    #[test]
    fn noisy_audio_prefers_textural_noise() {
        let audio = noise_audio(2.0, 48_000, 0.5);
        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
        let result = embedder.analyze(&audio);

        assert_eq!(top_label(&result), SemanticTagLabel::TexturalNoise);
        assert!(result.source_descriptors.spectral_shape.spread_hz > 1_000.0);
    }

    #[test]
    fn pulse_audio_prefers_pulse_driven_or_dynamic_punch() {
        let audio = adsr_pulse_audio(5, 120, 100, 500, 6, 48_000, 0.9);
        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
        let result = embedder.analyze(&audio);

        assert!(matches!(
            top_label(&result),
            SemanticTagLabel::PulseDriven | SemanticTagLabel::DynamicPunch
        ));
        assert!(
            result
                .source_descriptors
                .temporal_shape
                .peak_transient_strength
                > 0.9
        );
        assert!(result.diagnostics.top_tag_margin >= 0.0);
    }

    #[test]
    fn max_tag_count_limits_ranked_output() {
        let audio = sine_audio(440.0, 2.0, 48_000, 1.0);
        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
            max_tag_count: 2,
            ..SemanticEmbedderConfig::default()
        })
        .unwrap();
        let result = embedder.analyze(&audio);

        assert_eq!(result.semantic_tags.len(), 2);
    }

    #[test]
    fn semantic_diagnostics_are_bounded() {
        let audio = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);
        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
        let result = embedder.analyze(&audio);

        assert!(result.diagnostics.embedding_l2_norm.is_finite());
        assert!(result.diagnostics.active_embedding_dimensions > 0);
        assert!(result.diagnostics.semantic_confidence.0 >= 0.0);
        assert!(result.diagnostics.semantic_confidence.0 <= 1.0);
    }

    #[test]
    fn harness_semantic_cases_meet_frozen_acceptance_thresholds() {
        let cases = semantic_acceptance_cases();
        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();

        let report =
            run_audio_acceptance_harness(&cases, |audio| embedder.analyze(audio), semantic_metrics);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert!(report
            .cases
            .iter()
            .all(|case| case.status == AcceptanceStatus::Pass));
    }

    #[test]
    fn frozen_semantic_acceptance_report_remains_interpretable_for_closeout() {
        let cases = semantic_acceptance_cases();
        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();

        let report =
            run_audio_acceptance_harness(&cases, |audio| embedder.analyze(audio), semantic_metrics);

        println!("semantic_acceptance_report={:#?}", report);

        assert_eq!(report.status, AcceptanceStatus::Pass);
        assert_eq!(report.cases.len(), 3);
    }

    #[test]
    fn semantic_examples_remain_interpretable_for_closeout() {
        let tone = sine_audio(440.0, 2.0, 48_000, 1.0);
        let noise = noise_audio(2.0, 48_000, 0.5);
        let pulse = adsr_pulse_audio(5, 140, 120, 500, 6, 48_000, 0.9);

        let mut embedder = SemanticEmbedder::new(SemanticEmbedderConfig::default()).unwrap();
        let tone_result = embedder.analyze(&tone);
        let noise_result = embedder.analyze(&noise);
        let pulse_result = embedder.analyze(&pulse);

        let mut fallback_embedder = SemanticEmbedder::new(SemanticEmbedderConfig {
            requested_model_id: Some("signal:missing-model".to_string()),
            fallback_behavior: ModelFallbackBehavior::UseBuiltInDescriptorV1,
            ..SemanticEmbedderConfig::default()
        })
        .unwrap();
        let fallback_result = fallback_embedder.analyze(&tone);

        println!("tone_semantic={:#?}", tone_result);
        println!("noise_semantic={:#?}", noise_result);
        println!("pulse_semantic={:#?}", pulse_result);
        println!("fallback_semantic={:#?}", fallback_result);

        assert_eq!(top_label(&tone_result), SemanticTagLabel::TonalFocus);
        assert_eq!(top_label(&noise_result), SemanticTagLabel::TexturalNoise);
        assert!(matches!(
            top_label(&pulse_result),
            SemanticTagLabel::PulseDriven | SemanticTagLabel::DynamicPunch
        ));
        assert!(fallback_result.diagnostics.fallback_used);
        assert_eq!(
            fallback_result.embedding.model_id,
            BUILTIN_DESCRIPTOR_MODEL_ID
        );
        assert!(tone_result.diagnostics.semantic_confidence.0 > 0.0);
        assert!(noise_result.diagnostics.semantic_confidence.0 > 0.0);
        assert!(pulse_result.diagnostics.semantic_confidence.0 > 0.0);
    }
}
