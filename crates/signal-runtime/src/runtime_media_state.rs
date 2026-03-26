use super::*;

#[path = "runtime_media_state/clip_processing_render.rs"]
mod clip_processing_render;
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
#[path = "runtime_media_state/metering_capture.rs"]
mod metering_capture;
#[path = "runtime_media_state/metering_contract.rs"]
mod metering_contract;

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeMeteringWindowBlock {
    pub(crate) mean_square: f64,
    pub(crate) sample_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeMeteringStateModel {
    pub(crate) snapshot: RuntimeMeteringSnapshot,
    pub(crate) momentary_blocks: VecDeque<RuntimeMeteringWindowBlock>,
    pub(crate) short_term_blocks: VecDeque<RuntimeMeteringWindowBlock>,
    pub(crate) momentary_sum: f64,
    pub(crate) short_term_sum: f64,
    pub(crate) momentary_sample_count: usize,
    pub(crate) short_term_sample_count: usize,
    pub(crate) integrated_sum: f64,
    pub(crate) integrated_sample_count: u64,
    pub(crate) clipped_sample_count: u64,
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
