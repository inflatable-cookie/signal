//! Typed runtime-host interfaces for embedded Signal assemblies.
mod clip_analysis_family;
mod device_linux_json_family;
mod engine_block_surface_family;
mod engine_lifecycle_family;
mod event_recorder_family;
mod event_transport_family;
mod execution_metering_surface_family;
mod external_io_family;
mod fault_interruption_family;
mod host_observation_family;
#[path = "interfaces_host_platform_family.rs"]
mod interfaces_host_platform_family;
mod interfaces_json_family;
#[path = "interfaces_offline_contract_family.rs"]
mod interfaces_offline_contract_family;
mod media_clip_family;
mod media_surface_family;
mod observation_receipt_family;
mod observation_render_family;
mod offline_render_family;
mod plugin_chain_render_family;
mod plugin_discovery_family;
mod plugin_recall_family;
mod plugin_runtime_surface_family;
mod preview_transform_family;
mod receipt_surface_family;
mod runtime_continuity_json_family;
mod scheduler_surface_family;
mod spatial_topology_json_family;
pub(crate) use clip_analysis_family::*;
use device_linux_json_family::{
    format_runtime_advanced_hardware_snapshot_multiline,
    format_runtime_control_surface_snapshot_multiline,
    format_runtime_device_supervision_snapshot_compact,
    format_runtime_device_supervision_snapshot_multiline,
    format_runtime_external_midi_snapshot_compact, format_runtime_external_midi_snapshot_multiline,
    format_runtime_jack_coordination_snapshot_compact,
    format_runtime_jack_coordination_snapshot_multiline,
    format_runtime_linux_backend_session_snapshot_compact,
    format_runtime_linux_backend_session_snapshot_multiline,
    format_runtime_pipewire_alsa_parity_snapshot_compact,
    format_runtime_pipewire_alsa_parity_snapshot_multiline,
    json_runtime_advanced_hardware_snapshot, json_runtime_control_surface_snapshot,
    json_runtime_external_midi_snapshot, json_runtime_jack_coordination_snapshot,
    json_runtime_linux_backend_session_snapshot, json_runtime_pipewire_alsa_parity_snapshot,
};
use engine_block_surface_family::json_runtime_engine_block_snapshot;
pub use engine_lifecycle_family::*;
pub use event_recorder_family::*;
pub use event_transport_family::*;
use event_transport_family::{
    format_runtime_automation_snapshot_compact, format_runtime_deferred_service_receipt_compact,
    format_runtime_engine_transport_compact, format_runtime_plugin_event_snapshot_compact,
    format_runtime_transport_timeline_compact,
};
use execution_metering_surface_family::{
    format_runtime_execution_topology_summary_compact,
    format_runtime_execution_topology_summary_multiline, format_runtime_metering_snapshot_compact,
    format_runtime_metering_snapshot_multiline, json_runtime_metering_snapshot,
    json_runtime_planning_group_order, json_runtime_scheduler_topology_summary,
};
pub use external_io_family::*;
use external_io_family::{
    format_runtime_external_io_snapshot_compact, format_runtime_external_io_snapshot_multiline,
    json_runtime_external_io_snapshot,
};
pub use fault_interruption_family::*;
pub use host_observation_family::*;
pub use interfaces_host_platform_family::*;
use interfaces_json_family::{
    format_runtime_scheduler_snapshot_compact, format_runtime_scheduler_snapshot_multiline,
    format_scheduler_topology_compact, format_scheduler_topology_multiline, json_escape_string,
    json_graph_execution_context, json_option_f32, json_option_f64, json_option_i64,
    json_option_string, json_option_u32, json_option_u64, json_option_usize,
    json_plugin_feature_vec, json_plugin_format_vec, json_plugin_io_layout,
    json_plugin_lifecycle_contract, json_plugin_processing_contract, json_plugin_state_contract,
    json_runtime_auxiliary_path_summary_vec, json_runtime_bus_connection_summary_vec,
    json_runtime_bus_intent, json_runtime_execution_lane_order,
    json_runtime_meter_source_snapshot_vec, json_runtime_multichannel_io_summary,
    json_runtime_multichannel_layout_summary, json_runtime_secondary_input_route_summary,
    json_runtime_secondary_input_route_summary_vec,
    json_runtime_worker_lane_instrumentation_summaries, json_string, json_string_vec, json_u64_vec,
};
pub use media_clip_family::*;
use media_surface_family::{
    format_runtime_media_library_service_snapshot_compact,
    format_runtime_media_library_service_snapshot_multiline,
    format_runtime_media_pipeline_snapshot_compact,
    format_runtime_media_pipeline_snapshot_multiline,
    format_runtime_media_service_snapshot_compact, format_runtime_media_service_snapshot_multiline,
    json_runtime_media_library_service_snapshot, json_runtime_media_pipeline_snapshot,
    json_runtime_media_service_snapshot,
};
use observation_render_family::{
    render_runtime_observation_report_compact, render_runtime_supervisor_report_json,
};
pub use offline_render_family::*;
use plugin_chain_render_family::{
    format_runtime_plugin_chain_snapshot_compact, format_runtime_plugin_chain_snapshot_multiline,
    format_runtime_plugin_recall_snapshot_compact,
    format_runtime_routed_plugin_chain_summary_compact, json_runtime_plugin_chain_snapshot,
    json_runtime_routed_plugin_chain_summary,
};
pub use plugin_discovery_family::*;
use plugin_discovery_family::{
    json_runtime_lv2_extension_snapshot, json_runtime_plugin_complex_io_summary,
    json_runtime_plugin_discovery_snapshot, json_runtime_plugin_parity_coverage_vec,
    json_runtime_plugin_pin_group_identity_vec,
};
pub use plugin_recall_family::*;
use plugin_runtime_surface_family::{
    format_runtime_lv2_extension_snapshot_compact, format_runtime_lv2_extension_snapshot_multiline,
    format_runtime_plugin_discovery_snapshot_compact,
    format_runtime_plugin_discovery_snapshot_multiline,
    format_runtime_plugin_lifecycle_snapshot_compact,
    format_runtime_plugin_lifecycle_snapshot_multiline,
    format_runtime_plugin_pin_matrix_snapshot_compact,
    format_runtime_plugin_pin_matrix_snapshot_multiline, json_runtime_plugin_lifecycle_snapshot,
    json_runtime_plugin_pin_matrix_snapshot,
};
pub use preview_transform_family::*;
use preview_transform_family::{
    json_runtime_preview_transform_service_snapshot, json_runtime_transform_artifact_snapshot,
};
pub use receipt_surface_family::*;
use runtime_continuity_json_family::{
    format_runtime_offline_render_session_snapshot_compact,
    format_runtime_offline_render_session_snapshot_multiline,
    format_runtime_recording_capture_snapshot_compact,
    format_runtime_recording_capture_snapshot_multiline, json_runtime_degradation_summary,
    json_runtime_device_supervision_snapshot, json_runtime_fault_diagnostic_receipt,
    json_runtime_fault_status, json_runtime_interruption_summary,
    json_runtime_offline_render_session_snapshot, json_runtime_recording_capture_snapshot,
};
use scheduler_surface_family::{
    format_runtime_block_summary_compact, format_runtime_block_summary_multiline,
    format_runtime_scheduler_summary_compact, format_runtime_scheduler_summary_multiline,
    json_runtime_block_execution_summary, json_runtime_scheduler_export_summary,
    json_runtime_scheduler_snapshot,
};
use spatial_topology_json_family::{
    json_runtime_execution_topology_summary, json_runtime_spatial_execution_summary,
};

