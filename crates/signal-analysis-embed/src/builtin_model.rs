use signal_analysis_character::CharacterAnalyzerConfig;

use crate::types::{
    ModelFallbackBehavior, ModelLoadError, SemanticEmbedderConfig, SemanticModelResourceProfile,
    SemanticModelSource, SemanticModelSpec, SemanticModelVersion,
};

mod embedding;
mod semantics;

pub(crate) const BUILTIN_DESCRIPTOR_MODEL_ID: &str = "signal:descriptor-embed:v1";
pub(crate) const BUILTIN_DESCRIPTOR_MODEL_NOTES: &str =
    "Built-in deterministic descriptor projection over shared character packs.";
pub(crate) const EMBEDDING_DIMENSIONS: usize = 8;

#[derive(Clone, Copy, Debug)]
pub(crate) struct BuiltInDescriptorSemanticModel {
    pub(crate) version: SemanticModelVersion,
}

impl BuiltInDescriptorSemanticModel {
    pub(crate) fn resolve(
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

    pub(crate) fn spec(
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
}

pub(crate) fn normalize_unit(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

pub(crate) fn normalize_log_hz(value_hz: f32, low_hz: f32, high_hz: f32) -> f32 {
    if value_hz <= 0.0 || low_hz <= 0.0 || high_hz <= low_hz {
        return 0.0;
    }

    let low = low_hz.log2();
    let high = high_hz.log2();
    normalize_unit((value_hz.log2() - low) / (high - low))
}
