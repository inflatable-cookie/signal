use super::*;

/// Target topology node kind for an offline render stem.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeOfflineRenderTargetKind {
    /// The main mix output.
    MainMix,
    /// A track lane output.
    TrackLane,
    /// A bus group output.
    BusGroup,
    /// A console group output.
    ConsoleGroup,
    /// A send-return pair output.
    SendReturn,
}

/// Stem target specification: which topology node to render as a stem.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderStemTarget {
    /// Unique identifier for this stem.
    pub stem_id: String,
    /// Kind of topology node to render as this stem.
    pub target_kind: RuntimeOfflineRenderTargetKind,
    /// ID of the specific topology node, if applicable.
    pub target_id: Option<String>,
}

/// Request to capture a freeze artifact from a specific recall selection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineFreezeArtifactRequest {
    /// Unique identifier for this freeze artifact.
    pub artifact_id: String,
    /// ID of the stem whose plugin chain will be captured.
    pub source_stem_id: String,
    /// Plugin recall handoff selection specifying which stages to capture.
    pub recall_selection: RuntimePluginRecallHandoffSelection,
}

/// Full offline render request: timeline range, sample rates, stem targets,
/// and freeze artifact requests.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeOfflineRenderRequest {
    /// Unique identifier for this render request.
    pub request_id: String,
    /// Timeline start position in samples.
    pub timeline_start_samples: i64,
    /// Duration of the render in samples.
    pub duration_samples: u32,
    /// Sample rate for the exported audio in Hz.
    pub export_sample_rate_hz: u32,
    /// Whether to include the main mix output as an artifact.
    pub include_main_mix: bool,
    /// Root directory path for output artifacts, if writing to disk.
    pub artifact_root_path: Option<String>,
    /// Stem targets to render alongside the main mix.
    pub stem_targets: Vec<RuntimeOfflineRenderStemTarget>,
    /// Freeze artifact capture requests.
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactRequest>,
}

/// Preview of how a stem target resolves to graph node IDs and bus IDs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderStemPreview {
    /// Unique identifier for this stem.
    pub stem_id: String,
    /// Kind of topology node targeted by this stem.
    pub target_kind: RuntimeOfflineRenderTargetKind,
    /// ID of the specific topology node, if applicable.
    pub target_id: Option<String>,
    /// Graph node IDs resolved for this stem target.
    pub resolved_node_ids: Vec<String>,
    /// Output bus IDs resolved for this stem target.
    pub resolved_output_bus_ids: Vec<String>,
}

/// Preview of which recall stages will be captured in a freeze artifact render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineFreezeArtifactPreview {
    /// Unique identifier for this freeze artifact.
    pub artifact_id: String,
    /// ID of the stem from which plugin recall stages are sourced.
    pub source_stem_id: String,
    /// Number of plugin recall stages to be captured.
    pub recall_stage_count: usize,
    /// IDs of the plugin recall handoff stages to be captured.
    pub recall_stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
    /// Recall state for each stage to be captured.
    pub recall_states: Vec<RuntimePluginRecallState>,
}

