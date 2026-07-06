use super::*;

#[path = "runtime_media_state/clip_processing_snapshot_clip.rs"]
mod clip_processing_snapshot_clip;
#[path = "runtime_media_state/clip_processing_surface.rs"]
mod clip_processing_surface;
#[path = "runtime_media_state/media_library_snapshot.rs"]
mod media_library_snapshot;
#[path = "runtime_media_state/media_pipeline_reconcile.rs"]
mod media_pipeline_reconcile;
#[path = "runtime_media_state/media_pipeline_snapshots.rs"]
mod media_pipeline_snapshots;
#[path = "runtime_media_state/offline_stretch_artifact_snapshot.rs"]
mod offline_stretch_artifact_snapshot;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeMediaPipelineStateModel {
    pub(crate) policy: RuntimeMediaPipelinePolicy,
    pub(crate) assets: BTreeMap<String, RuntimeMediaPipelineAsset>,
    pub(crate) previewing_asset_id: Option<String>,
    pub(crate) last_preview_error: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeClipProcessingPipelineStateModel {
    pub(crate) clips: BTreeMap<String, RuntimeClipProcessingRegistration>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeOfflineStretchArtifactPlanStateModel {
    pub(crate) plans: BTreeMap<String, RuntimeOfflineStretchArtifactPlanRegistration>,
    pub(crate) materialized_artifacts:
        BTreeMap<String, RuntimeOfflineStretchArtifactMaterializationRegistration>,
    pub(crate) cache_decisions:
        BTreeMap<String, RuntimeOfflineStretchArtifactCacheDecisionRegistration>,
}

pub(crate) fn media_family_state(
    descriptor_state: RuntimeMediaAnalysisDescriptorState,
    available: bool,
) -> RuntimeMediaAnalysisFamilyState {
    if available && descriptor_state == RuntimeMediaAnalysisDescriptorState::Ready {
        RuntimeMediaAnalysisFamilyState::Ready
    } else if matches!(
        descriptor_state,
        RuntimeMediaAnalysisDescriptorState::Unavailable
    ) {
        RuntimeMediaAnalysisFamilyState::Unavailable
    } else {
        RuntimeMediaAnalysisFamilyState::Deferred
    }
}
