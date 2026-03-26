use super::*;

#[path = "runtime_tempo_warp_state/tempo_map.rs"]
mod tempo_map;
#[path = "runtime_tempo_warp_state/warp_pipeline.rs"]
mod warp_pipeline;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeTempoMapStateModel {
    pub(crate) projection: Option<RuntimeTempoMapProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RuntimeResolvedTempo {
    pub(crate) tempo_bpm: f64,
    pub(crate) source: RuntimeTempoSource,
    pub(crate) active_segment_id: Option<String>,
    pub(crate) active_segment_index: Option<usize>,
    pub(crate) next_segment_start_samples: Option<i64>,
    pub(crate) timeline_position_samples: Option<i64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct RuntimeWarpPipelineStateModel {
    pub(crate) clips: BTreeMap<String, RuntimeWarpClipRegistration>,
}
