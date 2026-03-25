use super::*;

impl SignalRuntime {
    pub(crate) fn render_clip_processing_buffer_with_resolved_tempo(
        &self,
        request: RuntimeClipRenderRequest,
        resolved_tempo: &RuntimeResolvedTempo,
    ) -> Result<RuntimeClipRenderResult, RuntimeError> {
        let preview_transform_snapshot = self.preview_transform_snapshot();
        let transform_artifact_snapshot = self.transform_artifact_snapshot();
        let transform_artifact_clip = transform_artifact_snapshot
            .clips
            .iter()
            .find(|clip| clip.clip_id == request.clip_id)
            .cloned()
            .unwrap_or_else(|| RuntimeTransformArtifactClipSnapshot {
                clip_id: request.clip_id.clone(),
                media_asset_id: None,
                artifact_identity: format!("artifact:missing:{}", request.clip_id),
                readiness: RuntimeTransformArtifactReadiness::Unsupported,
                invalidation_state: RuntimeTransformArtifactInvalidationState::None,
                reuse_state: RuntimeTransformArtifactReuseState::Unavailable,
                cached_media_ready: false,
                stretch_engine_class: RuntimeStretchEngineClass::Disabled,
                stretch_readiness: RuntimeStretchReadiness::Disabled,
                marker_analysis_readiness: RuntimeMarkerAnalysisReadiness::Unsupported,
                summary: format!(
                    "clip={} readiness=Unsupported invalidation=None reuse=Unavailable cached_media_ready=false stretch=Disabled/Disabled analysis=Unsupported",
                    request.clip_id
                ),
            });
        let preview_transform_clip = preview_transform_snapshot
            .clips
            .iter()
            .find(|clip| clip.clip_id == request.clip_id)
            .cloned()
            .unwrap_or_else(|| RuntimePreviewTransformClipSnapshot {
                clip_id: request.clip_id.clone(),
                media_asset_id: None,
                service_class: RuntimePreviewTransformServiceClass::Unavailable,
                readiness: RuntimePreviewTransformReadiness::Unsupported,
                degraded_state: RuntimePreviewTransformDegradedState::UnsupportedScope,
                fallback_kind: RuntimePreviewTransformFallbackKind::OfflineOnly,
                artifact_reuse_state: RuntimeTransformArtifactReuseState::Unavailable,
                audition_active: false,
                scrub_supported: false,
                summary: format!(
                    "clip={} class=Unavailable readiness=Unsupported degraded=UnsupportedScope fallback=OfflineOnly artifact_reuse=Unavailable audition_active=false scrub_supported=false",
                    request.clip_id
                ),
            });
        self.clip_processing_pipeline.render_clip(
            request,
            &self.media_pipeline,
            &self.warp_pipeline,
            resolved_tempo,
            transform_artifact_clip,
            preview_transform_clip,
        )
    }
}
