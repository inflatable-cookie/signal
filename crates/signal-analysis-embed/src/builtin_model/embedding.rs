use signal_analysis_character::CharacterAnalysisResult;

use super::{normalize_log_hz, normalize_unit, BuiltInDescriptorSemanticModel};

impl BuiltInDescriptorSemanticModel {
    pub(crate) fn embedding_from_descriptors(
        &self,
        descriptors: &CharacterAnalysisResult,
    ) -> Vec<f32> {
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
}
