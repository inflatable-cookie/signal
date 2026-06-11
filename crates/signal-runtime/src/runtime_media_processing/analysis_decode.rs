use std::{collections::BTreeMap, path::Path};

use signal_analysis::AnalysisStage;
use signal_analysis_character::{CharacterAnalyzer, CharacterAnalyzerConfig};
use signal_analysis_loudness::{LoudnessMeter, LoudnessMeterConfig};
use signal_primitives::AudioBuffer;

use super::media_decode::{decode_runtime_media_asset_with_symphonia, decode_runtime_wav_asset};
use crate::interfaces::{
    RuntimeError, RuntimeErrorKind, RuntimeMediaAnalysisDescriptorState,
    RuntimeMediaAssetRegistration, RuntimeMediaAssetState, RuntimeMediaCharacterDescriptor,
    RuntimeMediaLoudnessDescriptor,
};
use crate::runtime::{RuntimeMediaAnalysisStateModel, RuntimeMediaPipelineStateModel};

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

pub(crate) fn decode_runtime_media_asset(
    media_pipeline: &RuntimeMediaPipelineStateModel,
    asset_id: &str,
    decoded_assets: &mut BTreeMap<String, AudioBuffer>,
) -> Result<AudioBuffer, RuntimeError> {
    if let Some(buffer) = decoded_assets.get(asset_id) {
        return Ok(buffer.clone());
    }
    let asset = media_pipeline.assets.get(asset_id).ok_or_else(|| {
        RuntimeError::new(
            RuntimeErrorKind::InvalidState,
            format!("offline render references unknown media asset `{asset_id}`"),
        )
    })?;
    if asset.state != RuntimeMediaAssetState::Ready {
        return Err(RuntimeError::new(
            RuntimeErrorKind::InvalidState,
            format!(
                "offline render media asset `{asset_id}` is not ready: {:?}",
                asset.state
            ),
        ));
    }
    let cache_path = asset
        .cache_path
        .as_deref()
        .map(Path::new)
        .filter(|path| path.is_file());
    let path = if let Some(path) = cache_path {
        path
    } else {
        let source_path = Path::new(&asset.registration.source_path);
        if source_path.is_file() {
            source_path
        } else {
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                match asset.cache_path.as_deref() {
                    Some(cache_path) => format!(
                        "offline render media asset `{asset_id}` cache missing at {} and source missing at {}",
                        cache_path,
                        source_path.display()
                    ),
                    None => format!(
                        "offline render media asset `{asset_id}` has no cache path and source missing at {}",
                        source_path.display()
                    ),
                },
            ));
        }
    };
    let buffer = if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("wav"))
    {
        decode_runtime_wav_asset(path, asset_id)?
    } else {
        decode_runtime_media_asset_with_symphonia(path, asset_id)?
    };
    decoded_assets.insert(asset_id.to_string(), buffer.clone());
    Ok(buffer)
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
