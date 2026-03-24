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
    let cache_path = asset.cache_path.as_deref().ok_or_else(|| {
        RuntimeError::new(
            RuntimeErrorKind::InvalidState,
            format!("offline render media asset `{asset_id}` has no cache path"),
        )
    })?;
    let path = Path::new(cache_path);
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
        summary: format!(
            "integrated_lufs={:.3} true_peak_dbtp={:.3} loudness_range_lu={:.3} target_offset_lu={:.3} peak_to_loudness_lu={:.3} confidence={:.3}",
            result.integrated_lufs,
            result.true_peak_dbtp,
            result.loudness_range_lu,
            result.dynamics.target_offset_lu,
            result.dynamics.peak_to_loudness_lu,
            result.confidence.0,
        ),
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
        summary: format!(
            "centroid_hz={:.3} rolloff_95_hz={:.3} flatness={:.3} contrast_db={:.3} onset_density={:.3} transient_density={:.3} sustain_ratio={:.3} rms_energy={:.3} dynamic_range={:.3} confidence={:.3}",
            result.spectral_shape.centroid_hz,
            result.spectral_shape.rolloff_95_hz,
            result.spectral_shape.flatness,
            result.spectral_contrast.contrast_db,
            result.temporal.onset_density,
            result.temporal.transient_density,
            result.temporal.sustain_ratio,
            result.dynamics.rms_energy,
            result.dynamics.dynamic_range,
            result.confidence.0,
        ),
    }
}
