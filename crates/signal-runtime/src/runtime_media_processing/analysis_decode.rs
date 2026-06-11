use std::path::Path;

use signal_analysis::AnalysisStage;
use signal_analysis_character::{CharacterAnalyzer, CharacterAnalyzerConfig};
use signal_analysis_loudness::{LoudnessMeter, LoudnessMeterConfig};

use super::media_decode::{decode_runtime_media_asset_with_symphonia, decode_runtime_wav_asset};
use crate::interfaces::{
    RuntimeMediaAnalysisDescriptorState, RuntimeMediaAssetRegistration,
    RuntimeMediaCharacterDescriptor, RuntimeMediaLoudnessDescriptor,
};
use crate::runtime::RuntimeMediaAnalysisStateModel;

pub(crate) fn analyze_runtime_media_asset(
    cache_path: &Path,
    registration: &RuntimeMediaAssetRegistration,
) -> Result<RuntimeMediaAnalysisStateModel, String> {
    let decoded = if cache_path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
    {
        decode_runtime_wav_asset(cache_path, &registration.asset_id)
            .map_err(|error| error.message)?
    } else {
        decode_runtime_media_asset_with_symphonia(cache_path, &registration.asset_id)
            .map_err(|error| error.message)?
    };
    let mut loudness_meter = LoudnessMeter::new(LoudnessMeterConfig::default());
    let mut character_analyzer = CharacterAnalyzer::new(CharacterAnalyzerConfig::default());
    let loudness = loudness_descriptor_from_result(loudness_meter.analyze(&decoded));
    let character = character_descriptor_from_result(character_analyzer.analyze(&decoded));
    Ok(RuntimeMediaAnalysisStateModel {
        descriptor_state: RuntimeMediaAnalysisDescriptorState::Ready,
        loudness: Some(loudness),
        character: Some(character),
        last_error: None,
    })
}

fn loudness_descriptor_from_result(
    result: signal_analysis_loudness::LoudnessAnalysisResult,
) -> RuntimeMediaLoudnessDescriptor {
    RuntimeMediaLoudnessDescriptor {
        integrated_lufs: result.integrated_lufs,
        loudness_range_lu: result.loudness_range_lu,
        true_peak_dbtp: result.true_peak_dbtp,
        target_offset_lu: result.dynamics.target_offset_lu,
        peak_to_loudness_lu: result.dynamics.peak_to_loudness_lu,
        confidence: result.confidence.0,
    }
}

fn character_descriptor_from_result(
    result: signal_analysis_character::CharacterAnalysisResult,
) -> RuntimeMediaCharacterDescriptor {
    RuntimeMediaCharacterDescriptor {
        centroid_hz: result.spectral_shape.centroid_hz,
        rolloff_95_hz: result.spectral_shape.rolloff_95_hz,
        flatness: result.spectral_shape.flatness,
        contrast_db: result.spectral_contrast.contrast_db,
        onset_density: result.temporal.onset_density,
        transient_density: result.temporal.transient_density,
        sustain_ratio: result.temporal.sustain_ratio,
        rms_energy: result.dynamics.rms_energy,
        dynamic_range: result.dynamics.dynamic_range,
        confidence: result.confidence.0,
    }
}