/// Dependency preview for the plugin chain involved in an offline render:
/// all stage counts, latency sums, recall state breakdowns, and spatial details.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderChainDependencyPreview {
    /// Number of plugin chains involved in the render.
    pub chain_count: usize,
    /// Total number of plugin chain stages.
    pub stage_count: usize,
    /// Number of stages pending render readiness.
    pub pending_render_stage_count: usize,
    /// Number of stages in the settling state.
    pub settling_stage_count: usize,
    /// Number of stages with latency compensation active.
    pub compensated_stage_count: usize,
    /// Number of stages in a degraded state.
    pub degraded_stage_count: usize,
    /// Number of stages that are bypassed.
    pub bypassed_stage_count: usize,
    /// Number of stages missing a sandbox binding.
    pub missing_binding_stage_count: usize,
    /// Total planned latency across all stages in samples.
    pub total_planned_latency_samples: u32,
    /// Total realized latency across all stages in samples.
    pub total_realized_latency_samples: u32,
    /// Total plugin chain tail length across all stages in samples.
    pub total_tail_samples: u32,
    /// Number of stages with plugin recall state.
    pub recall_stage_count: usize,
    /// Number of recall stages with no sandbox binding.
    pub unbound_recall_stage_count: usize,
    /// Number of recall stages in the cold (no warm state) condition.
    pub cold_recall_stage_count: usize,
    /// Number of recall stages with warm (ready) state.
    pub warm_recall_stage_count: usize,
    /// Number of recall stages successfully recovered.
    pub recovered_recall_stage_count: usize,
    /// Number of recall stages that are unavailable.
    pub unavailable_recall_stage_count: usize,
    /// Total number of secondary (sidechain) inputs.
    pub secondary_input_count: usize,
    /// Number of required secondary inputs.
    pub required_secondary_input_count: usize,
    /// Number of optional secondary inputs.
    pub optional_secondary_input_count: usize,
    /// Number of secondary inputs that are disabled.
    pub disabled_secondary_input_count: usize,
    /// Number of secondary inputs using a terminal fallback route.
    pub terminal_fallback_secondary_input_count: usize,
    /// Number of bus connections in the chain topology.
    pub bus_connection_count: usize,
    /// Number of auxiliary paths in the chain topology.
    pub auxiliary_path_count: usize,
    /// Number of stages with complex I/O topology.
    pub complex_io_stage_count: usize,
    /// Number of multi-output instrument stages.
    pub multi_output_instrument_stage_count: usize,
    /// Number of bus-capable FX stages.
    pub bus_capable_fx_stage_count: usize,
    /// Number of sidechain-capable FX stages.
    pub sidechain_capable_fx_stage_count: usize,
    /// Total number of spatial processing stages.
    pub spatial_stage_count: usize,
    /// Number of spatial stages actively processing.
    pub active_spatial_stage_count: usize,
    /// Number of spatial stages that are bypassed.
    pub bypassed_spatial_stage_count: usize,
    /// Number of spatial stages using a fallback path.
    pub fallback_spatial_stage_count: usize,
    /// Number of surround-bed spatial stages.
    pub surround_bed_spatial_stage_count: usize,
    /// Number of object-aware spatial stages.
    pub object_aware_spatial_stage_count: usize,
    /// Number of expanded-fallback spatial stages.
    pub expanded_fallback_spatial_stage_count: usize,
    /// Number of immersive spatial stages.
    pub immersive_spatial_stage_count: usize,
    /// Number of room-policy-aware spatial stages.
    pub room_policy_aware_spatial_stage_count: usize,
    /// Number of spatial stages using a fallback room policy.
    pub fallback_room_policy_spatial_stage_count: usize,
    /// Number of deployment-configured spatial stages.
    pub deployment_spatial_stage_count: usize,
    /// Number of spatial stages folding down to a lower channel count.
    pub folded_down_spatial_stage_count: usize,
    /// Number of spatial stages using a fallback monitoring scene.
    pub fallback_monitoring_scene_spatial_stage_count: usize,
    /// Number of spatial stages constrained by renderer capabilities.
    pub renderer_capability_spatial_stage_count: usize,
    /// Number of spatial stages with a negotiated renderer.
    pub negotiated_renderer_spatial_stage_count: usize,
    /// Number of spatial stages targeting immersive export.
    pub immersive_export_spatial_stage_count: usize,
    /// Number of spatial stages using a fallback immersive export path.
    pub fallback_immersive_export_spatial_stage_count: usize,
    /// Secondary input route summaries.
    pub secondary_inputs: Vec<RuntimeSecondaryInputRouteSummary>,
    /// Bus connection summaries.
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    /// Auxiliary path summaries.
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    /// Complex-I/O stage summaries.
    pub complex_io_stages: Vec<RuntimeOfflineRenderComplexIoStageSummary>,
    /// Spatial stage summaries.
    pub spatial_stages: Vec<RuntimeOfflineRenderSpatialStageSummary>,
}

/// Summary of a complex-I/O plugin stage involved in an offline render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderComplexIoStageSummary {
    /// ID of the plugin chain this stage belongs to.
    pub chain_id: String,
    /// ID of the graph node for this stage.
    pub node_id: String,
    /// Zero-based index of this stage within its chain.
    pub stage_index: usize,
    /// Plugin type identifier, if known.
    pub plugin_type_id: Option<String>,
    /// Complex I/O topology summary for this stage.
    pub topology: RuntimePluginComplexIoSummary,
}

/// Summary of a spatial plugin stage involved in an offline render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeOfflineRenderSpatialStageSummary {
    /// ID of the plugin chain this stage belongs to.
    pub chain_id: String,
    /// ID of the graph node for this stage.
    pub node_id: String,
    /// Zero-based index of this stage within its chain.
    pub stage_index: usize,
    /// Plugin type identifier, if known.
    pub plugin_type_id: Option<String>,
    /// Spatial execution summary for this stage.
    pub spatial: RuntimeSpatialExecutionSummary,
}

