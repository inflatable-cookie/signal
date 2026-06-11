use super::*;

impl SignalRuntime {
    /// Renders a clip processing buffer at the current runtime sample rate.
    pub fn render_clip_processing_buffer(
        &self,
        request: RuntimeClipRenderRequest,
    ) -> Result<RuntimeClipRenderResult, RuntimeError> {
        if request.clip_id.is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "clip processing render requests require a non-empty clip id",
            ));
        }
        if request.buffer.sample_rate() != self.config.sample_rate {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "clip processing render buffer sample rate {} does not match runtime sample rate {}",
                    request.buffer.sample_rate().0,
                    self.config.sample_rate.0,
                ),
            ));
        }
        self.render_clip_processing_buffer_with_resolved_tempo(
            request,
            &self.current_resolved_tempo(),
        )
    }

    /// Starts a new recording capture session with the given request parameters.
    pub fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError> {
        self.recording_capture.start_capture(
            request,
            self.config.sample_rate.0,
            self.control.configured,
            &self.readiness,
        )
    }

    /// Finalises the active recording capture session and returns a commit receipt.
    pub fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError> {
        self.recording_capture.finish_capture()
    }

    /// Cancels the active recording capture session without committing any data.
    pub fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError> {
        self.recording_capture.cancel_capture()
    }

    /// Reconciles the registered media asset list with the current pipeline state.
    pub fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError> {
        self.media_pipeline.reconcile_assets(assets)
    }

    /// Starts media preview playback for the given asset.
    pub fn start_media_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError> {
        self.media_pipeline.start_preview(asset_id)
    }

    /// Stops the active media preview session.
    pub fn stop_media_preview(&mut self) -> Result<(), RuntimeError> {
        self.media_pipeline.stop_preview();
        Ok(())
    }

    /// Reconciles the registered warp clip list with the current pipeline state.
    pub fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError> {
        self.warp_pipeline.reconcile_clips(clips);
        Ok(())
    }

    /// Reconciles the registered clip processing clip list with the current pipeline state.
    pub fn reconcile_clip_processing_clips(
        &mut self,
        clips: Vec<RuntimeClipProcessingRegistration>,
    ) -> Result<(), RuntimeError> {
        self.clip_processing_pipeline.reconcile_clips(clips);
        Ok(())
    }

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