mod channel_bus_family;
pub use channel_bus_family::*;
mod graph_projection_family;
pub use graph_projection_family::*;
mod prework_forecast_family;
pub use prework_forecast_family::*;
mod sandbox_records_family;
pub use sandbox_records_family::*;
mod spatial_execution_family;
pub(crate) use spatial_execution_family::runtime_spatial_execution_summary_for_stages;
pub use spatial_execution_family::*;
mod transport_projection_family;
pub use transport_projection_family::*;

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use signal_graph::{
    GraphChannelAdaptationMode, GraphDynamicStageStateModel, GraphExecutionContext,
    GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeResetPolicy,
    GraphNodeSilencePolicy, GraphNodeTopologyRole, GraphStageSpec,
};
use signal_hardware::{AudioSampleFormat, BackendHealth, BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{
    BlockSequenceContinuityReport, CompletionState, PluginFeature, PluginFormat, PluginIoLayout,
    PluginLifecycleContract, PluginProcessingContract, PluginStateContract,
};
use signal_primitives::{AudioBuffer, ChannelLayout, SampleRate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeErrorKind {
    InvalidRequest,
    UnsupportedCapability,
    InvalidState,
    ResourceUnavailable,
    PluginFailure,
    HardwareFailure,
    Timeout,
    Fatal,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
}

impl RuntimeError {
    pub fn new(kind: RuntimeErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeRequest {
    pub client_version: String,
    pub anticipative_preferred: bool,
    pub max_sample_rate_hint: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HandshakeResponse {
    pub runtime_version: String,
    pub protocol_version: u32,
    pub supports_anticipative: bool,
    pub supports_dynamic_reconfigure: bool,
    pub max_channels: u32,
    pub max_sample_rate: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfigRequest {
    pub sample_rate: SampleRate,
    pub block_size: usize,
    pub anticipative_enabled: bool,
    pub realtime_safe_mode: bool,
    pub max_graph_latency_ms: Option<u32>,
    pub max_background_load_percent: Option<u8>,
}

impl RuntimeConfigRequest {
    pub fn new(sample_rate: u32, block_size: usize) -> Self {
        Self {
            sample_rate: SampleRate(sample_rate),
            block_size,
            anticipative_enabled: true,
            realtime_safe_mode: false,
            max_graph_latency_ms: None,
            max_background_load_percent: None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StopReason {
    UserRequested,
    DeviceReconfigure,
    DegradedModeRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecoveryRestartIntent {
    CrashRecovery,
    WatchdogRecovery,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartRequest {
    pub reconfigure: Option<RuntimeConfigRequest>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SafeModeRequest {
    pub enabled: bool,
}

mod plugin_pin_matrix_family;
pub(crate) use plugin_pin_matrix_family::runtime_bus_intents_for_topology_role;
pub(crate) use plugin_pin_matrix_family::runtime_bus_role_for_endpoint;
pub use plugin_pin_matrix_family::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePlannedGraphNode {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub group: GraphNodePlanningGroup,
    pub latency_samples: u32,
    pub topology_role: GraphNodeTopologyRole,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub input_bus_id: String,
    pub output_bus_id: String,
    pub input_channels: ChannelLayout,
    pub output_channels: ChannelLayout,
    pub input_layout: RuntimeMultichannelLayoutSummary,
    pub output_layout: RuntimeMultichannelLayoutSummary,
    pub input_bus_intent: RuntimeBusIntent,
    pub output_bus_intent: RuntimeBusIntent,
    pub secondary_input: Option<RuntimeSecondaryInputRouteSummary>,
    pub spatial_execution: Option<RuntimeSpatialExecutionSummary>,
    pub plugin_sandbox_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeEngineBlockResult {
    pub snapshot: RuntimeEngineBlockSnapshot,
    pub output: AudioBuffer,
    pub meter_sources: Vec<RuntimeMeterSourceSnapshot>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduleProjection {
    pub schedule_id: String,
    pub stream_count: usize,
}

mod fault_supervision_family;
pub use fault_supervision_family::*;

mod runtime_control_family;
pub use runtime_control_family::*;

mod metering_snapshot_family;
pub use metering_snapshot_family::*;

mod transport_fault_family;
pub use transport_fault_family::*;

mod transport_session_mgmt_family;
pub use transport_session_mgmt_family::*;

mod performance_snapshot_family;
pub(crate) use performance_snapshot_family::runtime_execution_lane_name;
pub use performance_snapshot_family::*;

mod execution_topology_family;
pub use execution_topology_family::*;

fn runtime_lane_for_group(group: GraphNodePlanningGroup) -> GraphExecutionLane {
    match group {
        GraphNodePlanningGroup::InlineRealtime | GraphNodePlanningGroup::StatefulRealtime => {
            GraphExecutionLane::Realtime
        }
        GraphNodePlanningGroup::AnticipativeEligible => GraphExecutionLane::Anticipative,
    }
}

mod observation_diagnostics_family;
pub use observation_diagnostics_family::*;

mod observation_report_family;
pub use observation_report_family::*;

mod supervisor_report_family;
pub use supervisor_report_family::*;


fn runtime_plugin_lifecycle_state_severity(state: RuntimePluginLifecycleState) -> u8 {
    match state {
        RuntimePluginLifecycleState::Faulted => 5,
        RuntimePluginLifecycleState::Quarantined => 4,
        RuntimePluginLifecycleState::Restarting => 3,
        RuntimePluginLifecycleState::Degraded => 2,
        RuntimePluginLifecycleState::Ready => 1,
        RuntimePluginLifecycleState::Booting | RuntimePluginLifecycleState::Stopped => 0,
    }
}

fn runtime_plugin_topology_fallback_to_negotiation_outcome(
    outcome: RuntimePluginTopologyFallbackOutcome,
) -> RuntimePluginNegotiationFallbackOutcome {
    match outcome {
        RuntimePluginTopologyFallbackOutcome::CollapseToPrimaryPath => {
            RuntimePluginNegotiationFallbackOutcome::RoutePrimaryOnly
        }
        RuntimePluginTopologyFallbackOutcome::BypassUnavailablePortGroup => {
            RuntimePluginNegotiationFallbackOutcome::DeactivateOptionalPath
        }
        RuntimePluginTopologyFallbackOutcome::MuteDependentOutput => {
            RuntimePluginNegotiationFallbackOutcome::CollapseToDeclaredBaseline
        }
        RuntimePluginTopologyFallbackOutcome::SafeModeDegradation => {
            RuntimePluginNegotiationFallbackOutcome::GuardedDegradation
        }
        RuntimePluginTopologyFallbackOutcome::TerminalPluginTopologyFailure => {
            RuntimePluginNegotiationFallbackOutcome::TerminalNegotiationFailure
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxSpec {
    pub sandbox_id: String,
    pub plugin_format: PluginFormat,
    pub plugin_type_id: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxHandle(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BackendPolicyOverride {
    pub tier: BackendPolicyTier,
}

pub trait RuntimeLifecycleApi {
    fn handshake(&mut self, request: HandshakeRequest) -> Result<HandshakeResponse, RuntimeError>;
    fn configure(&mut self, request: RuntimeConfigRequest) -> Result<(), RuntimeError>;
    fn start(&mut self) -> Result<(), RuntimeError>;
    fn stop(&mut self, reason: StopReason) -> Result<(), RuntimeError>;
    fn restart(&mut self, request: RestartRequest) -> Result<(), RuntimeError>;
    fn set_safe_mode(&mut self, request: SafeModeRequest) -> Result<(), RuntimeError>;
}

pub trait RuntimeProjectionApi {
    fn set_prework_service_pressure(
        &mut self,
        pressure: RuntimePreworkServicePressure,
    ) -> Result<(), RuntimeError>;
    fn set_prework_forecast_mode(
        &mut self,
        mode: RuntimePreworkForecastMode,
    ) -> Result<(), RuntimeError>;
    fn set_prework_forecast_profile(
        &mut self,
        selection: RuntimePreworkForecastProfileSelection,
    ) -> Result<(), RuntimeError>;
    fn set_prework_forecast_policy(
        &mut self,
        policy: RuntimePreworkForecastPolicy,
    ) -> Result<(), RuntimeError>;
    fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError>;
    fn apply_plugin_backed_node_bindings(
        &mut self,
        projection: PluginBackedNodeBindingProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_plugin_placement_policy(
        &mut self,
        policy: RuntimePluginPlacementPolicy,
    ) -> Result<(), RuntimeError>;
    fn apply_graph_contract_projection(
        &mut self,
        projection: GraphContractProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_graph_projection(
        &mut self,
        projection: GraphProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_schedule_projection(
        &mut self,
        projection: ScheduleProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_automation_projection(
        &mut self,
        projection: RuntimeAutomationProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_tempo_map_projection(
        &mut self,
        projection: RuntimeTempoMapProjection,
    ) -> Result<ProjectionReceipt, RuntimeError>;
    fn apply_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError>;
    fn apply_parameter_batch(&mut self, batch: ParameterBatch) -> Result<(), RuntimeError>;
    fn apply_hardware_config(&mut self, request: HardwareConfigRequest)
        -> Result<(), RuntimeError>;
}

pub trait RuntimeObservationApi {
    fn subscribe(&mut self, sink: Box<dyn RuntimeEventSink>) -> SubscriptionHandle;
    fn get_readiness(&self) -> RuntimeReadiness;
    fn get_acceptance_receipt(&self) -> RuntimeAcceptanceReceipt;
    fn get_effective_config(&self) -> EffectiveRuntimeConfig;
    fn get_control_snapshot(&self) -> RuntimeControlSnapshot;
    fn get_scheduler_snapshot(&self) -> RuntimeSchedulerSnapshot;
    fn get_scheduler_topology_summary(&self) -> RuntimeSchedulerTopologySummary;
    fn get_diagnostics_snapshot(&self) -> RuntimeDiagnosticsSnapshot;
    fn get_metering_snapshot(&self) -> RuntimeMeteringSnapshot;
    fn get_supervision_snapshot(&self) -> RuntimeSupervisionSnapshot;
    fn get_timeline_snapshot(&self) -> RuntimeTimelineSnapshot;
    fn get_transport_observation_snapshot(&self) -> RuntimeTransportObservationSnapshot;
    fn get_recording_capture_snapshot(&self) -> RuntimeRecordingCaptureSnapshot;
    fn get_offline_render_session_snapshot(&self) -> RuntimeOfflineRenderSessionSnapshot;
    fn get_media_pipeline_snapshot(&self) -> RuntimeMediaPipelineSnapshot;
    fn get_media_service_snapshot(&self) -> RuntimeMediaServiceSnapshot;
    fn get_media_library_service_snapshot(&self) -> RuntimeMediaLibraryServiceSnapshot;
    fn get_tempo_map_snapshot(&self) -> RuntimeTempoMapSnapshot;
    fn get_warp_pipeline_snapshot(&self) -> RuntimeWarpPipelineSnapshot;
    fn get_clip_processing_pipeline_snapshot(&self) -> RuntimeClipProcessingPipelineSnapshot;
    fn get_stretch_engine_snapshot(&self) -> RuntimeStretchEngineSnapshot;
    fn get_marker_analysis_snapshot(&self) -> RuntimeMarkerAnalysisSnapshot;
    fn get_transform_artifact_snapshot(&self) -> RuntimeTransformArtifactSnapshot;
    fn get_preview_transform_snapshot(&self) -> RuntimePreviewTransformServiceSnapshot;
    fn get_automation_snapshot(&self) -> RuntimeAutomationSnapshot;
    fn get_plugin_event_snapshot(&self) -> RuntimePluginEventSnapshot;
    fn get_engine_block_snapshot(&self) -> RuntimeEngineBlockSnapshot;
    fn get_execution_topology_summary(&self) -> RuntimeExecutionTopologySummary;
    fn get_transport_concurrency_snapshot(&self) -> RuntimeTransportConcurrencySnapshot;
    fn get_plugin_discovery_snapshot(&self) -> RuntimePluginDiscoverySnapshot;
    fn get_plugin_lifecycle_snapshot(&self) -> RuntimePluginLifecycleSnapshot;
    fn get_plugin_chain_snapshot(&self) -> RuntimePluginChainSnapshot;
    fn get_plugin_recall_handoff_snapshot(&self) -> RuntimePluginRecallHandoffSnapshot;
    fn get_last_deferred_service_receipt(&self) -> Option<RuntimeDeferredServiceReceipt>;
}

pub trait RuntimeSupervisorApi {
    fn start_plugin_scan(&mut self, request: PluginScanRequest)
        -> Result<ScanHandle, RuntimeError>;
    fn ensure_plugin_sandbox(
        &mut self,
        request: PluginSandboxSpec,
    ) -> Result<SandboxHandle, RuntimeError>;
    fn start_recording_capture(
        &mut self,
        request: RuntimeRecordingCaptureStartRequest,
    ) -> Result<(), RuntimeError>;
    fn finish_recording_capture(
        &mut self,
    ) -> Result<RuntimeRecordingCaptureCommitReceipt, RuntimeError>;
    fn cancel_recording_capture(&mut self) -> Result<(), RuntimeError>;
    fn reconcile_media_assets(
        &mut self,
        assets: Vec<RuntimeMediaAssetRegistration>,
    ) -> Result<(), RuntimeError>;
    fn start_media_preview(&mut self, asset_id: &str) -> Result<(), RuntimeError>;
    fn stop_media_preview(&mut self) -> Result<(), RuntimeError>;
    fn reconcile_warp_clips(
        &mut self,
        clips: Vec<RuntimeWarpClipRegistration>,
    ) -> Result<(), RuntimeError>;
    fn reconcile_clip_processing_clips(
        &mut self,
        clips: Vec<RuntimeClipProcessingRegistration>,
    ) -> Result<(), RuntimeError>;
    fn render_offline(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderResult, RuntimeError>;
    fn render_offline_with_checkpoints(
        &self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionReceipt, RuntimeError>;
    fn begin_offline_render_execution(
        &mut self,
        request: RuntimeOfflineRenderRequest,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn pause_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn resume_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn interrupt_offline_render_execution(
        &mut self,
        request_id: &str,
        reason: String,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn advance_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionProgressReceipt, RuntimeError>;
    fn cancel_offline_render_execution(
        &mut self,
        request_id: &str,
    ) -> Result<RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeError>;
    fn render_offline_queue(
        &self,
        requests: Vec<RuntimeOfflineRenderRequest>,
    ) -> Result<RuntimeOfflineRenderQueueResult, RuntimeError>;
    fn purge_offline_render_artifacts(
        &self,
        request: RuntimeOfflineRenderPurgeRequest,
    ) -> Result<RuntimeOfflineRenderPurgeReceipt, RuntimeError>;
    fn teardown_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn restart_plugin_sandbox(&mut self, sandbox_id: &str) -> Result<(), RuntimeError>;
    fn set_backend_policy(&mut self, request: BackendPolicyOverride) -> Result<(), RuntimeError>;
}

#[cfg(test)]
#[path = "interfaces_tests.rs"]
mod tests;