/// Full contract preview for a pending offline render request: resolved tempo,
/// clip/stretch state, chain dependencies, stem targets, and freeze artifacts.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderContractPreview {
    /// ID of the render request this preview belongs to.
    pub request_id: String,
    /// Timeline start position in samples.
    pub timeline_start_samples: i64,
    /// Timeline end position in samples.
    pub timeline_end_samples: i64,
    /// Duration of the render in samples.
    pub duration_samples: u32,
    /// Sample rate for the exported audio in Hz.
    pub export_sample_rate_hz: u32,
    /// Whether the main mix is included in this render.
    pub include_main_mix: bool,
    /// Total number of clips in the render range.
    pub clip_count: usize,
    /// Number of clips in the ready state.
    pub ready_clip_count: usize,
    /// Stretch engine snapshot at the time of the preview.
    pub stretch_engine_snapshot: RuntimeStretchEngineSnapshot,
    /// Preview transform service snapshot at the time of the preview.
    pub preview_transform_snapshot: RuntimePreviewTransformServiceSnapshot,
    /// Transform artifact snapshot at the time of the preview.
    pub transform_artifact_snapshot: RuntimeTransformArtifactSnapshot,
    /// Number of stems to render.
    pub stem_count: usize,
    /// Number of freeze artifacts to render.
    pub freeze_artifact_count: usize,
    /// Resolved project tempo for the render range in BPM.
    pub resolved_tempo_bpm: f64,
    /// Source from which `resolved_tempo_bpm` was derived.
    pub resolved_tempo_source: RuntimeTempoSource,
    /// Plugin chain dependency preview for this render.
    pub chain_contract: RuntimeOfflineRenderChainDependencyPreview,
    /// Per-stem target previews.
    pub stem_targets: Vec<RuntimeOfflineRenderStemPreview>,
    /// Per-freeze-artifact previews.
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactPreview>,
}

/// Rendered audio result for a single stem including peak/RMS levels.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderStemResult {
    /// Unique identifier for this stem.
    pub stem_id: String,
    /// Kind of topology node this stem was rendered from.
    pub target_kind: RuntimeOfflineRenderTargetKind,
    /// ID of the specific topology node, if applicable.
    pub target_id: Option<String>,
    /// Rendered audio output for this stem.
    pub output: AudioBuffer,
    /// Peak level of the rendered stem in linear scale.
    pub peak_level: f32,
    /// RMS level of the rendered stem in linear scale.
    pub rms_level: f32,
}

/// Rendered audio result for a single freeze artifact including peak/RMS levels.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineFreezeArtifactResult {
    /// Unique identifier for this freeze artifact.
    pub artifact_id: String,
    /// ID of the stem from which this artifact was captured.
    pub source_stem_id: String,
    /// Number of recall stages captured in this artifact.
    pub recall_stage_count: usize,
    /// IDs of the plugin recall handoff stages captured.
    pub recall_stage_ids: Vec<RuntimePluginRecallHandoffStageId>,
    /// Recall state for each captured stage.
    pub recall_states: Vec<RuntimePluginRecallState>,
    /// Rendered audio output for this freeze artifact.
    pub output: AudioBuffer,
    /// Peak level of the rendered artifact in linear scale.
    pub peak_level: f32,
    /// RMS level of the rendered artifact in linear scale.
    pub rms_level: f32,
}

/// Complete result of an offline render job: frame counts, main mix,
/// stems, freeze artifacts, manifest, plugin boundaries, and contract preview.
#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeOfflineRenderResult {
    /// ID of the render request this result belongs to.
    pub request_id: String,
    /// Total number of frames processed at the runtime sample rate.
    pub runtime_frame_count: usize,
    /// Total number of frames rendered at the export sample rate.
    pub rendered_frame_count: usize,
    /// Total number of audio blocks processed.
    pub block_count: usize,
    /// Sample rate of the exported audio in Hz.
    pub export_sample_rate_hz: u32,
    /// Main mix audio buffer, if the main mix was rendered.
    pub main_mix: Option<AudioBuffer>,
    /// Peak level of the main mix in linear scale, if rendered.
    pub main_mix_peak_level: Option<f32>,
    /// RMS level of the main mix in linear scale, if rendered.
    pub main_mix_rms_level: Option<f32>,
    /// Rendered stem results.
    pub stems: Vec<RuntimeOfflineRenderStemResult>,
    /// Rendered freeze artifact results.
    pub freeze_artifacts: Vec<RuntimeOfflineFreezeArtifactResult>,
    /// Artifact manifest for this render.
    pub manifest: RuntimeOfflineRenderManifest,
    /// Plugin execution boundary describing which stages ran where.
    pub plugin_execution_boundary: RuntimeOfflinePluginExecutionBoundary,
    /// Contract preview resolved at render time.
    pub contract_preview: RuntimeOfflineRenderContractPreview,
}
