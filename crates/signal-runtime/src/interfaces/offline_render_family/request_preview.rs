use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderTargetKind {
    MainMix,
    TrackLane,
    BusGroup,
    ConsoleGroup,
    SendReturn,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderStemTarget {
    pub stem_id: String,
    pub target_kind: RuntimeOfflineRenderTargetKind,
    pub target_id: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineFreezeArtifactRequest {
    pub artifact_id: String,
    pub source_stem_id: String,
    pub recall_selection: RuntimePluginRecallHandoffSelection,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderRequest {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub duration_samples: u32,
    pub export_sample_rate_hz: u32,
    pub include_main_mix: bool,
    pub artifact_root_path: Option<String>,
    pub stem_targets: Vec<RuntimeOfflineRenderStemTarget>,
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderStemPreview {
    pub stem_id: String,
    pub target_kind: RuntimeOfflineRenderTargetKind,
    pub target_id: Option<String>,
    pub resolved_node_ids: Vec<String>,
    pub resolved_output_bus_ids: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineFreezeArtifactPreview {
    pub artifact_id: String,
    pub source_stem_id: String,
    pub recall_stage_count: usize,
    pub recall_stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
    pub recall_states: Vec<RuntimePluginRecallState>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderChainDependencyPreview {
    pub chain_count: usize,
    pub stage_count: usize,
    pub pending_render_stage_count: usize,
    pub settling_stage_count: usize,
    pub compensated_stage_count: usize,
    pub degraded_stage_count: usize,
    pub bypassed_stage_count: usize,
    pub missing_binding_stage_count: usize,
    pub total_planned_latency_samples: u32,
    pub total_realized_latency_samples: u32,
    pub total_tail_samples: u32,
    pub recall_stage_count: usize,
    pub unbound_recall_stage_count: usize,
    pub cold_recall_stage_count: usize,
    pub warm_recall_stage_count: usize,
    pub recovered_recall_stage_count: usize,
    pub unavailable_recall_stage_count: usize,
    pub secondary_input_count: usize,
    pub required_secondary_input_count: usize,
    pub optional_secondary_input_count: usize,
    pub disabled_secondary_input_count: usize,
    pub terminal_fallback_secondary_input_count: usize,
    pub bus_connection_count: usize,
    pub auxiliary_path_count: usize,
    pub complex_io_stage_count: usize,
    pub multi_output_instrument_stage_count: usize,
    pub bus_capable_fx_stage_count: usize,
    pub sidechain_capable_fx_stage_count: usize,
    pub spatial_stage_count: usize,
    pub active_spatial_stage_count: usize,
    pub bypassed_spatial_stage_count: usize,
    pub fallback_spatial_stage_count: usize,
    pub surround_bed_spatial_stage_count: usize,
    pub object_aware_spatial_stage_count: usize,
    pub expanded_fallback_spatial_stage_count: usize,
    pub immersive_spatial_stage_count: usize,
    pub room_policy_aware_spatial_stage_count: usize,
    pub fallback_room_policy_spatial_stage_count: usize,
    pub deployment_spatial_stage_count: usize,
    pub folded_down_spatial_stage_count: usize,
    pub fallback_monitoring_scene_spatial_stage_count: usize,
    pub renderer_capability_spatial_stage_count: usize,
    pub negotiated_renderer_spatial_stage_count: usize,
    pub immersive_export_spatial_stage_count: usize,
    pub fallback_immersive_export_spatial_stage_count: usize,
    pub secondary_inputs: Vec<RuntimeSecondaryInputRouteSummary>,
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    pub complex_io_stages: Vec<RuntimeOfflineRenderComplexIoStageSummary>,
    pub spatial_stages: Vec<RuntimeOfflineRenderSpatialStageSummary>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderComplexIoStageSummary {
    pub chain_id: String,
    pub node_id: String,
    pub stage_index: usize,
    pub plugin_type_id: Option<String>,
    pub topology: RuntimePluginComplexIoSummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderSpatialStageSummary {
    pub chain_id: String,
    pub node_id: String,
    pub stage_index: usize,
    pub plugin_type_id: Option<String>,
    pub spatial: RuntimeSpatialExecutionSummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderContractPreview {
    pub request_id: String,
    pub timeline_start_samples: i64,
    pub timeline_end_samples: i64,
    pub duration_samples: u32,
    pub export_sample_rate_hz: u32,
    pub include_main_mix: bool,
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub stretch_engine_snapshot: RuntimeStretchEngineSnapshot,
    pub preview_transform_snapshot: RuntimePreviewTransformServiceSnapshot,
    pub transform_artifact_snapshot: RuntimeTransformArtifactSnapshot,
    pub stem_count: usize,
    pub freeze_artifact_count: usize,
    pub resolved_tempo_bpm: f64,
    pub resolved_tempo_source: RuntimeTempoSource,
    pub chain_contract: RuntimeOfflineRenderChainDependencyPreview,
    pub stem_targets: Vec<RuntimeOfflineRenderStemPreview>,
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactPreview>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderStemResult {
    pub stem_id: String,
    pub target_kind: RuntimeOfflineRenderTargetKind,
    pub target_id: Option<String>,
    pub output: AudioBuffer,
    pub peak_level: f32,
    pub rms_level: f32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineFreezeArtifactResult {
    pub artifact_id: String,
    pub source_stem_id: String,
    pub recall_stage_count: usize,
    pub recall_stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
    pub recall_states: Vec<RuntimePluginRecallState>,
    pub output: AudioBuffer,
    pub peak_level: f32,
    pub rms_level: f32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderResult {
    pub request_id: String,
    pub runtime_frame_count: usize,
    pub rendered_frame_count: usize,
    pub block_count: usize,
    pub export_sample_rate_hz: u32,
    pub main_mix: Option<AudioBuffer>,
    pub main_mix_peak_level: Option<f32>,
    pub main_mix_rms_level: Option<f32>,
    pub stems: Vec<RuntimeOfflineRenderStemResult>,
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactResult>,
    pub manifest: RuntimeOfflineRenderManifest,
    pub plugin_execution_boundary: RuntimeOfflinePluginExecutionBoundary,
    pub contract_preview: RuntimeOfflineRenderContractPreview,
    pub summary: String,
}
