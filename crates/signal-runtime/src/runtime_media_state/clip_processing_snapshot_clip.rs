use super::*;

impl RuntimeClipProcessingPipelineStateModel {
    pub(crate) fn snapshot_clip(
        &self,
        registration: &RuntimeClipProcessingRegistration,
        media_pipeline: &RuntimeMediaPipelineStateModel,
        warp_clips: &[RuntimeWarpClipSnapshot],
    ) -> RuntimeClipProcessingSnapshot {
        let fade_in_end_samples = registration
            .start_samples
            .saturating_add(i64::from(registration.fade_in.duration_samples));
        let fade_out_start_samples = registration.start_samples.saturating_add(i64::from(
            registration
                .duration_samples
                .saturating_sub(registration.fade_out.duration_samples),
        ));
        let treatment_stages = self.treatment_stages(registration);
        let _treatment_stages_summary = treatment_stages.clone();
        let (realized_warp_ratio, project_tempo_source, project_tempo_segment_id) =
            self.warp_context(registration, warp_clips);
        let _project_tempo_segment_id_summary = project_tempo_segment_id.clone();
        let (readiness, last_error) = if let Some(error) = self.validate_registration(registration)
        {
            (RuntimeClipProcessingReadiness::Invalid, Some(error))
        } else if let Some(media_asset_id) = registration.media_asset_id.as_deref() {
            match media_pipeline.assets.get(media_asset_id) {
                Some(asset) if asset.state == RuntimeMediaAssetState::Ready => {
                    if registration.warp_mode == RuntimeWarpMode::Off {
                        (RuntimeClipProcessingReadiness::Ready, None)
                    } else if let Some(warp_clip) = warp_clips
                        .iter()
                        .find(|clip| clip.clip_id == registration.clip_id)
                    {
                        match warp_clip.readiness {
                            RuntimeWarpReadiness::Ready | RuntimeWarpReadiness::Bypassed => {
                                (RuntimeClipProcessingReadiness::Ready, None)
                            }
                            RuntimeWarpReadiness::Degraded => (
                                RuntimeClipProcessingReadiness::Invalid,
                                warp_clip
                                    .last_error
                                    .clone()
                                    .or_else(|| Some("warp processing degraded".to_string())),
                            ),
                        }
                    } else {
                        (
                            RuntimeClipProcessingReadiness::PendingWarp,
                            Some("warp pipeline has not realized this clip yet".to_string()),
                        )
                    }
                }
                Some(asset)
                    if matches!(
                        asset.state,
                        RuntimeMediaAssetState::Ingesting
                            | RuntimeMediaAssetState::Conforming
                            | RuntimeMediaAssetState::Rebuilding
                    ) =>
                {
                    (
                        RuntimeClipProcessingReadiness::PendingMedia,
                        Some(format!("media asset not ready: {:?}", asset.state)),
                    )
                }
                Some(asset) => (
                    RuntimeClipProcessingReadiness::Invalid,
                    Some(format!("media asset invalid: {:?}", asset.state)),
                ),
                None => (
                    RuntimeClipProcessingReadiness::PendingMedia,
                    Some(format!(
                        "media asset `{media_asset_id}` not yet available in runtime cache"
                    )),
                ),
            }
        } else if registration.warp_mode != RuntimeWarpMode::Off {
            (
                RuntimeClipProcessingReadiness::Invalid,
                Some("warp-enabled clip has no media asset".to_string()),
            )
        } else {
            (RuntimeClipProcessingReadiness::Ready, None)
        };

        RuntimeClipProcessingSnapshot {
            clip_id: registration.clip_id.clone(),
            media_asset_id: registration.media_asset_id.clone(),
            warp_mode: registration.warp_mode,
            start_samples: registration.start_samples,
            duration_samples: registration.duration_samples,
            fade_in: registration.fade_in.clone(),
            fade_out: registration.fade_out.clone(),
            fade_in_end_samples,
            fade_out_start_samples,
            clip_gain: registration.clip_gain.clone(),
            treatment_stages,
            realized_warp_ratio,
            project_tempo_source,
            project_tempo_segment_id,
            readiness,
            last_error: last_error.clone(),
        }
    }
}
