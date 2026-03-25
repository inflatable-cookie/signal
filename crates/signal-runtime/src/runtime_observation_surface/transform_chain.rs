use super::*;

impl SignalRuntime {
    pub(crate) fn stretch_engine_snapshot(&self) -> RuntimeStretchEngineSnapshot {
        RuntimeStretchEngineSnapshot::from_clip_processing_pipeline(
            &self.clip_processing_pipeline_snapshot(),
        )
    }

    pub(crate) fn marker_analysis_snapshot(&self) -> RuntimeMarkerAnalysisSnapshot {
        RuntimeMarkerAnalysisSnapshot::from_clip_processing_and_media_library(
            &self.clip_processing_pipeline_snapshot(),
            &self.stretch_engine_snapshot(),
            &self.warp_pipeline_snapshot(),
            &self.media_library_service_snapshot(),
            self.config.sample_rate.0,
        )
    }

    pub(crate) fn transform_artifact_snapshot(&self) -> RuntimeTransformArtifactSnapshot {
        RuntimeTransformArtifactSnapshot::from_runtime_transform_state(
            &self.clip_processing_pipeline_snapshot(),
            &self.stretch_engine_snapshot(),
            &self.marker_analysis_snapshot(),
            &self.media_pipeline_snapshot(),
        )
    }

    pub(crate) fn preview_transform_snapshot(&self) -> RuntimePreviewTransformServiceSnapshot {
        RuntimePreviewTransformServiceSnapshot::from_runtime_preview_state(
            &self.clip_processing_pipeline_snapshot(),
            &self.media_service_snapshot(),
            &self.stretch_engine_snapshot(),
            &self.marker_analysis_snapshot(),
            &self.transform_artifact_snapshot(),
        )
    }
}
