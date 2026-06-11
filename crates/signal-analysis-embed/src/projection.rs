use signal_analysis::Confidence;
use signal_analysis_character::CharacterAnalysisResult;

use crate::types::{
    SemanticAnalysisDiagnostics, SemanticAnalysisResult, SemanticConfidenceDiagnostics,
    SemanticTag, SemanticTagEvidence, SemanticTagLabel,
};

/// Number of dimensions in the descriptor embedding.
pub const EMBEDDING_DIMENSIONS: usize = 8;

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

/// Project shared character descriptors into a deterministic
/// [`EMBEDDING_DIMENSIONS`]-dimensional vector of hand-weighted descriptor
/// combinations, each normalized to `0.0..=1.0`.
pub fn descriptor_embedding(descriptors: &CharacterAnalysisResult) -> Vec<f32> {
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
    let sustain_body =
        normalize_unit(temporal.sustain_ratio * 0.55 + temporal_shape.sustain_plateau_ratio * 0.45);
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

/// Match the descriptor embedding against the built-in semantic tag
/// prototypes and return up to `max_tag_count` ranked tags.
pub fn semantic_tags(
    descriptors: &CharacterAnalysisResult,
    max_tag_count: usize,
) -> Vec<SemanticTag> {
    let embedding = descriptor_embedding(descriptors);
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
            evidence: tag_evidence(
                "harmonic_focus",
                harmonic_focus,
                "spectral_contrast",
                contrast_norm,
            ),
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
            evidence: tag_evidence(
                "spectral_complexity",
                spectral_complexity,
                "inverse_spectral_contrast",
                1.0 - contrast_norm,
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
            evidence: tag_evidence(
                "rhythmic_activity",
                rhythmic_activity,
                "peak_transient_strength",
                descriptors.temporal_shape.peak_transient_strength,
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
            evidence: tag_evidence(
                "sustain_body",
                sustain_body,
                "decay_time",
                normalize_unit(descriptors.temporal_shape.decay_time_ms / 300.0),
            ),
        },
        SemanticTag {
            label: SemanticTagLabel::DynamicPunch,
            score: normalize_unit(
                dynamic_punch * 0.45
                    + descriptors.temporal_shape.peak_transient_strength * 0.30
                    + normalize_unit(descriptors.dynamics.dynamic_range / 0.7) * 0.25,
            ),
            confidence: descriptor_scaled_confidence(descriptor_confidence, dynamic_punch),
            evidence: tag_evidence(
                "dynamic_punch",
                dynamic_punch,
                "peak_transient_strength",
                descriptors.temporal_shape.peak_transient_strength,
            ),
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
    tags: &[SemanticTag],
    descriptor_confidence: Confidence,
    embedding_values: &[f32],
) -> (Confidence, SemanticConfidenceDiagnostics) {
    let top_margin = if tags.len() >= 2 {
        (tags[0].score - tags[1].score).max(0.0)
    } else {
        tags.first().map(|tag| tag.score).unwrap_or(0.0)
    };
    let embedding_activity =
        embedding_values.iter().copied().sum::<f32>() / embedding_values.len().max(1) as f32;
    let confidence_components = SemanticConfidenceDiagnostics {
        top_margin_component: normalize_unit(top_margin),
        embedding_activity_component: normalize_unit(embedding_activity),
        descriptor_confidence_component: descriptor_confidence.0,
    };
    let calibrated_signal = normalize_unit(
        0.60 * confidence_components.top_margin_component
            + 0.40 * confidence_components.embedding_activity_component,
    );
    let calibrated = confidence_components.descriptor_confidence_component * calibrated_signal;
    (Confidence::new(calibrated), confidence_components)
}

pub(crate) fn build_analysis_result(
    descriptors: CharacterAnalysisResult,
    max_tag_count: usize,
) -> SemanticAnalysisResult {
    let embedding_values = descriptor_embedding(&descriptors);
    let tags = semantic_tags(&descriptors, max_tag_count);
    let descriptor_confidence = descriptors.confidence;
    let (semantic_confidence, confidence_components) =
        semantic_confidence(&tags, descriptor_confidence, &embedding_values);
    let embedding_l2_norm = embedding_values
        .iter()
        .copied()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    let top_tag_margin = if tags.len() >= 2 {
        (tags[0].score - tags[1].score).max(0.0)
    } else {
        tags.first().map(|tag| tag.score).unwrap_or(0.0)
    };
    let top_tag_label = tags.first().map(|tag| tag.label);
    let active_embedding_dimensions = embedding_values
        .iter()
        .filter(|value| value.abs() > 1e-6)
        .count();

    SemanticAnalysisResult {
        source_descriptors: descriptors,
        embedding: embedding_values,
        semantic_tags: tags,
        diagnostics: SemanticAnalysisDiagnostics {
            descriptor_confidence,
            semantic_confidence,
            top_tag_margin,
            top_tag_label,
            confidence_components,
            embedding_l2_norm,
            active_embedding_dimensions,
        },
    }
}

fn tag_evidence(
    primary_driver: &'static str,
    primary_value: f32,
    supporting_driver: &'static str,
    supporting_value: f32,
) -> SemanticTagEvidence {
    SemanticTagEvidence {
        primary_driver,
        primary_value,
        supporting_driver,
        supporting_value,
        evidence_strength: normalize_unit((primary_value + supporting_value) * 0.5),
    }
}

fn descriptor_scaled_confidence(base: Confidence, evidence_strength: f32) -> Confidence {
    Confidence::new(base.0 * (0.45 + 0.55 * normalize_unit(evidence_strength)))
}
