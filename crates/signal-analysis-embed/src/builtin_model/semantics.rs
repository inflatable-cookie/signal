use signal_analysis::Confidence;
use signal_analysis_character::CharacterAnalysisResult;

use crate::types::{
    DescriptorEmbedding, SemanticAnalysisDiagnostics, SemanticAnalysisResult, SemanticTag,
    SemanticTagLabel,
};

use super::{normalize_unit, BuiltInDescriptorSemanticModel, BUILTIN_DESCRIPTOR_MODEL_ID};

impl BuiltInDescriptorSemanticModel {
    pub(crate) fn semantic_tags_from_descriptors(
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

    pub(crate) fn semantic_confidence(
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

    pub(crate) fn build_analysis_result(
        &self,
        descriptors: CharacterAnalysisResult,
        max_tag_count: usize,
        fallback_used: bool,
    ) -> SemanticAnalysisResult {
        let embedding_values = self.embedding_from_descriptors(&descriptors);
        let semantic_tags = self.semantic_tags_from_descriptors(&descriptors, max_tag_count);
        let descriptor_confidence = descriptors.confidence;
        let semantic_confidence =
            self.semantic_confidence(&semantic_tags, descriptor_confidence, &embedding_values);
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
                version: self.version,
                values: embedding_values,
            },
            semantic_tags,
            diagnostics: SemanticAnalysisDiagnostics {
                descriptor_confidence,
                semantic_confidence,
                top_tag_margin,
                embedding_l2_norm,
                active_embedding_dimensions,
                fallback_used,
            },
        }
    }
}

fn descriptor_scaled_confidence(base: Confidence, evidence_strength: f32) -> Confidence {
    Confidence::new(base.0 * (0.45 + 0.55 * normalize_unit(evidence_strength)))
}
