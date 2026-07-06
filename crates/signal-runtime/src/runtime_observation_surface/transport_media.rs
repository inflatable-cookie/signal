use super::*;

impl SignalRuntime {
    pub(crate) fn recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot {
        self.recording_capture
            .snapshot(self.control.configured, &self.readiness)
    }

    pub(crate) fn media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot {
        self.media_pipeline.snapshot()
    }

    pub(crate) fn media_service_snapshot(&self) -> RuntimeMediaServiceSnapshot {
        self.media_pipeline.service_snapshot()
    }

    pub(crate) fn media_library_service_snapshot(&self) -> RuntimeMediaLibraryServiceSnapshot {
        self.media_pipeline.library_service_snapshot()
    }

    pub(crate) fn offline_stretch_artifact_plan_snapshot(
        &self,
    ) -> RuntimeOfflineStretchArtifactPlanSnapshotSet {
        self.offline_stretch_artifact_plans.snapshot()
    }
}
