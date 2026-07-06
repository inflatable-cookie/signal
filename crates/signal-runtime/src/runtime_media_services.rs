use super::*;

impl SignalRuntime {
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

    /// Reconciles offline stretch artifact plans observed by render/export/freeze.
    pub fn reconcile_offline_stretch_artifact_plans(
        &mut self,
        plans: Vec<RuntimeOfflineStretchArtifactPlanRegistration>,
    ) -> Result<(), RuntimeError> {
        self.offline_stretch_artifact_plans.reconcile_plans(plans);
        Ok(())
    }

    /// Reconciles materialized offline stretch artifacts observed by render/export/freeze.
    pub fn reconcile_offline_stretch_artifact_materializations(
        &mut self,
        artifacts: Vec<RuntimeOfflineStretchArtifactMaterializationRegistration>,
    ) -> Result<(), RuntimeError> {
        self.offline_stretch_artifact_plans
            .reconcile_materialized_artifacts(artifacts);
        Ok(())
    }
}
