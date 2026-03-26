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
mod interfaces_json_family;
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

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex},
};

use signal_graph::{
    GraphChannelAdaptationMode, GraphDynamicStageStateModel, GraphExecutionContext,
    GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeResetPolicy,
    GraphNodeSilencePolicy, GraphNodeTopologyRole, GraphStageSpec,
};
use signal_hardware::{
    AudioSampleFormat, BackendHealth, BackendPolicyTier, HardwareBackendIdentity,
    HardwareClockSource, HardwareClockTopology, HardwareConfigRequest, HardwareLifecycleOwnership,
    HardwareRestartPolicy, LinuxAudioBackendKind,
};
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

#[derive(Clone, Debug, PartialEq)]
pub struct GraphNodeProjection {
    pub node_id: String,
    pub execution_class: GraphNodeExecutionClass,
    pub latency_samples: u32,
    pub stages: Vec<GraphStageSpec>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GraphProjection {
    pub graph_id: String,
    pub node_count: usize,
    pub nodes: Vec<GraphNodeProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBusEndpointProjection {
    pub bus_id: String,
    pub channels: ChannelLayout,
}

impl Default for GraphNodeBusEndpointProjection {
    fn default() -> Self {
        Self {
            bus_id: "main:in".into(),
            channels: ChannelLayout::Stereo,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeBufferContractProjection {
    pub input: GraphNodeBusEndpointProjection,
    pub output: GraphNodeBusEndpointProjection,
    pub secondary_input: Option<RuntimeSecondaryInputContractProjection>,
    pub scratch_buffers: usize,
    pub silence_policy: GraphNodeSilencePolicy,
    pub channel_adaptation: GraphChannelAdaptationMode,
    pub reset_policy: GraphNodeResetPolicy,
}

impl Default for GraphNodeBufferContractProjection {
    fn default() -> Self {
        Self {
            input: GraphNodeBusEndpointProjection::default(),
            output: GraphNodeBusEndpointProjection {
                bus_id: "main:out".into(),
                channels: ChannelLayout::Stereo,
            },
            secondary_input: None,
            scratch_buffers: 0,
            silence_policy: GraphNodeSilencePolicy::Process,
            channel_adaptation: GraphChannelAdaptationMode::AdaptiveMonoStereo,
            reset_policy: GraphNodeResetPolicy::RetainAcrossBlocks,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphNodeTopologyProjection {
    pub role: Option<GraphNodeTopologyRole>,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphNodeContractProjection {
    pub node_id: String,
    pub buffer_contract: GraphNodeBufferContractProjection,
    pub topology: GraphNodeTopologyProjection,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphContractProjection {
    pub graph_id: String,
    pub contract_count: usize,
    pub nodes: Vec<GraphNodeContractProjection>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBinding {
    pub node_id: String,
    pub sandbox_id: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginIsolationOutcome {
    InProcess,
    SharedSandbox,
    #[default]
    IsolatedSandbox,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimePluginPlacementRuleMatcher {
    Any,
    PluginFormat(PluginFormat),
    PluginTypeId(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementRule {
    pub rule_id: String,
    pub matcher: RuntimePluginPlacementRuleMatcher,
    pub outcome: RuntimePluginIsolationOutcome,
    pub sandbox_group_key: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPlacementPolicy {
    pub default_outcome: RuntimePluginIsolationOutcome,
    pub rules: Vec<RuntimePluginPlacementRule>,
}

impl Default for RuntimePluginPlacementPolicy {
    fn default() -> Self {
        Self {
            default_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            rules: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginBackedNodeBindingProjection {
    pub graph_id: String,
    pub bindings: Vec<PluginBackedNodeBinding>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRender {
    pub node_id: String,
    pub sandbox_id: String,
    pub output: AudioBuffer,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub bypassed: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginNodeRenderBatch {
    pub graph_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub renders: Vec<PluginNodeRender>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePluginDispatchState {
    pub transport: Option<TransportProjection>,
    pub parameter_batch: Option<ParameterBatch>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkCacheState {
    Disabled,
    Empty,
    Admitted,
    Consumed,
    Invalidated,
}

impl Default for RuntimePreworkCacheState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkInvalidationReason {
    RuntimeReconfigured,
    RuntimeStopped,
    PlanningDisabled,
    ForecastPlanChanged,
    GraphProjectionChanged,
    TransportStarted,
    TransportStopped,
    TransportSeeked,
    TransportTempoChanged,
    TransportLoopStateChanged,
    TransportLoopWrapped,
    ParameterBatchApplied,
    InputSignatureChanged,
    ProcessingEpochExpired,
    BlockSequenceExpired,
    SupersededByAdmission,
    PlanningWindowRevised,
    QueueCapacityExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkRetirementReason {
    RuntimeReconfigured,
    RuntimeStopped,
    ForecastPlanChanged,
    GraphProjectionChanged,
    TransportStarted,
    TransportStopped,
    TransportSeeked,
    TransportTempoChanged,
    TransportLoopStateChanged,
    TransportLoopWrapped,
    ParameterBatchApplied,
    InputSignatureChanged,
    ProcessingEpochExpired,
    BlockSequenceExpired,
    PlanningDisabled,
    SupersededByAdmission,
    PlanningWindowRevised,
    QueueCapacityExceeded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkFreshnessState {
    Disabled,
    Empty,
    Fresh,
    Expiring,
    Exhausted,
    Invalidated,
}

impl Default for RuntimePreworkFreshnessState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkServiceState {
    Disabled,
    Idle,
    Pending,
    Servicing,
    Yielding,
    Paused,
    Starved,
}

impl Default for RuntimePreworkServiceState {
    fn default() -> Self {
        Self::Disabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkServicePressure {
    Normal,
    Elevated,
    Critical,
}

impl Default for RuntimePreworkServicePressure {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkServiceSemanticPolicy {
    Balanced,
    LatencyFocused,
    PluginConstrained,
}

impl Default for RuntimePreworkServiceSemanticPolicy {
    fn default() -> Self {
        Self::Balanced
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeSchedulerTopologyIssue {
    MissingTrackLaneIds {
        node_count: usize,
    },
    MissingBusGroupIds {
        node_count: usize,
    },
    MissingSendReturnIds {
        node_count: usize,
    },
    MissingConsoleGroupIds {
        node_count: usize,
    },
    MissingRealtimeLaneForTopology,
    AnticipativeLaneMustPrecedeRealtime,
    RealtimeDispatchMustTerminateTopology,
    MissingScheduleProjectionForTrackLanes {
        required_streams: usize,
    },
    InsufficientScheduleStreams {
        required_streams: usize,
        actual_streams: usize,
    },
}

/// Scheduler-facing topology summary derived from the active graph projection
/// and schedule view.
///
/// This tells hosts whether the runtime-owned planning shape lines up with the
/// declared track/bus/send/console topology or whether a host would need to
/// reinterpret the current plan boundary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSchedulerTopologySummary {
    pub track_lane_node_count: usize,
    pub track_lane_group_count: usize,
    pub bus_node_count: usize,
    pub bus_group_count: usize,
    pub send_return_node_count: usize,
    pub send_return_group_count: usize,
    pub console_node_count: usize,
    pub console_group_count: usize,
    pub schedule_stream_count: Option<usize>,
    pub compatible: bool,
    pub requires_host_reinterpretation: bool,
    pub issues: Vec<RuntimeSchedulerTopologyIssue>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeTransportTransitionKind {
    Initial,
    Started,
    Stopped,
    Seeked,
    TempoChanged,
    LoopStateChanged,
    LoopWrapped,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimePreworkBacklogClass {
    Immediate,
    NearTerm,
    Deferred,
}

impl Default for RuntimePreworkBacklogClass {
    fn default() -> Self {
        Self::Immediate
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBlockDeadlinePressure {
    #[default]
    Normal,
    Elevated,
    Critical,
    Overrun,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePreworkWindowTarget {
    pub target_block_sequence: u64,
    pub admitted_from_block_sequence: u64,
    pub buffer: AudioBuffer,
    pub parameter_epoch_override: Option<u64>,
    pub transport_override: Option<TransportProjection>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePreworkForecastPolicy {
    pub target_window_blocks: usize,
    pub prepare_budget_per_cycle: usize,
    pub buffer_seed_offset: u64,
    pub transport_playing: bool,
    pub transport_tempo_bpm: f64,
    pub transport_loop_length_blocks: usize,
    pub parameter_target: String,
    pub parameter_cycle_length: u64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreworkForecastMode {
    #[default]
    Disabled,
    RuntimeRoleDefault,
    ExplicitProfile,
    RawPolicyOverride,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkForecastProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePreworkForecastProfileSource {
    RuntimeRoleDefault,
    ExplicitSelection,
    RawPolicyOverride,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimePreworkForecastProfileSelection {
    pub profile: RuntimePreworkForecastProfile,
    pub target_window_blocks_override: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeCanonicalChannelLayout {
    Mono,
    Stereo,
    Lcr,
    Quad,
    Surround5_0,
    Surround5_1,
    Surround7_1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeChannelRole {
    Mono,
    FrontLeft,
    FrontRight,
    FrontCenter,
    LowFrequencyEffects,
    SideLeft,
    SideRight,
    RearLeft,
    RearRight,
    Discrete(u16),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusIntent {
    #[default]
    MainProgram,
    AuxSend,
    AuxReturn,
    Sidechain,
    HardwareInput,
    HardwareOutput,
    AnalysisTap,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputSourceKind {
    #[default]
    NodeOutput,
    BusGroup,
    HardwareInput,
    AnalysisTap,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputTargetKind {
    #[default]
    NodeInput,
    PluginInput,
    RenderInput,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputAttachmentPolicy {
    #[default]
    Required,
    Optional,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputFallbackOutcome {
    #[default]
    BypassSecondaryInput,
    MuteDependentPath,
    SafeModeDegradation,
    TerminalRoutingFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSecondaryInputContractProjection {
    pub source_kind: RuntimeSecondaryInputSourceKind,
    pub source_id: String,
    pub source_bus_id: Option<String>,
    pub target_bus_id: String,
    pub attachment_policy: RuntimeSecondaryInputAttachmentPolicy,
    pub fallback_outcome: RuntimeSecondaryInputFallbackOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSecondaryInputRouteSummary {
    pub source_kind: RuntimeSecondaryInputSourceKind,
    pub source_id: String,
    pub source_bus_id: Option<String>,
    pub target_kind: RuntimeSecondaryInputTargetKind,
    pub target_id: String,
    pub target_bus_id: String,
    pub attachment_policy: RuntimeSecondaryInputAttachmentPolicy,
    pub fallback_outcome: RuntimeSecondaryInputFallbackOutcome,
    pub summary: String,
}

impl RuntimeSecondaryInputRouteSummary {
    pub fn from_contract_for_target(
        contract: &RuntimeSecondaryInputContractProjection,
        target_kind: RuntimeSecondaryInputTargetKind,
        target_id: impl Into<String>,
    ) -> Self {
        let target_id = target_id.into();
        let summary = format!(
            "source={:?}:{}/{} target={:?}:{}/{} policy={:?} fallback={:?}",
            contract.source_kind,
            contract.source_id,
            contract.source_bus_id.as_deref().unwrap_or("none"),
            target_kind,
            target_id,
            contract.target_bus_id,
            contract.attachment_policy,
            contract.fallback_outcome,
        );
        Self {
            source_kind: contract.source_kind,
            source_id: contract.source_id.clone(),
            source_bus_id: contract.source_bus_id.clone(),
            target_kind,
            target_id,
            target_bus_id: contract.target_bus_id.clone(),
            attachment_policy: contract.attachment_policy,
            fallback_outcome: contract.fallback_outcome,
            summary,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusRole {
    #[default]
    ProgramMain,
    Submix,
    AuxSend,
    AuxReturn,
    AnalysisTap,
    HardwareIngress,
    HardwareEgress,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeAuxiliaryPathKind {
    #[default]
    SendReturn,
    Submix,
    Analysis,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusConnectionAttachmentClass {
    #[default]
    Required,
    Optional,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusConnectionFallbackOutcome {
    #[default]
    NoFallback,
    BypassAuxiliaryPath,
    MuteDependentPath,
    SafeModeDegradation,
    TerminalTopologyFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeBusConnectionSummary {
    pub connection_id: String,
    pub source_node_id: String,
    pub source_bus_id: String,
    pub source_bus_role: RuntimeBusRole,
    pub target_node_id: String,
    pub target_bus_id: String,
    pub target_bus_role: RuntimeBusRole,
    pub auxiliary_path_kind: Option<RuntimeAuxiliaryPathKind>,
    pub auxiliary_path_id: Option<String>,
    pub attachment_class: RuntimeBusConnectionAttachmentClass,
    pub fallback_outcome: RuntimeBusConnectionFallbackOutcome,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAuxiliaryPathSummary {
    pub auxiliary_path_id: String,
    pub path_kind: RuntimeAuxiliaryPathKind,
    pub bus_role: RuntimeBusRole,
    pub material_bus_intent: RuntimeBusIntent,
    pub source_node_ids: Vec<String>,
    pub target_node_ids: Vec<String>,
    pub bus_ids: Vec<String>,
    pub connection_ids: Vec<String>,
    pub attachment_class: RuntimeBusConnectionAttachmentClass,
    pub fallback_outcome: RuntimeBusConnectionFallbackOutcome,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMultichannelLayoutSummary {
    pub channel_count: u16,
    pub canonical_layout: Option<RuntimeCanonicalChannelLayout>,
    pub channel_roles: Vec<RuntimeChannelRole>,
    pub uses_custom_fallback: bool,
    pub summary: String,
}

impl Default for RuntimeMultichannelLayoutSummary {
    fn default() -> Self {
        Self::from_channel_count(0)
    }
}

impl RuntimeMultichannelLayoutSummary {
    pub fn from_channel_layout(layout: ChannelLayout) -> Self {
        Self::from_channel_count(layout.channels().0 as u16)
    }

    pub fn from_channel_count(channel_count: u16) -> Self {
        let (canonical_layout, channel_roles, uses_custom_fallback) = match channel_count {
            0 => (None, Vec::new(), false),
            1 => (
                Some(RuntimeCanonicalChannelLayout::Mono),
                vec![RuntimeChannelRole::Mono],
                false,
            ),
            2 => (
                Some(RuntimeCanonicalChannelLayout::Stereo),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                ],
                false,
            ),
            3 => (
                Some(RuntimeCanonicalChannelLayout::Lcr),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::FrontRight,
                ],
                false,
            ),
            4 => (
                Some(RuntimeCanonicalChannelLayout::Quad),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::RearLeft,
                    RuntimeChannelRole::RearRight,
                ],
                false,
            ),
            5 => (
                Some(RuntimeCanonicalChannelLayout::Surround5_0),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::SideLeft,
                    RuntimeChannelRole::SideRight,
                ],
                false,
            ),
            6 => (
                Some(RuntimeCanonicalChannelLayout::Surround5_1),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::LowFrequencyEffects,
                    RuntimeChannelRole::SideLeft,
                    RuntimeChannelRole::SideRight,
                ],
                false,
            ),
            8 => (
                Some(RuntimeCanonicalChannelLayout::Surround7_1),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::LowFrequencyEffects,
                    RuntimeChannelRole::SideLeft,
                    RuntimeChannelRole::SideRight,
                    RuntimeChannelRole::RearLeft,
                    RuntimeChannelRole::RearRight,
                ],
                false,
            ),
            _ => (
                None,
                (0..channel_count)
                    .map(RuntimeChannelRole::Discrete)
                    .collect(),
                true,
            ),
        };
        let summary = match canonical_layout {
            Some(layout) => format!(
                "channels={} canonical={layout:?} roles={:?}",
                channel_count, channel_roles
            ),
            None if channel_count == 0 => "channels=0 canonical=None roles=[]".into(),
            None => format!(
                "channels={} canonical=None roles={:?} fallback=Discrete",
                channel_count, channel_roles
            ),
        };
        Self {
            channel_count,
            canonical_layout,
            channel_roles,
            uses_custom_fallback,
            summary,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMultichannelIoSummary {
    pub input_layout: RuntimeMultichannelLayoutSummary,
    pub output_layout: RuntimeMultichannelLayoutSummary,
    pub input_bus_intent: RuntimeBusIntent,
    pub output_bus_intent: RuntimeBusIntent,
    pub summary: String,
}

impl Default for RuntimeMultichannelIoSummary {
    fn default() -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::default(),
            RuntimeMultichannelLayoutSummary::default(),
            RuntimeBusIntent::MainProgram,
            RuntimeBusIntent::MainProgram,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialAdapterClass {
    #[default]
    Balance,
    PerChannelGain,
    LayoutTransform,
    Renderer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialExecutionMode {
    #[default]
    Bypassed,
    BalanceGroups,
    PerChannelAttenuation,
    TransformToTargetLayout,
    RenderToEnvironment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialTargetEnvironment {
    #[default]
    SourceLayout,
    CanonicalLayout,
    DeviceLayout,
    CustomEnvironment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialControlFamily {
    #[default]
    BalanceScalar,
    PerChannelVector,
    AdapterParameterSet,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialActivationPolicy {
    Disabled,
    #[default]
    EnabledIfSupported,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialFallbackOutcome {
    BypassSpatialProcessing,
    CollapseToBalance,
    CollapseToPerChannelGain,
    SafeModeDegradation,
    TerminalSpatialFailure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialBedClass {
    #[default]
    StereoBed,
    CanonicalSurroundBed,
    CustomDiscreteBed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialObjectRole {
    PrimaryObject,
    AuxiliaryObject,
    EffectObject,
    AnalysisObject,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialMixPolicy {
    #[default]
    BedOnly,
    BedWithObjects,
    ObjectPreferredWithBedFallback,
    DownmixToCanonicalBed,
    CollapseToBaselineSpatial,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialRenderScope {
    #[default]
    BedRender,
    BedAndObjectRender,
    BedFoldDownRender,
    ObjectMetadataOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialExpandedFallbackOutcome {
    CollapseObjectsIntoBed,
    CollapseToCanonicalBed,
    CollapseToBaselineSpatial,
    BypassExpandedSpatial,
    TerminalExpandedSpatialFailure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveObjectRenderingPosture {
    #[default]
    NotRequested,
    MetadataOnly,
    RoomPolicyAware,
    CollapsedToBed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRoomPolicyClass {
    #[default]
    NoRoomPolicy,
    ReferenceRoom,
    MonitoringRoom,
    DeploymentRoom,
    FallbackRoom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRoomPolicyAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveRoomOutcome {
    #[default]
    BypassRoomPolicy,
    RenderObjectsAgainstRoomPolicy,
    PreserveObjectMetadataOnly,
    CollapseObjectsIntoBed,
    TerminalImmersiveFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeImmersiveRoomPolicySummary {
    pub object_rendering_posture: RuntimeImmersiveObjectRenderingPosture,
    pub room_policy_class: RuntimeRoomPolicyClass,
    pub room_policy_authority: RuntimeRoomPolicyAuthority,
    pub room_outcome: RuntimeImmersiveRoomOutcome,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeploymentClass {
    #[default]
    SourceLayoutDeployment,
    ReferenceSpeakerDeployment,
    MonitoringSpeakerDeployment,
    PortableFoldDownDeployment,
    FallbackDeployment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeFoldDownPolicy {
    #[default]
    PreserveDeclaredDeployment,
    FoldDownToReferenceBed,
    FoldDownToStereoMonitoring,
    FoldDownToPortablePreview,
    BypassDeploymentPolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringSceneClass {
    #[default]
    NoMonitoringScene,
    ReferenceScene,
    FoldDownScene,
    ConfidenceScene,
    FallbackScene,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringSceneAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringOutcome {
    MonitorDeclaredDeployment,
    MonitorFoldedDownScene,
    MonitorPortablePreview,
    #[default]
    BypassMonitoringScene,
    TerminalMonitoringFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDeploymentMonitoringSummary {
    pub deployment_class: RuntimeDeploymentClass,
    pub fold_down_policy: RuntimeFoldDownPolicy,
    pub monitoring_scene_class: RuntimeMonitoringSceneClass,
    pub monitoring_scene_authority: RuntimeMonitoringSceneAuthority,
    pub monitoring_outcome: RuntimeMonitoringOutcome,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRendererCapabilityNegotiationPosture {
    #[default]
    NotRequested,
    DeclaredCompatible,
    NegotiatedCompatible,
    FallbackNegotiation,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRendererCapabilityAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportClass {
    #[default]
    NoImmersiveExport,
    BedOnlyExport,
    ObjectAwareExport,
    MonitoringPreviewExport,
    FallbackExport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportOutcome {
    PreserveDeclaredExport,
    CollapseToBedExport,
    PreserveMetadataOnly,
    #[default]
    BypassImmersiveExport,
    TerminalExportFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeRendererImmersiveExportSummary {
    pub renderer_capability_posture: RuntimeRendererCapabilityNegotiationPosture,
    pub capability_authority: RuntimeRendererCapabilityAuthority,
    pub immersive_export_class: RuntimeImmersiveExportClass,
    pub export_authority: RuntimeImmersiveExportAuthority,
    pub export_outcome: RuntimeImmersiveExportOutcome,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpatialExecutionSummary {
    pub node_id: String,
    pub adapter_class: RuntimeSpatialAdapterClass,
    pub execution_mode: RuntimeSpatialExecutionMode,
    pub target_environment: RuntimeSpatialTargetEnvironment,
    pub control_family: RuntimeSpatialControlFamily,
    pub activation_policy: RuntimeSpatialActivationPolicy,
    pub fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    pub bed_class: RuntimeSpatialBedClass,
    pub object_role: Option<RuntimeSpatialObjectRole>,
    pub object_count: usize,
    pub mix_policy: RuntimeSpatialMixPolicy,
    pub render_scope: RuntimeSpatialRenderScope,
    pub expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
    pub immersive_room_policy: Option<RuntimeImmersiveRoomPolicySummary>,
    pub deployment_monitoring: Option<RuntimeDeploymentMonitoringSummary>,
    pub renderer_export: Option<RuntimeRendererImmersiveExportSummary>,
    pub balance: Option<String>,
    pub input_layout: RuntimeMultichannelLayoutSummary,
    pub output_layout: RuntimeMultichannelLayoutSummary,
    pub summary: String,
}

fn runtime_spatial_target_environment_for_layout(
    layout: &RuntimeMultichannelLayoutSummary,
) -> RuntimeSpatialTargetEnvironment {
    if layout.uses_custom_fallback {
        RuntimeSpatialTargetEnvironment::CustomEnvironment
    } else {
        RuntimeSpatialTargetEnvironment::SourceLayout
    }
}

fn runtime_spatial_bed_class_for_layout(
    layout: &RuntimeMultichannelLayoutSummary,
) -> RuntimeSpatialBedClass {
    match layout.canonical_layout {
        Some(RuntimeCanonicalChannelLayout::Stereo) if layout.channel_count == 2 => {
            RuntimeSpatialBedClass::StereoBed
        }
        Some(
            RuntimeCanonicalChannelLayout::Lcr
            | RuntimeCanonicalChannelLayout::Quad
            | RuntimeCanonicalChannelLayout::Surround5_0
            | RuntimeCanonicalChannelLayout::Surround5_1
            | RuntimeCanonicalChannelLayout::Surround7_1,
        ) => RuntimeSpatialBedClass::CanonicalSurroundBed,
        _ => RuntimeSpatialBedClass::CustomDiscreteBed,
    }
}

fn runtime_spatial_mix_policy_for_layout(
    layout: &RuntimeMultichannelLayoutSummary,
) -> RuntimeSpatialMixPolicy {
    if layout.channel_count == 2 && !layout.uses_custom_fallback {
        RuntimeSpatialMixPolicy::BedOnly
    } else {
        RuntimeSpatialMixPolicy::CollapseToBaselineSpatial
    }
}

fn runtime_spatial_render_scope_for_summary(
    object_count: usize,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
) -> RuntimeSpatialRenderScope {
    if object_count > 0 {
        if matches!(
            expanded_fallback_outcome,
            Some(RuntimeSpatialExpandedFallbackOutcome::CollapseObjectsIntoBed)
        ) {
            RuntimeSpatialRenderScope::BedFoldDownRender
        } else {
            RuntimeSpatialRenderScope::BedAndObjectRender
        }
    } else {
        RuntimeSpatialRenderScope::BedRender
    }
}

fn runtime_immersive_room_policy_summary_for_spatial(
    adapter_class: RuntimeSpatialAdapterClass,
    execution_mode: RuntimeSpatialExecutionMode,
    target_environment: RuntimeSpatialTargetEnvironment,
    fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    bed_class: RuntimeSpatialBedClass,
    object_role: Option<RuntimeSpatialObjectRole>,
    object_count: usize,
    render_scope: RuntimeSpatialRenderScope,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
) -> Option<RuntimeImmersiveRoomPolicySummary> {
    let immersive_candidate = bed_class != RuntimeSpatialBedClass::StereoBed
        || object_count > 0
        || object_role.is_some()
        || adapter_class == RuntimeSpatialAdapterClass::Renderer
        || execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
        || target_environment != RuntimeSpatialTargetEnvironment::SourceLayout
        || matches!(
            render_scope,
            RuntimeSpatialRenderScope::BedAndObjectRender
                | RuntimeSpatialRenderScope::BedFoldDownRender
                | RuntimeSpatialRenderScope::ObjectMetadataOnly
        );
    if !immersive_candidate {
        return None;
    }

    let room_policy_class = if execution_mode == RuntimeSpatialExecutionMode::Bypassed
        || fallback_outcome.is_some()
        || expanded_fallback_outcome.is_some()
    {
        RuntimeRoomPolicyClass::FallbackRoom
    } else {
        match target_environment {
            RuntimeSpatialTargetEnvironment::SourceLayout
            | RuntimeSpatialTargetEnvironment::CanonicalLayout => {
                RuntimeRoomPolicyClass::ReferenceRoom
            }
            RuntimeSpatialTargetEnvironment::DeviceLayout => RuntimeRoomPolicyClass::MonitoringRoom,
            RuntimeSpatialTargetEnvironment::CustomEnvironment => {
                RuntimeRoomPolicyClass::DeploymentRoom
            }
        }
    };

    let room_policy_authority = if room_policy_class == RuntimeRoomPolicyClass::FallbackRoom {
        RuntimeRoomPolicyAuthority::RuntimeDefault
    } else if execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
        || adapter_class == RuntimeSpatialAdapterClass::Renderer
    {
        RuntimeRoomPolicyAuthority::RendererAdvisory
    } else {
        RuntimeRoomPolicyAuthority::RuntimeDeclared
    };

    let object_rendering_posture = if object_count == 0 && object_role.is_none() {
        RuntimeImmersiveObjectRenderingPosture::NotRequested
    } else if render_scope == RuntimeSpatialRenderScope::ObjectMetadataOnly {
        RuntimeImmersiveObjectRenderingPosture::MetadataOnly
    } else if execution_mode == RuntimeSpatialExecutionMode::Bypassed
        || fallback_outcome.is_some()
        || matches!(
            expanded_fallback_outcome,
            Some(
                RuntimeSpatialExpandedFallbackOutcome::CollapseObjectsIntoBed
                    | RuntimeSpatialExpandedFallbackOutcome::CollapseToCanonicalBed
                    | RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial
                    | RuntimeSpatialExpandedFallbackOutcome::BypassExpandedSpatial
            )
        )
    {
        RuntimeImmersiveObjectRenderingPosture::CollapsedToBed
    } else if room_policy_class == RuntimeRoomPolicyClass::FallbackRoom {
        RuntimeImmersiveObjectRenderingPosture::Unavailable
    } else {
        RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
    };

    let room_outcome = if matches!(
        expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::TerminalExpandedSpatialFailure)
    ) || fallback_outcome
        == Some(RuntimeSpatialFallbackOutcome::TerminalSpatialFailure)
    {
        RuntimeImmersiveRoomOutcome::TerminalImmersiveFailure
    } else {
        match object_rendering_posture {
            RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware => {
                RuntimeImmersiveRoomOutcome::RenderObjectsAgainstRoomPolicy
            }
            RuntimeImmersiveObjectRenderingPosture::MetadataOnly => {
                RuntimeImmersiveRoomOutcome::PreserveObjectMetadataOnly
            }
            RuntimeImmersiveObjectRenderingPosture::CollapsedToBed => {
                RuntimeImmersiveRoomOutcome::CollapseObjectsIntoBed
            }
            RuntimeImmersiveObjectRenderingPosture::NotRequested
            | RuntimeImmersiveObjectRenderingPosture::Unavailable => {
                RuntimeImmersiveRoomOutcome::BypassRoomPolicy
            }
        }
    };

    Some(RuntimeImmersiveRoomPolicySummary {
        object_rendering_posture,
        room_policy_class,
        room_policy_authority,
        room_outcome,
        summary: format!(
            "objects={:?} room_class={:?} authority={:?} outcome={:?}",
            object_rendering_posture, room_policy_class, room_policy_authority, room_outcome,
        ),
    })
}

fn runtime_deployment_monitoring_summary_for_spatial(
    target_environment: RuntimeSpatialTargetEnvironment,
    bed_class: RuntimeSpatialBedClass,
    fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
    immersive_room_policy: Option<&RuntimeImmersiveRoomPolicySummary>,
) -> Option<RuntimeDeploymentMonitoringSummary> {
    let immersive_room_policy = immersive_room_policy?;
    let fallback_active = immersive_room_policy.room_policy_class
        == RuntimeRoomPolicyClass::FallbackRoom
        || fallback_outcome.is_some()
        || expanded_fallback_outcome.is_some();

    let deployment_class = if fallback_active {
        RuntimeDeploymentClass::FallbackDeployment
    } else {
        match target_environment {
            RuntimeSpatialTargetEnvironment::SourceLayout => {
                RuntimeDeploymentClass::SourceLayoutDeployment
            }
            RuntimeSpatialTargetEnvironment::CanonicalLayout => {
                RuntimeDeploymentClass::ReferenceSpeakerDeployment
            }
            RuntimeSpatialTargetEnvironment::DeviceLayout => {
                RuntimeDeploymentClass::MonitoringSpeakerDeployment
            }
            RuntimeSpatialTargetEnvironment::CustomEnvironment => {
                RuntimeDeploymentClass::PortableFoldDownDeployment
            }
        }
    };

    let fold_down_policy = if fallback_active {
        match bed_class {
            RuntimeSpatialBedClass::StereoBed => RuntimeFoldDownPolicy::FoldDownToStereoMonitoring,
            RuntimeSpatialBedClass::CanonicalSurroundBed
            | RuntimeSpatialBedClass::CustomDiscreteBed => {
                RuntimeFoldDownPolicy::FoldDownToReferenceBed
            }
        }
    } else {
        match deployment_class {
            RuntimeDeploymentClass::PortableFoldDownDeployment => {
                RuntimeFoldDownPolicy::FoldDownToPortablePreview
            }
            RuntimeDeploymentClass::SourceLayoutDeployment
            | RuntimeDeploymentClass::ReferenceSpeakerDeployment
            | RuntimeDeploymentClass::MonitoringSpeakerDeployment => {
                RuntimeFoldDownPolicy::PreserveDeclaredDeployment
            }
            RuntimeDeploymentClass::FallbackDeployment => {
                RuntimeFoldDownPolicy::BypassDeploymentPolicy
            }
        }
    };

    let monitoring_scene_class = if fallback_active {
        RuntimeMonitoringSceneClass::FallbackScene
    } else {
        match fold_down_policy {
            RuntimeFoldDownPolicy::PreserveDeclaredDeployment => {
                if deployment_class == RuntimeDeploymentClass::MonitoringSpeakerDeployment {
                    RuntimeMonitoringSceneClass::ConfidenceScene
                } else {
                    RuntimeMonitoringSceneClass::ReferenceScene
                }
            }
            RuntimeFoldDownPolicy::FoldDownToReferenceBed
            | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring
            | RuntimeFoldDownPolicy::FoldDownToPortablePreview => {
                RuntimeMonitoringSceneClass::FoldDownScene
            }
            RuntimeFoldDownPolicy::BypassDeploymentPolicy => {
                RuntimeMonitoringSceneClass::FallbackScene
            }
        }
    };

    let monitoring_scene_authority = if fallback_active {
        RuntimeMonitoringSceneAuthority::RuntimeDefault
    } else if deployment_class == RuntimeDeploymentClass::MonitoringSpeakerDeployment {
        RuntimeMonitoringSceneAuthority::HostForwarded
    } else {
        RuntimeMonitoringSceneAuthority::RuntimeDeclared
    };

    let monitoring_outcome = if matches!(
        expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::TerminalExpandedSpatialFailure)
    ) || fallback_outcome
        == Some(RuntimeSpatialFallbackOutcome::TerminalSpatialFailure)
    {
        RuntimeMonitoringOutcome::TerminalMonitoringFailure
    } else if fallback_active {
        RuntimeMonitoringOutcome::BypassMonitoringScene
    } else {
        match fold_down_policy {
            RuntimeFoldDownPolicy::PreserveDeclaredDeployment => {
                RuntimeMonitoringOutcome::MonitorDeclaredDeployment
            }
            RuntimeFoldDownPolicy::FoldDownToReferenceBed
            | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring => {
                RuntimeMonitoringOutcome::MonitorFoldedDownScene
            }
            RuntimeFoldDownPolicy::FoldDownToPortablePreview => {
                RuntimeMonitoringOutcome::MonitorPortablePreview
            }
            RuntimeFoldDownPolicy::BypassDeploymentPolicy => {
                RuntimeMonitoringOutcome::BypassMonitoringScene
            }
        }
    };

    Some(RuntimeDeploymentMonitoringSummary {
        deployment_class,
        fold_down_policy,
        monitoring_scene_class,
        monitoring_scene_authority,
        monitoring_outcome,
        summary: format!(
            "deployment={:?} fold_down={:?} scene={:?} authority={:?} outcome={:?}",
            deployment_class,
            fold_down_policy,
            monitoring_scene_class,
            monitoring_scene_authority,
            monitoring_outcome,
        ),
    })
}

fn runtime_renderer_immersive_export_summary_for_spatial(
    adapter_class: RuntimeSpatialAdapterClass,
    execution_mode: RuntimeSpatialExecutionMode,
    target_environment: RuntimeSpatialTargetEnvironment,
    fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
    immersive_room_policy: Option<&RuntimeImmersiveRoomPolicySummary>,
    deployment_monitoring: Option<&RuntimeDeploymentMonitoringSummary>,
) -> Option<RuntimeRendererImmersiveExportSummary> {
    let immersive_room_policy = immersive_room_policy?;
    let fallback_active = immersive_room_policy.room_policy_class
        == RuntimeRoomPolicyClass::FallbackRoom
        || deployment_monitoring.is_some_and(|monitoring| {
            monitoring.monitoring_scene_class == RuntimeMonitoringSceneClass::FallbackScene
        })
        || fallback_outcome.is_some()
        || expanded_fallback_outcome.is_some();

    let renderer_capability_posture = if fallback_active {
        RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
    } else if adapter_class == RuntimeSpatialAdapterClass::Renderer
        || execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
    {
        RuntimeRendererCapabilityNegotiationPosture::NegotiatedCompatible
    } else if target_environment != RuntimeSpatialTargetEnvironment::SourceLayout {
        RuntimeRendererCapabilityNegotiationPosture::DeclaredCompatible
    } else {
        RuntimeRendererCapabilityNegotiationPosture::DeclaredCompatible
    };

    let capability_authority = if fallback_active {
        RuntimeRendererCapabilityAuthority::RuntimeDefault
    } else if adapter_class == RuntimeSpatialAdapterClass::Renderer
        || execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
    {
        RuntimeRendererCapabilityAuthority::RendererAdvisory
    } else if deployment_monitoring.is_some_and(|monitoring| {
        monitoring.deployment_class == RuntimeDeploymentClass::MonitoringSpeakerDeployment
    }) {
        RuntimeRendererCapabilityAuthority::HostForwarded
    } else {
        RuntimeRendererCapabilityAuthority::RuntimeDeclared
    };

    let immersive_export_class = if fallback_active {
        RuntimeImmersiveExportClass::FallbackExport
    } else if immersive_room_policy.object_rendering_posture
        == RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
    {
        RuntimeImmersiveExportClass::ObjectAwareExport
    } else if deployment_monitoring.is_some_and(|monitoring| {
        monitoring.fold_down_policy != RuntimeFoldDownPolicy::PreserveDeclaredDeployment
    }) {
        RuntimeImmersiveExportClass::MonitoringPreviewExport
    } else {
        RuntimeImmersiveExportClass::BedOnlyExport
    };

    let export_authority = if fallback_active {
        RuntimeImmersiveExportAuthority::RuntimeDefault
    } else if adapter_class == RuntimeSpatialAdapterClass::Renderer
        || execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
    {
        RuntimeImmersiveExportAuthority::RendererAdvisory
    } else if deployment_monitoring.is_some_and(|monitoring| {
        monitoring.monitoring_scene_authority == RuntimeMonitoringSceneAuthority::HostForwarded
    }) {
        RuntimeImmersiveExportAuthority::HostForwarded
    } else {
        RuntimeImmersiveExportAuthority::RuntimeDeclared
    };

    let export_outcome = if matches!(
        expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::TerminalExpandedSpatialFailure)
    ) || fallback_outcome
        == Some(RuntimeSpatialFallbackOutcome::TerminalSpatialFailure)
    {
        RuntimeImmersiveExportOutcome::TerminalExportFailure
    } else if fallback_active {
        match immersive_room_policy.room_outcome {
            RuntimeImmersiveRoomOutcome::PreserveObjectMetadataOnly => {
                RuntimeImmersiveExportOutcome::PreserveMetadataOnly
            }
            RuntimeImmersiveRoomOutcome::CollapseObjectsIntoBed => {
                RuntimeImmersiveExportOutcome::CollapseToBedExport
            }
            RuntimeImmersiveRoomOutcome::BypassRoomPolicy
            | RuntimeImmersiveRoomOutcome::RenderObjectsAgainstRoomPolicy => {
                RuntimeImmersiveExportOutcome::BypassImmersiveExport
            }
            RuntimeImmersiveRoomOutcome::TerminalImmersiveFailure => {
                RuntimeImmersiveExportOutcome::TerminalExportFailure
            }
        }
    } else {
        match immersive_export_class {
            RuntimeImmersiveExportClass::BedOnlyExport
            | RuntimeImmersiveExportClass::ObjectAwareExport
            | RuntimeImmersiveExportClass::MonitoringPreviewExport => {
                RuntimeImmersiveExportOutcome::PreserveDeclaredExport
            }
            RuntimeImmersiveExportClass::FallbackExport
            | RuntimeImmersiveExportClass::NoImmersiveExport => {
                RuntimeImmersiveExportOutcome::BypassImmersiveExport
            }
        }
    };

    Some(RuntimeRendererImmersiveExportSummary {
        renderer_capability_posture,
        capability_authority,
        immersive_export_class,
        export_authority,
        export_outcome,
        summary: format!(
            "renderer={:?} capability_authority={:?} export={:?} export_authority={:?} outcome={:?}",
            renderer_capability_posture,
            capability_authority,
            immersive_export_class,
            export_authority,
            export_outcome,
        ),
    })
}

pub(crate) fn runtime_spatial_execution_summary_for_stages(
    node_id: &str,
    stages: &[GraphStageSpec],
    input_layout: &RuntimeMultichannelLayoutSummary,
    output_layout: &RuntimeMultichannelLayoutSummary,
) -> Option<RuntimeSpatialExecutionSummary> {
    stages.iter().find_map(|stage| match stage {
        GraphStageSpec::StereoBalance { balance } => {
            let supports_direct_balance = output_layout.channel_count == 2;
            let execution_mode = if supports_direct_balance {
                RuntimeSpatialExecutionMode::BalanceGroups
            } else {
                RuntimeSpatialExecutionMode::Bypassed
            };
            let fallback_outcome = (!supports_direct_balance)
                .then_some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing);
            let bed_class = runtime_spatial_bed_class_for_layout(output_layout);
            let object_count = 0usize;
            let mix_policy = runtime_spatial_mix_policy_for_layout(output_layout);
            let expanded_fallback_outcome = (!supports_direct_balance)
                .then_some(RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial);
            let render_scope =
                runtime_spatial_render_scope_for_summary(object_count, expanded_fallback_outcome);
            let balance = format!("{balance:.3}");
            let target_environment = runtime_spatial_target_environment_for_layout(output_layout);
            let immersive_room_policy = runtime_immersive_room_policy_summary_for_spatial(
                RuntimeSpatialAdapterClass::Balance,
                execution_mode,
                target_environment,
                fallback_outcome,
                bed_class,
                None,
                object_count,
                render_scope,
                expanded_fallback_outcome,
            );
            let deployment_monitoring = runtime_deployment_monitoring_summary_for_spatial(
                target_environment,
                bed_class,
                fallback_outcome,
                expanded_fallback_outcome,
                immersive_room_policy.as_ref(),
            );
            let renderer_export = runtime_renderer_immersive_export_summary_for_spatial(
                RuntimeSpatialAdapterClass::Balance,
                execution_mode,
                target_environment,
                fallback_outcome,
                expanded_fallback_outcome,
                immersive_room_policy.as_ref(),
                deployment_monitoring.as_ref(),
            );
            Some(RuntimeSpatialExecutionSummary {
                node_id: node_id.into(),
                adapter_class: RuntimeSpatialAdapterClass::Balance,
                execution_mode,
                target_environment,
                control_family: RuntimeSpatialControlFamily::BalanceScalar,
                activation_policy: RuntimeSpatialActivationPolicy::EnabledIfSupported,
                fallback_outcome,
                bed_class,
                object_role: None,
                object_count,
                mix_policy,
                render_scope,
                expanded_fallback_outcome,
                immersive_room_policy: immersive_room_policy.clone(),
                deployment_monitoring: deployment_monitoring.clone(),
                renderer_export: renderer_export.clone(),
                balance: Some(balance.clone()),
                input_layout: input_layout.clone(),
                output_layout: output_layout.clone(),
                summary: format!(
                    "node={} adapter={:?} mode={:?} target={:?} controls={:?} policy={:?} fallback={:?}/{:?} bed={:?} objects={:?}/{} mix={:?} render={:?} immersive={:?} monitoring={:?} export={:?} balance={} input={} output={}",
                    node_id,
                    RuntimeSpatialAdapterClass::Balance,
                    execution_mode,
                    target_environment,
                    RuntimeSpatialControlFamily::BalanceScalar,
                    RuntimeSpatialActivationPolicy::EnabledIfSupported,
                    fallback_outcome,
                    expanded_fallback_outcome,
                    bed_class,
                    None::<RuntimeSpatialObjectRole>,
                    object_count,
                    mix_policy,
                    render_scope,
                    immersive_room_policy.as_ref().map(|summary| &summary.summary),
                    deployment_monitoring.as_ref().map(|summary| &summary.summary),
                    renderer_export.as_ref().map(|summary| &summary.summary),
                    balance,
                    input_layout.summary,
                    output_layout.summary,
                ),
            })
        }
        _ => None,
    })
}

impl RuntimeMultichannelIoSummary {
    pub fn new(
        input_layout: RuntimeMultichannelLayoutSummary,
        output_layout: RuntimeMultichannelLayoutSummary,
        input_bus_intent: RuntimeBusIntent,
        output_bus_intent: RuntimeBusIntent,
    ) -> Self {
        let summary = format!(
            "input={:?}/{:?} output={:?}/{:?}",
            input_bus_intent,
            input_layout.canonical_layout,
            output_bus_intent,
            output_layout.canonical_layout
        );
        Self {
            input_layout,
            output_layout,
            input_bus_intent,
            output_bus_intent,
            summary,
        }
    }

    pub fn for_channel_layouts(
        input_layout: ChannelLayout,
        output_layout: ChannelLayout,
        input_bus_intent: RuntimeBusIntent,
        output_bus_intent: RuntimeBusIntent,
    ) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_layout(input_layout),
            RuntimeMultichannelLayoutSummary::from_channel_layout(output_layout),
            input_bus_intent,
            output_bus_intent,
        )
    }

    pub fn for_plugin_io(layout: PluginIoLayout) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_count(layout.audio_inputs),
            RuntimeMultichannelLayoutSummary::from_channel_count(layout.audio_outputs),
            RuntimeBusIntent::MainProgram,
            RuntimeBusIntent::MainProgram,
        )
    }

    pub fn for_hardware(input_channels: u16, output_channels: u16) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_count(input_channels),
            RuntimeMultichannelLayoutSummary::from_channel_count(output_channels),
            RuntimeBusIntent::HardwareInput,
            RuntimeBusIntent::HardwareOutput,
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPortClass {
    #[default]
    MainInput,
    MainOutput,
    SecondaryInput,
    AuxInput,
    AuxOutput,
    InstrumentOutput,
    AnalysisOutput,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginBusCapableFxClass {
    #[default]
    SinglePathFx,
    SidechainCapableFx,
    SendReturnCapableFx,
    ParallelCapableFx,
    MultiStemFx,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginTopologyAttachmentPolicy {
    #[default]
    Required,
    Optional,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginTopologyFallbackOutcome {
    #[default]
    CollapseToPrimaryPath,
    BypassUnavailablePortGroup,
    MuteDependentOutput,
    SafeModeDegradation,
    TerminalPluginTopologyFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginComplexIoSummary {
    pub has_complex_topology: bool,
    pub declared_port_classes: Vec<RuntimePluginPortClass>,
    pub port_group_count: usize,
    pub main_input_group_count: usize,
    pub main_output_group_count: usize,
    pub secondary_input_group_count: usize,
    pub aux_input_group_count: usize,
    pub aux_output_group_count: usize,
    pub instrument_output_group_count: usize,
    pub analysis_output_group_count: usize,
    pub multi_output_instrument: bool,
    pub bus_capable_fx_class: Option<RuntimePluginBusCapableFxClass>,
    pub attachment_policy: RuntimePluginTopologyAttachmentPolicy,
    pub fallback_outcome: RuntimePluginTopologyFallbackOutcome,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePluginPinGroupIdentity {
    PrimaryProgramPath,
    SecondaryProgramPath,
    AuxReturnPath,
    SidechainPath,
    AnalysisPath,
    InactiveDeclaredPath,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPinMatrixPosture {
    #[default]
    Simple,
    Declared,
    Negotiated,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDynamicBusNegotiationPosture {
    #[default]
    Static,
    Negotiated,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginNegotiationFallbackOutcome {
    CollapseToDeclaredBaseline,
    DeactivateOptionalPath,
    #[default]
    RoutePrimaryOnly,
    GuardedDegradation,
    TerminalNegotiationFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPinMatrixRecord {
    pub plugin_type_id: String,
    pub plugin_id: String,
    pub pin_group_identities: Vec<RuntimePluginPinGroupIdentity>,
    pub pin_matrix_posture: RuntimePluginPinMatrixPosture,
    pub dynamic_bus_negotiation_posture: RuntimeDynamicBusNegotiationPosture,
    pub fallback_outcome: RuntimePluginNegotiationFallbackOutcome,
    pub strongest_lifecycle_state: Option<RuntimePluginLifecycleState>,
    pub stage_count: usize,
    pub active_stage_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginPinMatrixSnapshot {
    pub plugin_type_count: usize,
    pub negotiated_type_count: usize,
    pub guarded_type_count: usize,
    pub unavailable_type_count: usize,
    pub dynamic_negotiated_type_count: usize,
    pub dynamic_guarded_type_count: usize,
    pub records: Vec<RuntimePluginPinMatrixRecord>,
    pub summary: String,
}

fn div_ceil_u16(value: u16, divisor: u16) -> u16 {
    if value == 0 {
        0
    } else {
        1 + ((value - 1) / divisor)
    }
}

impl RuntimePluginComplexIoSummary {
    pub fn from_plugin_features_and_layout(
        features: &[PluginFeature],
        layout: PluginIoLayout,
    ) -> Self {
        let is_instrument = features.contains(&PluginFeature::Instrument);
        let is_analyzer = features.contains(&PluginFeature::Analyzer);
        let is_fx = features.iter().any(|feature| {
            matches!(
                feature,
                PluginFeature::AudioEffect
                    | PluginFeature::Utility
                    | PluginFeature::Analyzer
                    | PluginFeature::NoteEffect
            )
        }) && !is_instrument;

        let main_input_group_count = usize::from(layout.audio_inputs > 0);
        let main_output_group_count = usize::from(layout.audio_outputs > 0);
        let main_input_channels = if layout.audio_inputs > 0 {
            layout.audio_inputs.min(2)
        } else {
            0
        };
        let main_output_channels = if layout.audio_outputs > 0 {
            layout.audio_outputs.min(2)
        } else {
            0
        };
        let extra_input_groups = usize::from(div_ceil_u16(
            layout.audio_inputs.saturating_sub(main_input_channels),
            2,
        ));
        let extra_output_groups = usize::from(div_ceil_u16(
            layout.audio_outputs.saturating_sub(main_output_channels),
            2,
        ));

        let secondary_input_group_count = if is_fx && extra_input_groups > 0 {
            1
        } else {
            0
        };
        let aux_input_group_count = if is_fx {
            extra_input_groups.saturating_sub(secondary_input_group_count)
        } else {
            0
        };
        let instrument_output_group_count = if is_instrument {
            extra_output_groups
        } else {
            0
        };
        let analysis_output_group_count =
            if is_analyzer && !is_instrument && layout.audio_outputs == 0 {
                1
            } else {
                0
            };
        let aux_output_group_count = if is_instrument {
            0
        } else {
            extra_output_groups
        };
        let multi_output_instrument = is_instrument && instrument_output_group_count > 0;

        let bus_capable_fx_class = if !is_fx {
            None
        } else if secondary_input_group_count > 0 && aux_output_group_count > 0 {
            Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
        } else if secondary_input_group_count > 0 {
            Some(RuntimePluginBusCapableFxClass::SidechainCapableFx)
        } else if aux_output_group_count > 1 {
            Some(RuntimePluginBusCapableFxClass::MultiStemFx)
        } else if aux_input_group_count > 0 || aux_output_group_count > 0 {
            Some(RuntimePluginBusCapableFxClass::ParallelCapableFx)
        } else {
            Some(RuntimePluginBusCapableFxClass::SinglePathFx)
        };

        let has_complex_topology = multi_output_instrument
            || secondary_input_group_count > 0
            || aux_input_group_count > 0
            || aux_output_group_count > 0
            || analysis_output_group_count > 0;

        let attachment_policy = if has_complex_topology {
            RuntimePluginTopologyAttachmentPolicy::Optional
        } else {
            RuntimePluginTopologyAttachmentPolicy::Required
        };
        let fallback_outcome = if multi_output_instrument {
            RuntimePluginTopologyFallbackOutcome::CollapseToPrimaryPath
        } else if secondary_input_group_count > 0 {
            RuntimePluginTopologyFallbackOutcome::SafeModeDegradation
        } else if has_complex_topology {
            RuntimePluginTopologyFallbackOutcome::BypassUnavailablePortGroup
        } else {
            RuntimePluginTopologyFallbackOutcome::TerminalPluginTopologyFailure
        };

        let mut declared_port_classes = Vec::new();
        if main_input_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::MainInput);
        }
        if main_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::MainOutput);
        }
        if secondary_input_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::SecondaryInput);
        }
        if aux_input_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::AuxInput);
        }
        if aux_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::AuxOutput);
        }
        if instrument_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::InstrumentOutput);
        }
        if analysis_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::AnalysisOutput);
        }

        let port_group_count = main_input_group_count
            + main_output_group_count
            + secondary_input_group_count
            + aux_input_group_count
            + aux_output_group_count
            + instrument_output_group_count
            + analysis_output_group_count;

        let summary = format!(
            "complex={} classes={:?} groups={} main_in={} main_out={} secondary_in={} aux_in={} aux_out={} instrument_out={} analysis_out={} multi_output_instrument={} fx_class={:?} attachment={:?} fallback={:?}",
            has_complex_topology,
            declared_port_classes,
            port_group_count,
            main_input_group_count,
            main_output_group_count,
            secondary_input_group_count,
            aux_input_group_count,
            aux_output_group_count,
            instrument_output_group_count,
            analysis_output_group_count,
            multi_output_instrument,
            bus_capable_fx_class,
            attachment_policy,
            fallback_outcome,
        );

        Self {
            has_complex_topology,
            declared_port_classes,
            port_group_count,
            main_input_group_count,
            main_output_group_count,
            secondary_input_group_count,
            aux_input_group_count,
            aux_output_group_count,
            instrument_output_group_count,
            analysis_output_group_count,
            multi_output_instrument,
            bus_capable_fx_class,
            attachment_policy,
            fallback_outcome,
            summary,
        }
    }
}

pub(crate) fn runtime_bus_intents_for_topology_role(
    topology_role: GraphNodeTopologyRole,
) -> (RuntimeBusIntent, RuntimeBusIntent) {
    match topology_role {
        GraphNodeTopologyRole::Utility => {
            (RuntimeBusIntent::AnalysisTap, RuntimeBusIntent::AnalysisTap)
        }
        GraphNodeTopologyRole::TrackLane
        | GraphNodeTopologyRole::Bus
        | GraphNodeTopologyRole::ConsoleNode => {
            (RuntimeBusIntent::MainProgram, RuntimeBusIntent::MainProgram)
        }
        GraphNodeTopologyRole::Send => (RuntimeBusIntent::MainProgram, RuntimeBusIntent::AuxSend),
        GraphNodeTopologyRole::Return => {
            (RuntimeBusIntent::AuxReturn, RuntimeBusIntent::MainProgram)
        }
    }
}

pub(crate) fn runtime_bus_role_for_endpoint(
    topology_role: GraphNodeTopologyRole,
    bus_intent: RuntimeBusIntent,
) -> RuntimeBusRole {
    match bus_intent {
        RuntimeBusIntent::AuxSend => RuntimeBusRole::AuxSend,
        RuntimeBusIntent::AuxReturn => RuntimeBusRole::AuxReturn,
        RuntimeBusIntent::Sidechain => RuntimeBusRole::AuxSend,
        RuntimeBusIntent::AnalysisTap => RuntimeBusRole::AnalysisTap,
        RuntimeBusIntent::HardwareInput => RuntimeBusRole::HardwareIngress,
        RuntimeBusIntent::HardwareOutput => RuntimeBusRole::HardwareEgress,
        RuntimeBusIntent::MainProgram => match topology_role {
            GraphNodeTopologyRole::Bus => RuntimeBusRole::Submix,
            GraphNodeTopologyRole::Utility => RuntimeBusRole::AnalysisTap,
            GraphNodeTopologyRole::Send => RuntimeBusRole::AuxSend,
            GraphNodeTopologyRole::Return => RuntimeBusRole::AuxReturn,
            GraphNodeTopologyRole::TrackLane | GraphNodeTopologyRole::ConsoleNode => {
                RuntimeBusRole::ProgramMain
            }
        },
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LoopRegion {
    pub start_samples: i64,
    pub end_samples: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransportProjection {
    pub playing: bool,
    pub timeline_position_samples: i64,
    pub tempo_bpm: f64,
    pub loop_state: Option<LoopRegion>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParameterEvent {
    pub target: String,
    pub sample_offset: usize,
    pub normalized_value: f32,
}

/// Runtime-owned batch of parameter changes accepted for one automation epoch.
///
/// Runtime stays authoritative for epoch assignment and block-boundary
/// application; callers supply only the logical event payload.
#[derive(Clone, Debug, PartialEq)]
pub struct ParameterBatch {
    pub epoch: u64,
    pub events: Vec<ParameterEvent>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeAutomationInterpolation {
    Hold,
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationResolution {
    pub ramp_step_samples: usize,
    pub max_sub_blocks: usize,
}

impl Default for RuntimeAutomationResolution {
    fn default() -> Self {
        Self {
            ramp_step_samples: 32,
            max_sub_blocks: 8,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAutomationTargetProjection {
    pub node_id: String,
    pub parameter_id: String,
}

impl RuntimeAutomationTargetProjection {
    pub fn parameter_path(&self) -> String {
        format!("{}.{}", self.node_id, self.parameter_id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationPointProjection {
    pub time_samples: i64,
    pub normalized_value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeAutomationLaneProjection {
    pub automation_lane_id: String,
    pub target: RuntimeAutomationTargetProjection,
    pub base_normalized_value: f32,
    pub interpolation: RuntimeAutomationInterpolation,
    pub resolution: RuntimeAutomationResolution,
    pub point_count: usize,
    pub points: Vec<RuntimeAutomationPointProjection>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeAutomationProjection {
    pub lane_count: usize,
    pub point_count: usize,
    pub lanes: Vec<RuntimeAutomationLaneProjection>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionReceipt {
    pub accepted_epoch: u64,
    pub applied_at_block_boundary: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DegradedReason(pub &'static str);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFaultKind {
    Timeout,
    Crash,
    ProtocolViolation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeWatchdogTrigger {
    DeadlineMisses,
    HeartbeatMisses,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WatchdogRestartRecord {
    pub sandbox_id: String,
    pub trigger: RuntimeWatchdogTrigger,
    pub processing_epoch: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSupervisionSnapshot {
    pub watchdog_restart_count: u32,
    pub safe_mode_enabled: bool,
    pub xrun_overload_active: bool,
    pub last_watchdog_trigger: Option<RuntimeWatchdogTrigger>,
    pub last_sandbox_id: Option<String>,
    pub last_processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTimelineSnapshot {
    pub next_block_sequence: u64,
    pub block_sequence_continuity: BlockSequenceContinuityReport,
    pub transport_epoch: u64,
    pub last_transport_transition: Option<RuntimeTransportTransitionKind>,
    pub last_transport_transition_processing_epoch: Option<u64>,
    pub last_transport_transition_block_sequence: Option<u64>,
    pub last_transport_playing: Option<bool>,
    pub last_transport_tempo_bpm: Option<f64>,
    pub last_transport_timeline_position_samples: Option<i64>,
    pub last_transport_loop_start_samples: Option<i64>,
    pub last_transport_loop_end_samples: Option<i64>,
    pub last_engine_block_start_samples: Option<i64>,
    pub last_engine_block_end_samples: Option<i64>,
    pub loop_wrap_count: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTransportObservationSnapshot {
    pub transport_epoch: u64,
    pub projected_playing: Option<bool>,
    pub projected_tempo_bpm: Option<f64>,
    pub projected_timeline_position_samples: Option<i64>,
    pub projected_loop_start_samples: Option<i64>,
    pub projected_loop_end_samples: Option<i64>,
    pub observed_playing: Option<bool>,
    pub observed_tempo_bpm: Option<f64>,
    pub observed_timeline_position_samples: Option<i64>,
    pub observed_loop_start_samples: Option<i64>,
    pub observed_loop_end_samples: Option<i64>,
    pub last_transition: Option<RuntimeTransportTransitionKind>,
    pub last_transition_processing_epoch: Option<u64>,
    pub last_transition_block_sequence: Option<u64>,
    pub last_engine_block_start_samples: Option<i64>,
    pub last_engine_block_end_samples: Option<i64>,
    pub loop_wrap_count: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureState {
    Idle,
    Capturing,
    Failed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureKind {
    Audio,
    Midi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeRecordingCaptureCheckpointClass {
    Armed,
    Streaming,
    Buffered,
    Committed,
    Failed,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCheckpointSnapshot {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub checkpoint_class: RuntimeRecordingCaptureCheckpointClass,
    pub interruption_class: RuntimeInterruptionClass,
    pub take_id: String,
    pub track_id: String,
    pub capture_start_samples: i64,
    pub capture_path: String,
    pub buffered_block_count: u64,
    pub buffered_frame_count: u64,
    pub buffered_event_count: u64,
    pub captured_channel_count: usize,
    pub peak_level: Option<f32>,
    pub pressure_event_count: u64,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRecordingCaptureStartRequest {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub capture_path: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeRecordingCaptureCommitReceipt {
    pub capture_kind: RuntimeRecordingCaptureKind,
    pub take_id: String,
    pub track_id: String,
    pub start_samples: i64,
    pub duration_samples: u32,
    pub channel_count: usize,
    pub peak_level: f32,
    pub capture_path: String,
    pub committed_checkpoint: RuntimeRecordingCaptureCheckpointSnapshot,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRecordingCaptureSnapshot {
    pub capture_ready: bool,
    pub state: Option<RuntimeRecordingCaptureState>,
    pub capture_kind: Option<RuntimeRecordingCaptureKind>,
    pub active_take_id: Option<String>,
    pub active_track_id: Option<String>,
    pub capture_start_samples: Option<i64>,
    pub active_capture_path: Option<String>,
    pub buffered_block_count: u64,
    pub buffered_frame_count: u64,
    pub buffered_event_count: u64,
    pub captured_channel_count: usize,
    pub peak_level: Option<f32>,
    pub pressure_event_count: u64,
    pub active_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    pub last_checkpoint: Option<RuntimeRecordingCaptureCheckpointSnapshot>,
    pub last_committed_take_id: Option<String>,
    pub last_committed_path: Option<String>,
    pub last_committed_duration_samples: Option<u32>,
    pub last_error: Option<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceClass {
    OfflineRenderQueue,
    OfflineRenderPurge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceDecision {
    Run,
    Defer,
    Throttle,
    Abort,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServicePriorityBand {
    RealtimeCritical,
    RecoveryCritical,
    UserVisible,
    Maintenance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceReason {
    Ready,
    RealtimeActive,
    PendingCleanup,
    RecoveryDegraded,
    SafeMode,
    InvalidRequest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceBackpressureSource {
    RealtimeAudio,
    RecoveryOverlap,
    CleanupBacklog,
    SafeMode,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeDeferredServiceCancellationCause {
    InvalidRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeDeferredServiceReceipt {
    pub work_class: RuntimeDeferredServiceClass,
    pub decision: RuntimeDeferredServiceDecision,
    pub reason: RuntimeDeferredServiceReason,
    pub priority_band: RuntimeDeferredServicePriorityBand,
    pub blocking_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    pub backpressure_source: Option<RuntimeDeferredServiceBackpressureSource>,
    pub starvation_risk: bool,
    pub starved_work_item_count: usize,
    pub cancellation_cause: Option<RuntimeDeferredServiceCancellationCause>,
    pub cancelled_work_item_count: usize,
    pub interruption_class: RuntimeInterruptionClass,
    pub interruption_rebindable: bool,
    pub queued_work_item_count: usize,
    pub admitted_work_item_count: usize,
    pub completed_work_item_count: usize,
    pub deferred_work_item_count: usize,
    pub runtime_running: bool,
    pub safe_mode_enabled: bool,
    pub readiness_degraded: bool,
    pub pending_cleanup_work_items: usize,
    pub pending_deferred_retry_work_items: usize,
    pub recovery_overlap_session_count: usize,
    pub summary: String,
}

impl RuntimeDeferredServiceReceipt {
    pub fn render_multiline(&self) -> String {
        format!(
            concat!(
                "work_class={:?}",
                "\ndecision={:?}",
                "\nreason={:?}",
                "\npriority_band={:?}",
                "\nblocking_priority_band={:?}",
                "\nbackpressure_source={:?}",
                "\nstarvation_risk={}",
                "\nstarved_work_item_count={}",
                "\ncancellation_cause={:?}",
                "\ncancelled_work_item_count={}",
                "\ninterruption_class={:?}",
                "\ninterruption_rebindable={}",
                "\nqueued_work_item_count={}",
                "\nadmitted_work_item_count={}",
                "\ncompleted_work_item_count={}",
                "\ndeferred_work_item_count={}",
                "\nruntime_running={}",
                "\nsafe_mode_enabled={}",
                "\nreadiness_degraded={}",
                "\npending_cleanup_work_items={}",
                "\npending_deferred_retry_work_items={}",
                "\nrecovery_overlap_session_count={}",
                "\nsummary={}",
            ),
            self.work_class,
            self.decision,
            self.reason,
            self.priority_band,
            self.blocking_priority_band,
            self.backpressure_source,
            self.starvation_risk,
            self.starved_work_item_count,
            self.cancellation_cause,
            self.cancelled_work_item_count,
            self.interruption_class,
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            self.summary,
        )
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"work_class\":{},",
                "\"decision\":{},",
                "\"reason\":{},",
                "\"priority_band\":{},",
                "\"blocking_priority_band\":{},",
                "\"backpressure_source\":{},",
                "\"starvation_risk\":{},",
                "\"starved_work_item_count\":{},",
                "\"cancellation_cause\":{},",
                "\"cancelled_work_item_count\":{},",
                "\"interruption_class\":{},",
                "\"interruption_rebindable\":{},",
                "\"queued_work_item_count\":{},",
                "\"admitted_work_item_count\":{},",
                "\"completed_work_item_count\":{},",
                "\"deferred_work_item_count\":{},",
                "\"runtime_running\":{},",
                "\"safe_mode_enabled\":{},",
                "\"readiness_degraded\":{},",
                "\"pending_cleanup_work_items\":{},",
                "\"pending_deferred_retry_work_items\":{},",
                "\"recovery_overlap_session_count\":{},",
                "\"summary\":{}",
                "}}"
            ),
            json_string(&format!("{:?}", self.work_class)),
            json_string(&format!("{:?}", self.decision)),
            json_string(&format!("{:?}", self.reason)),
            json_string(&format!("{:?}", self.priority_band)),
            json_option_string(
                self.blocking_priority_band
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            json_option_string(
                self.backpressure_source
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.starvation_risk,
            self.starved_work_item_count,
            json_option_string(
                self.cancellation_cause
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.cancelled_work_item_count,
            json_string(&format!("{:?}", self.interruption_class)),
            self.interruption_rebindable,
            self.queued_work_item_count,
            self.admitted_work_item_count,
            self.completed_work_item_count,
            self.deferred_work_item_count,
            self.runtime_running,
            self.safe_mode_enabled,
            self.readiness_degraded,
            self.pending_cleanup_work_items,
            self.pending_deferred_retry_work_items,
            self.recovery_overlap_session_count,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

impl Default for RuntimeDeferredServiceReceipt {
    fn default() -> Self {
        Self {
            work_class: RuntimeDeferredServiceClass::OfflineRenderQueue,
            decision: RuntimeDeferredServiceDecision::Abort,
            reason: RuntimeDeferredServiceReason::InvalidRequest,
            priority_band: RuntimeDeferredServicePriorityBand::UserVisible,
            blocking_priority_band: None,
            backpressure_source: None,
            starvation_risk: false,
            starved_work_item_count: 0,
            cancellation_cause: Some(RuntimeDeferredServiceCancellationCause::InvalidRequest),
            cancelled_work_item_count: 0,
            interruption_class: RuntimeInterruptionClass::Terminal,
            interruption_rebindable: false,
            queued_work_item_count: 0,
            admitted_work_item_count: 0,
            completed_work_item_count: 0,
            deferred_work_item_count: 0,
            runtime_running: false,
            safe_mode_enabled: false,
            readiness_degraded: false,
            pending_cleanup_work_items: 0,
            pending_deferred_retry_work_items: 0,
            recovery_overlap_session_count: 0,
            summary: "class=OfflineRenderQueue decision=Abort reason=InvalidRequest queued=0 admitted=0 completed=0 deferred=0".to_string(),
        }
    }
}

/// Runtime control-plane summary.
///
/// Callers typically pair this with `RuntimeReadiness` and
/// `EffectiveRuntimeConfig` to decide whether the runtime has been handshaken,
/// configured, started, or restarted and which control request most recently
/// changed that state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeControlSnapshot {
    pub handshaken: bool,
    pub configured: bool,
    pub running: bool,
    pub handshake_count: u64,
    pub configure_count: u64,
    pub start_count: u64,
    pub stop_count: u64,
    pub restart_count: u64,
    pub last_client_version: Option<String>,
    pub last_stop_reason: Option<StopReason>,
    pub last_reconfigure: Option<RuntimeConfigRequest>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSchedulerState {
    #[default]
    Stopped,
    Configured,
    ReadyIdle,
    RealtimeOnly,
    Anticipative,
    Degraded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeExecutionPhase {
    #[default]
    Idle,
    Priming,
    Prework,
    Realtime,
    Degraded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSchedulerSnapshot {
    pub state: RuntimeSchedulerState,
    pub phase: RuntimeExecutionPhase,
    pub graph_applied: bool,
    pub schedule_applied: bool,
    pub transport_projected: bool,
    pub anticipative_enabled: bool,
    pub active_graph_id: Option<String>,
    pub phase_count: usize,
    pub lane_count: usize,
    pub dispatch_count: usize,
    pub pending_prework_target_count: usize,
    pub processed_block_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeReadiness {
    Starting,
    Ready,
    Degraded { reasons: Vec<DegradedReason> },
    Stopped,
    Failed { fatal: RuntimeError },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveRuntimeConfig {
    pub sample_rate: SampleRate,
    pub block_size: usize,
    pub anticipative_enabled: bool,
    pub safe_mode_enabled: bool,
    pub active_output_device: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub cpu_load_percent: f32,
    pub xruns: u64,
    pub graph_latency_ms: f32,
    pub active_plugin_sandboxes: u32,
    pub backend_policy_tier: BackendPolicyTier,
    pub topology_compatible: bool,
    pub topology_issue_count: usize,
    pub degraded_bound_plugin_sandboxes: usize,
    pub missing_bound_plugin_sandboxes: usize,
    pub last_output_peak: Option<f32>,
    pub last_output_rms: Option<f32>,
    pub momentary_loudness_lufs: Option<f32>,
    pub short_term_loudness_lufs: Option<f32>,
    pub integrated_loudness_lufs: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultCause {
    XrunOverload,
    PluginFault,
    WatchdogRestart,
    DeviceLoss,
    TransportFault,
    MissingPluginBinding,
    RuntimeError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticFamily {
    XrunPressure,
    CallbackPressure,
    PluginBoundaryFault,
    DevicePathFault,
    DeferredWorkPressure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticAuthority {
    RuntimeCanonical,
    HostAdvisory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultContributionReceipt {
    pub family: RuntimeFaultDiagnosticFamily,
    pub authority: RuntimeFaultDiagnosticAuthority,
    pub active: bool,
    pub event_count: u64,
    pub detail: Option<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeMeterSourceRole {
    Utility,
    TrackLane,
    Bus,
    Send,
    Return,
    ConsoleNode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeterSourceSnapshot {
    pub bus_id: String,
    pub topology_role: RuntimeMeterSourceRole,
    pub track_lane_id: Option<String>,
    pub bus_group_id: Option<String>,
    pub console_group_id: Option<String>,
    pub send_return_id: Option<String>,
    pub producer_node_ids: Vec<String>,
    pub peak_level: f32,
    pub rms_level: f32,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeMeteringSnapshot {
    pub meter_count: usize,
    pub main_output_peak_level: Option<f32>,
    pub main_output_rms_level: Option<f32>,
    pub momentary_loudness_lufs: Option<f32>,
    pub short_term_loudness_lufs: Option<f32>,
    pub integrated_loudness_lufs: Option<f32>,
    pub clipped_sample_count: u64,
    pub meters: Vec<RuntimeMeterSourceSnapshot>,
    pub track_lanes: Vec<RuntimeTrackLaneMeterSummary>,
    pub bus_groups: Vec<RuntimeBusGroupMeterSummary>,
    pub console_groups: Vec<RuntimeConsoleGroupMeterSummary>,
    pub send_returns: Vec<RuntimeSendReturnMeterSummary>,
    pub bus_connection_count: usize,
    pub auxiliary_path_count: usize,
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeRoutedMeterAggregate {
    pub meter_count: usize,
    pub metered_bus_ids: Vec<String>,
    pub producer_node_ids: Vec<String>,
    pub peak_level: Option<f32>,
    pub rms_level: Option<f32>,
    pub latency_samples: u32,
    pub tail_samples: u32,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeTrackLaneMeterSummary {
    pub track_lane_id: String,
    pub bus_group_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeBusGroupMeterSummary {
    pub bus_group_id: String,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeConsoleGroupMeterSummary {
    pub console_group_id: String,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RuntimeSendReturnMeterSummary {
    pub send_return_id: String,
    pub send_node_ids: Vec<String>,
    pub return_node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub aggregate: RuntimeRoutedMeterAggregate,
}

impl RuntimeMeteringSnapshot {
    pub fn with_execution_topology(mut self, topology: &RuntimeExecutionTopologySummary) -> Self {
        self.track_lanes = topology
            .track_lanes
            .iter()
            .map(|track_lane| RuntimeTrackLaneMeterSummary {
                track_lane_id: track_lane.track_lane_id.clone(),
                bus_group_ids: track_lane.bus_group_ids.clone(),
                input_bus_ids: track_lane.input_bus_ids.clone(),
                output_bus_ids: track_lane.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.track_lane_id.as_deref() == Some(track_lane.track_lane_id.as_str())
                    }),
                    format!("track_lane={}", track_lane.track_lane_id),
                ),
            })
            .collect();
        self.bus_groups = topology
            .bus_groups
            .iter()
            .map(|bus_group| RuntimeBusGroupMeterSummary {
                bus_group_id: bus_group.bus_group_id.clone(),
                topology_roles: bus_group.topology_roles.clone(),
                node_ids: bus_group.node_ids.clone(),
                input_bus_ids: bus_group.input_bus_ids.clone(),
                output_bus_ids: bus_group.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.bus_group_id.as_deref() == Some(bus_group.bus_group_id.as_str())
                    }),
                    format!("bus_group={}", bus_group.bus_group_id),
                ),
            })
            .collect();
        self.console_groups = topology
            .console_groups
            .iter()
            .map(|console_group| RuntimeConsoleGroupMeterSummary {
                console_group_id: console_group.console_group_id.clone(),
                node_ids: console_group.node_ids.clone(),
                input_bus_ids: console_group.input_bus_ids.clone(),
                output_bus_ids: console_group.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.console_group_id.as_deref()
                            == Some(console_group.console_group_id.as_str())
                    }),
                    format!("console_group={}", console_group.console_group_id),
                ),
            })
            .collect();
        self.send_returns = topology
            .send_returns
            .iter()
            .map(|send_return| RuntimeSendReturnMeterSummary {
                send_return_id: send_return.send_return_id.clone(),
                send_node_ids: send_return.send_node_ids.clone(),
                return_node_ids: send_return.return_node_ids.clone(),
                input_bus_ids: send_return.input_bus_ids.clone(),
                output_bus_ids: send_return.output_bus_ids.clone(),
                aggregate: aggregate_runtime_meter_sources(
                    self.meters.iter().filter(|meter| {
                        meter.send_return_id.as_deref() == Some(send_return.send_return_id.as_str())
                    }),
                    format!("send_return={}", send_return.send_return_id),
                ),
            })
            .collect();
        self.bus_connection_count = topology.bus_connection_count;
        self.auxiliary_path_count = topology.auxiliary_path_count;
        self.bus_connections = topology.bus_connections.clone();
        self.auxiliary_paths = topology.auxiliary_paths.clone();
        self.summary = format!(
            "meters={} main_peak={:?} main_rms={:?} momentary_lufs={:?} short_term_lufs={:?} integrated_lufs={:?} clipped={} routes={}/{}/{}/{} bus_connections={} auxiliary_paths={}",
            self.meter_count,
            self.main_output_peak_level,
            self.main_output_rms_level,
            self.momentary_loudness_lufs,
            self.short_term_loudness_lufs,
            self.integrated_loudness_lufs,
            self.clipped_sample_count,
            self.track_lanes.len(),
            self.bus_groups.len(),
            self.send_returns.len(),
            self.console_groups.len(),
            self.bus_connection_count,
            self.auxiliary_path_count,
        );
        self
    }
}

fn aggregate_runtime_meter_sources<'a>(
    meters: impl Iterator<Item = &'a RuntimeMeterSourceSnapshot>,
    scope: String,
) -> RuntimeRoutedMeterAggregate {
    let mut aggregate = RuntimeRoutedMeterAggregate::default();

    for meter in meters {
        aggregate.meter_count += 1;
        if !aggregate.metered_bus_ids.contains(&meter.bus_id) {
            aggregate.metered_bus_ids.push(meter.bus_id.clone());
        }
        for producer_node_id in &meter.producer_node_ids {
            if !aggregate.producer_node_ids.contains(producer_node_id) {
                aggregate.producer_node_ids.push(producer_node_id.clone());
            }
        }
        aggregate.peak_level = Some(match aggregate.peak_level {
            Some(peak_level) => peak_level.max(meter.peak_level),
            None => meter.peak_level,
        });
        aggregate.rms_level = Some(match aggregate.rms_level {
            Some(rms_level) => rms_level.max(meter.rms_level),
            None => meter.rms_level,
        });
        aggregate.latency_samples = aggregate.latency_samples.max(meter.latency_samples);
        aggregate.tail_samples = aggregate.tail_samples.max(meter.tail_samples);
    }

    aggregate.summary = format!(
        "{scope} meters={} peak={:?} rms={:?} buses={:?} producers={}",
        aggregate.meter_count,
        aggregate.peak_level,
        aggregate.rms_level,
        aggregate.metered_bus_ids,
        aggregate.producer_node_ids.len(),
    );
    aggregate
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveryRecord {
    pub sandbox_id: String,
    pub intent: RecoveryRestartIntent,
    pub stop_reason: StopReason,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxLifecycleStage {
    SandboxEnsured,
    SandboxHandshaken,
    PluginTypeLoaded,
    InstanceCreated,
    InstancePrepared,
    TransportAttached,
    InstanceActivated,
    InstanceDeactivated,
    InstanceReset,
    InstanceDestroyed,
    SandboxTeardown,
    TransportTornDown,
    SandboxRestarted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxLifecycleRecord {
    pub sandbox_id: String,
    pub stage: PluginSandboxLifecycleStage,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxTransportStage {
    Attached,
    DetachRequested,
    Detached,
    DetachFault,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxTransportRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub stage: PluginSandboxTransportStage,
    pub processing_epoch: Option<u64>,
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceFaultRecord {
    pub kind: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxInstanceStateRecord {
    pub sandbox_id: String,
    pub plugin_type_id: String,
    pub instance_id: String,
    pub lifecycle_state: String,
    pub readiness_state: String,
    pub degraded_reasons: Vec<String>,
    pub active: bool,
    pub processing_epoch: Option<u64>,
    pub processing_sample_rate_hz: Option<u32>,
    pub processing_max_block_frames: Option<u32>,
    pub audio_inputs: Option<u16>,
    pub audio_outputs: Option<u16>,
    pub midi_inputs: Option<u16>,
    pub midi_outputs: Option<u16>,
    pub last_fault: Option<PluginSandboxInstanceFaultRecord>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HeartbeatCycleStage {
    Requested,
    Responded,
    Missed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeartbeatCycleRecord {
    pub sandbox_id: String,
    pub stage: HeartbeatCycleStage,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockDispatchStage {
    Requested,
    Completed,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockDispatchRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub frame_count: u32,
    pub stage: BlockDispatchStage,
    pub completion_state: Option<CompletionState>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaseRolloverRecord {
    pub sandbox_id: String,
    pub previous_lease_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub first_block_sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerInvalidationStage {
    CompletionRegionInvalidated,
    LeaseEpochInvalidated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerInvalidationRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: Option<u64>,
    pub stage: BrokerInvalidationStage,
    pub reason: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompletionSlotStage {
    ReadyForProcessing,
    Processing,
    Completed,
    TimedOut,
    Invalidated,
    FallbackApplied,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionSlotRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub processing_epoch: u64,
    pub block_sequence: u64,
    pub stage: CompletionSlotStage,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrokerFailureStage {
    PreparePlanCreate,
    PayloadWrite,
    PayloadRead,
    TransportDestroy,
    TransportTeardown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrokerFailureRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
    pub stage: BrokerFailureStage,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxOperationFailureStage {
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SandboxOperationFailureRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub operation: String,
    pub error_kind: String,
    pub stage: SandboxOperationFailureStage,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultSource {
    HostBroker,
    SandboxOperation,
    RuntimeDispatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultStage {
    PreparePlanCreate,
    PayloadWrite,
    PayloadRead,
    TransportDestroy,
    TransportTeardown,
    TransportDetachRequested,
    TransportDetached,
    PrepareAttach,
    ProcessAttach,
    ProcessFlush,
    ProcessProtocolViolation,
    ControlProtocolViolation,
    TransportDetachFault,
    CompletionRegionInvalidated,
    LeaseEpochInvalidated,
    CompletionSlotTimedOut,
    CompletionSlotInvalidated,
    FallbackApplied,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultPhase {
    Prepare,
    Dispatch,
    Teardown,
    Control,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultResource {
    PreparePlan,
    SharedMemoryPayload,
    SharedMemoryLease,
    CompletionSlot,
    ProcessProtocol,
    ControlProtocol,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultRecord {
    pub sandbox_id: String,
    pub lease_id: Option<String>,
    pub processing_epoch: Option<u64>,
    pub block_sequence: Option<u64>,
    pub source: TransportFaultSource,
    pub stage: TransportFaultStage,
    pub phase: TransportFaultPhase,
    pub resource: TransportFaultResource,
    pub operation: String,
    pub error_kind: Option<String>,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportFaultBoundaryMode {
    FaultAdjacentOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportFaultSummary {
    pub boundary_mode: TransportFaultBoundaryMode,
    pub total_events: usize,
    pub host_broker_events: usize,
    pub sandbox_operation_events: usize,
    pub runtime_dispatch_events: usize,
    pub prepare_events: usize,
    pub dispatch_events: usize,
    pub teardown_events: usize,
    pub control_events: usize,
    pub first_processing_epoch: Option<u64>,
    pub last_processing_epoch: Option<u64>,
    pub first_block_sequence: Option<u64>,
    pub last_block_sequence: Option<u64>,
}

impl TransportFaultSummary {
    pub fn from_records(records: &[TransportFaultRecord]) -> Self {
        let mut summary = Self {
            boundary_mode: TransportFaultBoundaryMode::FaultAdjacentOnly,
            total_events: records.len(),
            host_broker_events: 0,
            sandbox_operation_events: 0,
            runtime_dispatch_events: 0,
            prepare_events: 0,
            dispatch_events: 0,
            teardown_events: 0,
            control_events: 0,
            first_processing_epoch: None,
            last_processing_epoch: None,
            first_block_sequence: None,
            last_block_sequence: None,
        };

        for record in records {
            match record.source {
                TransportFaultSource::HostBroker => {
                    summary.host_broker_events = summary.host_broker_events.saturating_add(1)
                }
                TransportFaultSource::SandboxOperation => {
                    summary.sandbox_operation_events =
                        summary.sandbox_operation_events.saturating_add(1)
                }
                TransportFaultSource::RuntimeDispatch => {
                    summary.runtime_dispatch_events =
                        summary.runtime_dispatch_events.saturating_add(1)
                }
            }
            match record.phase {
                TransportFaultPhase::Prepare => {
                    summary.prepare_events = summary.prepare_events.saturating_add(1)
                }
                TransportFaultPhase::Dispatch => {
                    summary.dispatch_events = summary.dispatch_events.saturating_add(1)
                }
                TransportFaultPhase::Teardown => {
                    summary.teardown_events = summary.teardown_events.saturating_add(1)
                }
                TransportFaultPhase::Control => {
                    summary.control_events = summary.control_events.saturating_add(1)
                }
            }

            if let Some(epoch) = record.processing_epoch {
                summary.first_processing_epoch = Some(
                    summary
                        .first_processing_epoch
                        .map_or(epoch, |current| current.min(epoch)),
                );
                summary.last_processing_epoch = Some(
                    summary
                        .last_processing_epoch
                        .map_or(epoch, |current| current.max(epoch)),
                );
            }
            if let Some(block_sequence) = record.block_sequence {
                summary.first_block_sequence = Some(
                    summary
                        .first_block_sequence
                        .map_or(block_sequence, |current| current.min(block_sequence)),
                );
                summary.last_block_sequence = Some(
                    summary
                        .last_block_sequence
                        .map_or(block_sequence, |current| current.max(block_sequence)),
                );
            }
        }

        summary
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionBoundaryMode {
    HealthyPathVisible,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionState {
    Detached,
    AttachActive,
    DetachRequested,
    DetachFaulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportHeartbeatFreshness {
    Unknown,
    Requested,
    Fresh,
    Missed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportDispatchState {
    Idle,
    Requested,
    Completed,
    TimedOut,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTransportSessionRecord {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub processing_epoch: Option<u64>,
    pub active_block_sequence: Option<u64>,
    pub transport_fault_count: usize,
    pub last_transport_fault_source: Option<TransportFaultSource>,
    pub last_transport_fault_stage: Option<TransportFaultStage>,
    pub last_transport_fault_phase: Option<TransportFaultPhase>,
    pub last_transport_fault_processing_epoch: Option<u64>,
    pub last_transport_fault_block_sequence: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransportSessionSummary {
    pub boundary_mode: TransportSessionBoundaryMode,
    pub current_state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub current_attached_session_count: usize,
    pub max_concurrent_attached_sessions: usize,
    pub attach_events: usize,
    pub detach_requested_events: usize,
    pub detached_events: usize,
    pub detach_fault_events: usize,
    pub heartbeat_requested_events: usize,
    pub heartbeat_responded_events: usize,
    pub heartbeat_missed_events: usize,
    pub dispatch_requested_events: usize,
    pub dispatch_completed_events: usize,
    pub dispatch_timed_out_events: usize,
    pub first_processing_epoch: Option<u64>,
    pub last_processing_epoch: Option<u64>,
    pub first_block_sequence: Option<u64>,
    pub last_block_sequence: Option<u64>,
    pub active_sandbox_id: Option<String>,
    pub active_lease_id: Option<String>,
    pub active_region_id: Option<String>,
    pub active_block_sequence: Option<u64>,
    pub active_sessions: Vec<ActiveTransportSessionRecord>,
    pub last_sandbox_id: Option<String>,
    pub last_lease_id: Option<String>,
    pub last_region_id: Option<String>,
}

impl TransportSessionSummary {
    pub fn from_diagnostics(diagnostics: &RuntimeObservationDiagnostics) -> Self {
        let mut summary = Self {
            boundary_mode: TransportSessionBoundaryMode::HealthyPathVisible,
            current_state: TransportSessionState::Detached,
            currently_attached: false,
            heartbeat_freshness: TransportHeartbeatFreshness::Unknown,
            dispatch_state: TransportDispatchState::Idle,
            current_attached_session_count: 0,
            max_concurrent_attached_sessions: 0,
            attach_events: 0,
            detach_requested_events: 0,
            detached_events: 0,
            detach_fault_events: 0,
            heartbeat_requested_events: 0,
            heartbeat_responded_events: 0,
            heartbeat_missed_events: 0,
            dispatch_requested_events: 0,
            dispatch_completed_events: 0,
            dispatch_timed_out_events: 0,
            first_processing_epoch: None,
            last_processing_epoch: None,
            first_block_sequence: None,
            last_block_sequence: None,
            active_sandbox_id: None,
            active_lease_id: None,
            active_region_id: None,
            active_block_sequence: None,
            active_sessions: Vec::new(),
            last_sandbox_id: None,
            last_lease_id: None,
            last_region_id: None,
        };
        let mut active_sessions: BTreeMap<(String, String, String), ActiveTransportSessionRecord> =
            BTreeMap::new();
        let mut last_transport_key: Option<(String, String, String)> = None;

        for record in &diagnostics.transport_events {
            match record.stage {
                PluginSandboxTransportStage::Attached => {
                    summary.attach_events = summary.attach_events.saturating_add(1)
                }
                PluginSandboxTransportStage::DetachRequested => {
                    summary.detach_requested_events =
                        summary.detach_requested_events.saturating_add(1)
                }
                PluginSandboxTransportStage::Detached => {
                    summary.detached_events = summary.detached_events.saturating_add(1)
                }
                PluginSandboxTransportStage::DetachFault => {
                    summary.detach_fault_events = summary.detach_fault_events.saturating_add(1)
                }
            }
            update_transport_session_epoch_bounds(&mut summary, record.processing_epoch);
            last_transport_key = Some(apply_transport_session_state(
                &mut summary,
                &mut active_sessions,
                record,
            ));
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
            summary.last_lease_id = Some(record.lease_id.clone());
            summary.last_region_id = Some(record.region_id.clone());
        }

        for record in &diagnostics.heartbeat_events {
            match record.stage {
                HeartbeatCycleStage::Requested => {
                    summary.heartbeat_requested_events =
                        summary.heartbeat_requested_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Requested;
                }
                HeartbeatCycleStage::Responded => {
                    summary.heartbeat_responded_events =
                        summary.heartbeat_responded_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Fresh;
                }
                HeartbeatCycleStage::Missed => {
                    summary.heartbeat_missed_events =
                        summary.heartbeat_missed_events.saturating_add(1);
                    summary.heartbeat_freshness = TransportHeartbeatFreshness::Missed;
                }
            }
            update_transport_session_epoch_bounds(&mut summary, record.processing_epoch);
            update_transport_session_block_bounds(&mut summary, record.block_sequence);
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                None,
                last_transport_key.as_ref(),
            ) {
                session.heartbeat_freshness = summary.heartbeat_freshness;
                session.processing_epoch = record.processing_epoch.or(session.processing_epoch);
            }
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
        }

        for record in &diagnostics.block_dispatch_events {
            match record.stage {
                BlockDispatchStage::Requested => {
                    summary.dispatch_requested_events =
                        summary.dispatch_requested_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::Requested;
                }
                BlockDispatchStage::Completed => {
                    summary.dispatch_completed_events =
                        summary.dispatch_completed_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::Completed;
                }
                BlockDispatchStage::TimedOut => {
                    summary.dispatch_timed_out_events =
                        summary.dispatch_timed_out_events.saturating_add(1);
                    summary.dispatch_state = TransportDispatchState::TimedOut;
                }
            }
            update_transport_session_epoch_bounds(&mut summary, Some(record.processing_epoch));
            update_transport_session_block_bounds(&mut summary, Some(record.block_sequence));
            summary.last_sandbox_id = Some(record.sandbox_id.clone());
            summary.last_lease_id = Some(record.lease_id.clone());
            summary.active_block_sequence = Some(record.block_sequence);
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                Some(&record.lease_id),
                last_transport_key.as_ref(),
            ) {
                session.dispatch_state = summary.dispatch_state;
                session.processing_epoch = Some(record.processing_epoch);
                session.active_block_sequence = Some(record.block_sequence);
            }
        }

        for record in &diagnostics.transport_fault_events {
            if let Some(session) = resolve_active_session_mut(
                &mut active_sessions,
                &record.sandbox_id,
                record.lease_id.as_deref(),
                last_transport_key.as_ref(),
            ) {
                session.transport_fault_count = session.transport_fault_count.saturating_add(1);
                session.last_transport_fault_source = Some(record.source);
                session.last_transport_fault_stage = Some(record.stage);
                session.last_transport_fault_phase = Some(record.phase);
                session.last_transport_fault_processing_epoch = record.processing_epoch;
                session.last_transport_fault_block_sequence = record.block_sequence;
                session.processing_epoch = record.processing_epoch.or(session.processing_epoch);
                if let Some(block_sequence) = record.block_sequence {
                    session.active_block_sequence = Some(block_sequence);
                }
            }
        }

        summary.current_attached_session_count = active_sessions.len();
        summary.active_sessions = active_sessions.into_values().collect();

        summary
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimePerformanceSnapshot {
    pub sample_rate_hz: u32,
    pub block_size: usize,
    pub processed_block_count: u64,
    pub last_block_sequence: Option<u64>,
    pub cpu_load_percent: f32,
    pub graph_latency_ms: f32,
    pub last_block_execution_time_ns: Option<u64>,
    pub last_block_deadline_budget_ns: Option<u64>,
    pub last_block_budget_utilization_percent: Option<f32>,
    pub last_block_budget_overrun_ns: Option<u64>,
    pub last_block_deadline_pressure: RuntimeBlockDeadlinePressure,
    pub budget_overrun_count: u64,
    pub peak_block_execution_time_ns: u64,
    pub peak_block_budget_utilization_percent: f32,
    pub peak_block_budget_overrun_ns: u64,
    pub xrun_count: u64,
    pub scheduler_phase_count: usize,
    pub scheduler_lane_count: usize,
    pub scheduler_dispatch_count: usize,
    pub scheduler_prepared_dispatch_count: usize,
    pub scheduler_realtime_dispatch_count: usize,
    pub scheduler_dispatch_handoff_count: usize,
    pub scheduler_topology_compatible: bool,
    pub scheduler_topology_requires_host_reinterpretation: bool,
    pub scheduler_topology_issue_count: usize,
    pub prework_service_state: RuntimePreworkServiceState,
    pub prework_service_pressure: RuntimePreworkServicePressure,
    pub prework_service_semantic_policy: RuntimePreworkServiceSemanticPolicy,
    pub pending_prework_target_count: usize,
    pub pending_prework_deferred_target_count: usize,
    pub prework_queue_depth: usize,
    pub prework_peak_queue_depth: usize,
    pub prework_service_cycle_count: u64,
    pub prework_service_starvation_count: u64,
    pub prework_service_throttle_count: u64,
    pub prework_service_yield_count: u64,
    pub last_prework_service_effective_cycles: usize,
    pub last_prework_service_budget_per_cycle: Option<usize>,
    pub last_prework_service_effective_budget_per_cycle: Option<usize>,
    pub last_prework_serviced_backlog_class: Option<String>,
    pub transport_gate_active: bool,
    pub plugin_gate_active: bool,
    pub hot_latency_node_id: Option<String>,
    pub hot_latency_node_group: Option<String>,
    pub hot_latency_node_topology_role: Option<String>,
    pub hot_latency_node_plugin_sandbox_id: Option<String>,
    pub hot_latency_node_samples: u32,
    pub hot_latency_group: Option<String>,
    pub hot_latency_group_node_count: usize,
    pub hot_latency_group_total_samples: u32,
    pub critical_path_lane: Option<String>,
    pub critical_path_lane_node_count: usize,
    pub critical_path_lane_plugin_backed_node_count: usize,
    pub critical_path_lane_planning_group_count: usize,
    pub critical_path_lane_total_latency_samples: u32,
    pub worker_lane_summaries: Vec<RuntimeWorkerLaneInstrumentationSummary>,
    pub background_service_class: Option<RuntimeDeferredServiceClass>,
    pub background_service_decision: Option<RuntimeDeferredServiceDecision>,
    pub background_service_reason: Option<RuntimeDeferredServiceReason>,
    pub background_service_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    pub background_service_blocking_priority_band: Option<RuntimeDeferredServicePriorityBand>,
    pub background_service_backpressure_source: Option<RuntimeDeferredServiceBackpressureSource>,
    pub background_service_starvation_risk: bool,
    pub background_service_starved_work_item_count: usize,
    pub background_service_cancellation_cause: Option<RuntimeDeferredServiceCancellationCause>,
    pub background_service_cancelled_work_item_count: usize,
    pub background_queued_work_item_count: usize,
    pub background_deferred_work_item_count: usize,
    pub background_pending_cleanup_work_item_count: usize,
    pub background_pending_retry_work_item_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeWorkerLaneInstrumentationSummary {
    pub lane: GraphExecutionLane,
    pub node_count: usize,
    pub plugin_backed_node_count: usize,
    pub planning_group_count: usize,
    pub total_latency_samples: u32,
    pub max_node_latency_samples: u32,
}

impl RuntimePerformanceSnapshot {
    pub fn capture(
        effective_config: &EffectiveRuntimeConfig,
        diagnostics_snapshot: &RuntimeDiagnosticsSnapshot,
        engine_block_snapshot: &RuntimeEngineBlockSnapshot,
        last_deferred_service_receipt: Option<&RuntimeDeferredServiceReceipt>,
    ) -> Self {
        let worker_lane_summaries =
            runtime_worker_lane_instrumentation_summaries(engine_block_snapshot);
        let hot_latency_node = engine_block_snapshot
            .planned_nodes
            .iter()
            .max_by_key(|node| node.latency_samples)
            .filter(|node| node.latency_samples > 0);
        let mut inline_realtime_group_total_samples = 0u32;
        let mut inline_realtime_group_node_count = 0usize;
        let mut stateful_realtime_group_total_samples = 0u32;
        let mut stateful_realtime_group_node_count = 0usize;
        let mut anticipative_group_total_samples = 0u32;
        let mut anticipative_group_node_count = 0usize;
        for node in &engine_block_snapshot.planned_nodes {
            match node.group {
                GraphNodePlanningGroup::InlineRealtime => {
                    inline_realtime_group_total_samples =
                        inline_realtime_group_total_samples.saturating_add(node.latency_samples);
                    inline_realtime_group_node_count =
                        inline_realtime_group_node_count.saturating_add(1);
                }
                GraphNodePlanningGroup::StatefulRealtime => {
                    stateful_realtime_group_total_samples =
                        stateful_realtime_group_total_samples.saturating_add(node.latency_samples);
                    stateful_realtime_group_node_count =
                        stateful_realtime_group_node_count.saturating_add(1);
                }
                GraphNodePlanningGroup::AnticipativeEligible => {
                    anticipative_group_total_samples =
                        anticipative_group_total_samples.saturating_add(node.latency_samples);
                    anticipative_group_node_count = anticipative_group_node_count.saturating_add(1);
                }
            }
        }
        let hot_latency_group = [
            (
                GraphNodePlanningGroup::InlineRealtime,
                inline_realtime_group_total_samples,
                inline_realtime_group_node_count,
            ),
            (
                GraphNodePlanningGroup::StatefulRealtime,
                stateful_realtime_group_total_samples,
                stateful_realtime_group_node_count,
            ),
            (
                GraphNodePlanningGroup::AnticipativeEligible,
                anticipative_group_total_samples,
                anticipative_group_node_count,
            ),
        ]
        .into_iter()
        .max_by_key(|(_, total_samples, _)| *total_samples)
        .filter(|(_, total_samples, _)| *total_samples > 0);
        let critical_path_lane = worker_lane_summaries
            .iter()
            .max_by_key(|summary| summary.total_latency_samples)
            .filter(|summary| summary.total_latency_samples > 0);
        let mut snapshot = Self {
            sample_rate_hz: effective_config.sample_rate.0,
            block_size: effective_config.block_size,
            processed_block_count: engine_block_snapshot.processed_blocks,
            last_block_sequence: engine_block_snapshot.last_block_sequence,
            cpu_load_percent: diagnostics_snapshot.cpu_load_percent,
            graph_latency_ms: diagnostics_snapshot.graph_latency_ms,
            last_block_execution_time_ns: engine_block_snapshot.last_block_execution_time_ns,
            last_block_deadline_budget_ns: engine_block_snapshot.last_block_deadline_budget_ns,
            last_block_budget_utilization_percent: engine_block_snapshot
                .last_block_budget_utilization_percent,
            last_block_budget_overrun_ns: engine_block_snapshot.last_block_budget_overrun_ns,
            last_block_deadline_pressure: engine_block_snapshot.last_block_deadline_pressure,
            budget_overrun_count: engine_block_snapshot.budget_overrun_count,
            peak_block_execution_time_ns: engine_block_snapshot.peak_block_execution_time_ns,
            peak_block_budget_utilization_percent: engine_block_snapshot
                .peak_block_budget_utilization_percent,
            peak_block_budget_overrun_ns: engine_block_snapshot.peak_block_budget_overrun_ns,
            xrun_count: diagnostics_snapshot.xruns,
            scheduler_phase_count: engine_block_snapshot.phase_count,
            scheduler_lane_count: engine_block_snapshot.lane_count,
            scheduler_dispatch_count: engine_block_snapshot.dispatch_count,
            scheduler_prepared_dispatch_count: engine_block_snapshot.prepared_dispatch_count,
            scheduler_realtime_dispatch_count: engine_block_snapshot.realtime_dispatch_count,
            scheduler_dispatch_handoff_count: engine_block_snapshot.dispatch_handoff_count,
            scheduler_topology_compatible: engine_block_snapshot.scheduler_topology.compatible,
            scheduler_topology_requires_host_reinterpretation: engine_block_snapshot
                .scheduler_topology
                .requires_host_reinterpretation,
            scheduler_topology_issue_count: engine_block_snapshot.scheduler_topology.issues.len(),
            prework_service_state: engine_block_snapshot.prework_service_state,
            prework_service_pressure: engine_block_snapshot.prework_service_pressure,
            prework_service_semantic_policy: engine_block_snapshot.prework_service_semantic_policy,
            pending_prework_target_count: engine_block_snapshot.prework_pending_target_count,
            pending_prework_deferred_target_count: engine_block_snapshot
                .prework_pending_deferred_target_count,
            prework_queue_depth: engine_block_snapshot.prework_cache_queue_depth,
            prework_peak_queue_depth: engine_block_snapshot.prework_cache_peak_queue_depth,
            prework_service_cycle_count: engine_block_snapshot.prework_service_cycle_count,
            prework_service_starvation_count: engine_block_snapshot
                .prework_service_starvation_count,
            prework_service_throttle_count: engine_block_snapshot.prework_service_throttle_count,
            prework_service_yield_count: engine_block_snapshot.prework_service_yield_count,
            last_prework_service_effective_cycles: engine_block_snapshot
                .last_prework_service_effective_cycles,
            last_prework_service_budget_per_cycle: engine_block_snapshot
                .last_prework_service_budget_per_cycle,
            last_prework_service_effective_budget_per_cycle: engine_block_snapshot
                .last_prework_service_effective_budget_per_cycle,
            last_prework_serviced_backlog_class: engine_block_snapshot
                .last_prework_serviced_backlog_class
                .map(|value| runtime_prework_backlog_class_name(value).to_string()),
            transport_gate_active: engine_block_snapshot.prework_service_transport_gate_active,
            plugin_gate_active: engine_block_snapshot.prework_service_plugin_gate_active,
            hot_latency_node_id: hot_latency_node.map(|node| node.node_id.clone()),
            hot_latency_node_group: hot_latency_node
                .map(|node| runtime_graph_node_planning_group_name(node.group).to_string()),
            hot_latency_node_topology_role: hot_latency_node
                .map(|node| runtime_graph_node_topology_role_name(node.topology_role).to_string()),
            hot_latency_node_plugin_sandbox_id: hot_latency_node
                .and_then(|node| node.plugin_sandbox_id.clone()),
            hot_latency_node_samples: hot_latency_node.map_or(0, |node| node.latency_samples),
            hot_latency_group: hot_latency_group
                .map(|(group, _, _)| runtime_graph_node_planning_group_name(group).to_string()),
            hot_latency_group_node_count: hot_latency_group
                .map_or(0, |(_, _, node_count)| node_count),
            hot_latency_group_total_samples: hot_latency_group
                .map_or(0, |(_, total_samples, _)| total_samples),
            critical_path_lane: critical_path_lane
                .map(|summary| runtime_execution_lane_name(summary.lane).to_string()),
            critical_path_lane_node_count: critical_path_lane
                .map_or(0, |summary| summary.node_count),
            critical_path_lane_plugin_backed_node_count: critical_path_lane
                .map_or(0, |summary| summary.plugin_backed_node_count),
            critical_path_lane_planning_group_count: critical_path_lane
                .map_or(0, |summary| summary.planning_group_count),
            critical_path_lane_total_latency_samples: critical_path_lane
                .map_or(0, |summary| summary.total_latency_samples),
            worker_lane_summaries,
            background_service_class: last_deferred_service_receipt
                .map(|receipt| receipt.work_class),
            background_service_decision: last_deferred_service_receipt
                .map(|receipt| receipt.decision),
            background_service_reason: last_deferred_service_receipt.map(|receipt| receipt.reason),
            background_service_priority_band: last_deferred_service_receipt
                .map(|receipt| receipt.priority_band),
            background_service_blocking_priority_band: last_deferred_service_receipt
                .and_then(|receipt| receipt.blocking_priority_band),
            background_service_backpressure_source: last_deferred_service_receipt
                .and_then(|receipt| receipt.backpressure_source),
            background_service_starvation_risk: last_deferred_service_receipt
                .is_some_and(|receipt| receipt.starvation_risk),
            background_service_starved_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.starved_work_item_count)
                .unwrap_or(0),
            background_service_cancellation_cause: last_deferred_service_receipt
                .and_then(|receipt| receipt.cancellation_cause),
            background_service_cancelled_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.cancelled_work_item_count)
                .unwrap_or(0),
            background_queued_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.queued_work_item_count)
                .unwrap_or(0),
            background_deferred_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.deferred_work_item_count)
                .unwrap_or(0),
            background_pending_cleanup_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.pending_cleanup_work_items)
                .unwrap_or(0),
            background_pending_retry_work_item_count: last_deferred_service_receipt
                .map(|receipt| receipt.pending_deferred_retry_work_items)
                .unwrap_or(0),
            summary: String::new(),
        };
        let dispatch_summary = format!(
            "{}/{}/{}",
            snapshot.scheduler_dispatch_count,
            snapshot.scheduler_prepared_dispatch_count,
            snapshot.scheduler_realtime_dispatch_count
        );
        let topology_summary = format!(
            "{}/{}/{}",
            snapshot.scheduler_topology_compatible,
            snapshot.scheduler_topology_requires_host_reinterpretation,
            snapshot.scheduler_topology_issue_count
        );
        let prework_summary = format!(
            "{:?}/{:?}/{:?}",
            snapshot.prework_service_state,
            snapshot.prework_service_pressure,
            snapshot.prework_service_semantic_policy,
        );
        let service_summary = format!(
            "{}/{}/{}/{}",
            snapshot.prework_service_starvation_count,
            snapshot.prework_service_throttle_count,
            snapshot.prework_service_yield_count,
            snapshot.last_prework_service_effective_cycles,
        );
        let hot_node_summary = format!(
            "{:?}/{:?}/{:?}/{}",
            snapshot.hot_latency_node_id,
            snapshot.hot_latency_node_group,
            snapshot.hot_latency_node_topology_role,
            snapshot.hot_latency_node_samples,
        );
        let hot_group_summary = format!(
            "{:?}/{}/{}",
            snapshot.hot_latency_group,
            snapshot.hot_latency_group_node_count,
            snapshot.hot_latency_group_total_samples
        );
        let critical_lane_summary = format!(
            "{:?}/{}/{}/{}/{}",
            snapshot.critical_path_lane,
            snapshot.critical_path_lane_node_count,
            snapshot.critical_path_lane_plugin_backed_node_count,
            snapshot.critical_path_lane_planning_group_count,
            snapshot.critical_path_lane_total_latency_samples
        );
        let worker_lane_summary = snapshot
            .worker_lane_summaries
            .iter()
            .map(|summary| {
                format!(
                    "{}:{}/{}/{}/{}/{}",
                    runtime_execution_lane_name(summary.lane),
                    summary.node_count,
                    summary.plugin_backed_node_count,
                    summary.planning_group_count,
                    summary.total_latency_samples,
                    summary.max_node_latency_samples,
                )
            })
            .collect::<Vec<_>>()
            .join("|");
        let background_summary = format!(
            "{:?}/{:?}/{:?}/{:?}/{:?}/{:?}/{}/{}",
            snapshot.background_service_class,
            snapshot.background_service_decision,
            snapshot.background_service_reason,
            snapshot.background_service_priority_band,
            snapshot.background_service_blocking_priority_band,
            snapshot.background_service_backpressure_source,
            snapshot.background_service_starvation_risk,
            snapshot.background_service_cancelled_work_item_count,
        );
        snapshot.summary = format!(
            "sample_rate={} block_size={} blocks={} cpu_load={:.3} graph_latency_ms={:.3} timing={:?}/{:?}/{:?}/{:?}/{:?}/{} xruns={} phases={} lanes={} dispatches={} handoff={} topology={} prework={} pending_targets={}/{} queue={}/{} service={} cycles={} budget={:?}/{:?} backlog={:?} gates={}/{} hot_node={} hot_group={} critical_lane={} worker_lanes={} background={} policy={:?}/{:?}/{:?}/{}/{} items={}/{}/{}/{}",
            snapshot.sample_rate_hz,
            snapshot.block_size,
            snapshot.processed_block_count,
            snapshot.cpu_load_percent,
            snapshot.graph_latency_ms,
            snapshot.last_block_execution_time_ns,
            snapshot.last_block_deadline_budget_ns,
            snapshot.last_block_budget_utilization_percent,
            snapshot.last_block_budget_overrun_ns,
            snapshot.last_block_deadline_pressure,
            snapshot.budget_overrun_count,
            snapshot.xrun_count,
            snapshot.scheduler_phase_count,
            snapshot.scheduler_lane_count,
            dispatch_summary,
            snapshot.scheduler_dispatch_handoff_count,
            topology_summary,
            prework_summary,
            snapshot.pending_prework_target_count,
            snapshot.pending_prework_deferred_target_count,
            snapshot.prework_queue_depth,
            snapshot.prework_peak_queue_depth,
            service_summary,
            snapshot.prework_service_cycle_count,
            snapshot.last_prework_service_budget_per_cycle,
            snapshot.last_prework_service_effective_budget_per_cycle,
            snapshot.last_prework_serviced_backlog_class,
            snapshot.transport_gate_active,
            snapshot.plugin_gate_active,
            hot_node_summary,
            hot_group_summary,
            critical_lane_summary,
            worker_lane_summary,
            background_summary,
            snapshot.background_service_priority_band,
            snapshot.background_service_blocking_priority_band,
            snapshot.background_service_backpressure_source,
            snapshot.background_service_starved_work_item_count,
            snapshot.background_service_cancelled_work_item_count,
            snapshot.background_queued_work_item_count,
            snapshot.background_deferred_work_item_count,
            snapshot.background_pending_cleanup_work_item_count,
            snapshot.background_pending_retry_work_item_count,
        );
        snapshot
    }

    pub fn render_json(&self) -> String {
        format!(
            concat!(
                "{{",
                "\"sample_rate_hz\":{},",
                "\"block_size\":{},",
                "\"processed_block_count\":{},",
                "\"last_block_sequence\":{},",
                "\"cpu_load_percent\":{},",
                "\"graph_latency_ms\":{},",
                "\"last_block_execution_time_ns\":{},",
                "\"last_block_deadline_budget_ns\":{},",
                "\"last_block_budget_utilization_percent\":{},",
                "\"last_block_budget_overrun_ns\":{},",
                "\"last_block_deadline_pressure\":\"{:?}\",",
                "\"budget_overrun_count\":{},",
                "\"peak_block_execution_time_ns\":{},",
                "\"peak_block_budget_utilization_percent\":{},",
                "\"peak_block_budget_overrun_ns\":{},",
                "\"xrun_count\":{},",
                "\"scheduler_phase_count\":{},",
                "\"scheduler_lane_count\":{},",
                "\"scheduler_dispatch_count\":{},",
                "\"scheduler_prepared_dispatch_count\":{},",
                "\"scheduler_realtime_dispatch_count\":{},",
                "\"scheduler_dispatch_handoff_count\":{},",
                "\"scheduler_topology_compatible\":{},",
                "\"scheduler_topology_requires_host_reinterpretation\":{},",
                "\"scheduler_topology_issue_count\":{},",
                "\"prework_service_state\":\"{:?}\",",
                "\"prework_service_pressure\":\"{:?}\",",
                "\"prework_service_semantic_policy\":\"{:?}\",",
                "\"pending_prework_target_count\":{},",
                "\"pending_prework_deferred_target_count\":{},",
                "\"prework_queue_depth\":{},",
                "\"prework_peak_queue_depth\":{},",
                "\"prework_service_cycle_count\":{},",
                "\"prework_service_starvation_count\":{},",
                "\"prework_service_throttle_count\":{},",
                "\"prework_service_yield_count\":{},",
                "\"last_prework_service_effective_cycles\":{},",
                "\"last_prework_service_budget_per_cycle\":{},",
                "\"last_prework_service_effective_budget_per_cycle\":{},",
                "\"last_prework_serviced_backlog_class\":{},",
                "\"transport_gate_active\":{},",
                "\"plugin_gate_active\":{},",
                "\"hot_latency_node_id\":{},",
                "\"hot_latency_node_group\":{},",
                "\"hot_latency_node_topology_role\":{},",
                "\"hot_latency_node_plugin_sandbox_id\":{},",
                "\"hot_latency_node_samples\":{},",
                "\"hot_latency_group\":{},",
                "\"hot_latency_group_node_count\":{},",
                "\"hot_latency_group_total_samples\":{},",
                "\"critical_path_lane\":{},",
                "\"critical_path_lane_node_count\":{},",
                "\"critical_path_lane_plugin_backed_node_count\":{},",
                "\"critical_path_lane_planning_group_count\":{},",
                "\"critical_path_lane_total_latency_samples\":{},",
                "\"worker_lane_summaries\":{},",
                "\"background_service_class\":{},",
                "\"background_service_decision\":{},",
                "\"background_service_reason\":{},",
                "\"background_service_priority_band\":{},",
                "\"background_service_blocking_priority_band\":{},",
                "\"background_service_backpressure_source\":{},",
                "\"background_service_starvation_risk\":{},",
                "\"background_service_starved_work_item_count\":{},",
                "\"background_service_cancellation_cause\":{},",
                "\"background_service_cancelled_work_item_count\":{},",
                "\"background_queued_work_item_count\":{},",
                "\"background_deferred_work_item_count\":{},",
                "\"background_pending_cleanup_work_item_count\":{},",
                "\"background_pending_retry_work_item_count\":{},",
                "\"summary\":{}}}",
            ),
            self.sample_rate_hz,
            self.block_size,
            self.processed_block_count,
            json_option_u64(self.last_block_sequence),
            self.cpu_load_percent,
            self.graph_latency_ms,
            json_option_u64(self.last_block_execution_time_ns),
            json_option_u64(self.last_block_deadline_budget_ns),
            json_option_f32(self.last_block_budget_utilization_percent),
            json_option_u64(self.last_block_budget_overrun_ns),
            self.last_block_deadline_pressure,
            self.budget_overrun_count,
            self.peak_block_execution_time_ns,
            self.peak_block_budget_utilization_percent,
            self.peak_block_budget_overrun_ns,
            self.xrun_count,
            self.scheduler_phase_count,
            self.scheduler_lane_count,
            self.scheduler_dispatch_count,
            self.scheduler_prepared_dispatch_count,
            self.scheduler_realtime_dispatch_count,
            self.scheduler_dispatch_handoff_count,
            self.scheduler_topology_compatible,
            self.scheduler_topology_requires_host_reinterpretation,
            self.scheduler_topology_issue_count,
            self.prework_service_state,
            self.prework_service_pressure,
            self.prework_service_semantic_policy,
            self.pending_prework_target_count,
            self.pending_prework_deferred_target_count,
            self.prework_queue_depth,
            self.prework_peak_queue_depth,
            self.prework_service_cycle_count,
            self.prework_service_starvation_count,
            self.prework_service_throttle_count,
            self.prework_service_yield_count,
            self.last_prework_service_effective_cycles,
            json_option_u64(
                self.last_prework_service_budget_per_cycle
                    .map(|value| value as u64)
            ),
            json_option_u64(
                self.last_prework_service_effective_budget_per_cycle
                    .map(|value| value as u64),
            ),
            json_option_string(self.last_prework_serviced_backlog_class.as_deref()),
            self.transport_gate_active,
            self.plugin_gate_active,
            json_option_string(self.hot_latency_node_id.as_deref()),
            json_option_string(self.hot_latency_node_group.as_deref()),
            json_option_string(self.hot_latency_node_topology_role.as_deref()),
            json_option_string(self.hot_latency_node_plugin_sandbox_id.as_deref()),
            self.hot_latency_node_samples,
            json_option_string(self.hot_latency_group.as_deref()),
            self.hot_latency_group_node_count,
            self.hot_latency_group_total_samples,
            json_option_string(self.critical_path_lane.as_deref()),
            self.critical_path_lane_node_count,
            self.critical_path_lane_plugin_backed_node_count,
            self.critical_path_lane_planning_group_count,
            self.critical_path_lane_total_latency_samples,
            json_runtime_worker_lane_instrumentation_summaries(&self.worker_lane_summaries),
            json_option_string(
                self.background_service_class
                    .as_ref()
                    .map(|value| match value {
                        RuntimeDeferredServiceClass::OfflineRenderQueue => "OfflineRenderQueue",
                        RuntimeDeferredServiceClass::OfflineRenderPurge => "OfflineRenderPurge",
                    }),
            ),
            json_option_string(self.background_service_decision.as_ref().map(
                |value| match value {
                    RuntimeDeferredServiceDecision::Run => "Run",
                    RuntimeDeferredServiceDecision::Defer => "Defer",
                    RuntimeDeferredServiceDecision::Throttle => "Throttle",
                    RuntimeDeferredServiceDecision::Abort => "Abort",
                }
            ),),
            json_option_string(
                self.background_service_reason
                    .as_ref()
                    .map(|value| match value {
                        RuntimeDeferredServiceReason::Ready => "Ready",
                        RuntimeDeferredServiceReason::RealtimeActive => "RealtimeActive",
                        RuntimeDeferredServiceReason::PendingCleanup => "PendingCleanup",
                        RuntimeDeferredServiceReason::RecoveryDegraded => "RecoveryDegraded",
                        RuntimeDeferredServiceReason::SafeMode => "SafeMode",
                        RuntimeDeferredServiceReason::InvalidRequest => "InvalidRequest",
                    }),
            ),
            json_option_string(
                self.background_service_priority_band
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            json_option_string(
                self.background_service_blocking_priority_band
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            json_option_string(
                self.background_service_backpressure_source
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.background_service_starvation_risk,
            self.background_service_starved_work_item_count,
            json_option_string(
                self.background_service_cancellation_cause
                    .as_ref()
                    .map(|value| format!("{value:?}"))
                    .as_deref(),
            ),
            self.background_service_cancelled_work_item_count,
            self.background_queued_work_item_count,
            self.background_deferred_work_item_count,
            self.background_pending_cleanup_work_item_count,
            self.background_pending_retry_work_item_count,
            json_option_string(Some(self.summary.as_str())),
        )
    }
}

fn runtime_worker_lane_instrumentation_summaries(
    engine_block_snapshot: &RuntimeEngineBlockSnapshot,
) -> Vec<RuntimeWorkerLaneInstrumentationSummary> {
    let mut lane_order = engine_block_snapshot.lane_order.clone();
    for node in &engine_block_snapshot.planned_nodes {
        let lane = runtime_lane_for_group(node.group);
        if !lane_order.contains(&lane) {
            lane_order.push(lane);
        }
    }

    let mut summaries = Vec::new();
    for lane in lane_order {
        let mut node_count = 0usize;
        let mut plugin_backed_node_count = 0usize;
        let mut planning_groups = Vec::new();
        let mut total_latency_samples = 0u32;
        let mut max_node_latency_samples = 0u32;

        for node in engine_block_snapshot
            .planned_nodes
            .iter()
            .filter(|node| runtime_lane_for_group(node.group) == lane)
        {
            node_count = node_count.saturating_add(1);
            if matches!(node.execution_class, GraphNodeExecutionClass::PluginBacked) {
                plugin_backed_node_count = plugin_backed_node_count.saturating_add(1);
            }
            if !planning_groups.contains(&node.group) {
                planning_groups.push(node.group);
            }
            total_latency_samples = total_latency_samples.saturating_add(node.latency_samples);
            max_node_latency_samples = max_node_latency_samples.max(node.latency_samples);
        }

        if node_count > 0 {
            summaries.push(RuntimeWorkerLaneInstrumentationSummary {
                lane,
                node_count,
                plugin_backed_node_count,
                planning_group_count: planning_groups.len(),
                total_latency_samples,
                max_node_latency_samples,
            });
        }
    }

    summaries
}

fn runtime_graph_node_planning_group_name(group: GraphNodePlanningGroup) -> &'static str {
    match group {
        GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
        GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
        GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
    }
}

fn runtime_graph_node_topology_role_name(role: GraphNodeTopologyRole) -> &'static str {
    match role {
        GraphNodeTopologyRole::Utility => "Utility",
        GraphNodeTopologyRole::TrackLane => "TrackLane",
        GraphNodeTopologyRole::Bus => "Bus",
        GraphNodeTopologyRole::Send => "Send",
        GraphNodeTopologyRole::Return => "Return",
        GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
    }
}

fn runtime_execution_lane_name(lane: GraphExecutionLane) -> &'static str {
    match lane {
        GraphExecutionLane::Realtime => "Realtime",
        GraphExecutionLane::Anticipative => "Anticipative",
    }
}

fn runtime_prework_backlog_class_name(value: RuntimePreworkBacklogClass) -> &'static str {
    match value {
        RuntimePreworkBacklogClass::Immediate => "Immediate",
        RuntimePreworkBacklogClass::NearTerm => "NearTerm",
        RuntimePreworkBacklogClass::Deferred => "Deferred",
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostAudioStreamState {
    Stopped,
    Running,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeHostAudioTransferPolicy {
    pub max_callback_frames: usize,
    pub max_transfer_channels: u16,
    pub zero_fill_unwritten_output: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostLifecycleOwnership {
    HostDrivenCallback,
    BackendManagedCallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostRestartPolicy {
    HostMustRestart,
    BackendMayRestart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockSource {
    Internal,
    ExternalWordClock,
    DigitalInput,
    Virtual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDomain {
    SameClock,
    CrossClock,
    Aggregate,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockFallbackState {
    Direct,
    RuntimeResampled,
    RecoveryConstrained,
    Unconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockTransitionState {
    InitialObservation,
    Stable,
    EnteredAggregateClock,
    EnteredCrossClockFallback,
    EnteredRecoveryFallback,
    ReturnedToDirect,
    LostConfiguration,
    Reconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDriftState {
    Stable,
    CrossClockManaged,
    AggregateManaged,
    Resyncing,
    Unconfigured,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostClockDiscontinuityState {
    Continuous,
    Reconfigured,
    Recovering,
    LostConfiguration,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostDuplexMismatchState {
    NotApplicable,
    Aligned,
    CrossClockDiverged,
    PartialAvailability,
    Degraded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeHostEndpointTopology {
    Unconfigured,
    OutputOnly,
    InputOnly,
    Duplex,
    Aggregate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostClockingSummary {
    pub clock_source: RuntimeHostClockSource,
    pub ownership: RuntimeHostLifecycleOwnership,
    pub restart_policy: RuntimeHostRestartPolicy,
    pub processing_sample_rate_hz: u32,
    pub hardware_sample_rate_hz: u32,
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub transition_state: RuntimeHostClockTransitionState,
    pub drift_state: RuntimeHostClockDriftState,
    pub discontinuity_state: RuntimeHostClockDiscontinuityState,
    pub duplex_mismatch_state: RuntimeHostDuplexMismatchState,
    pub endpoint_topology: RuntimeHostEndpointTopology,
    pub linux_clocking_parity: RuntimeLinuxAudioBackendClockingParityBand,
    pub linux_duplex_parity: RuntimeLinuxAudioBackendDuplexParityState,
    pub linux_endpoint_topology_parity: RuntimeLinuxAudioBackendEndpointTopologyParityState,
    pub partial_availability: bool,
    pub crossing_required: bool,
    pub callback_interval_ms: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeHostLatencySummary {
    pub input_latency_samples: Option<u32>,
    pub output_latency_samples: u32,
    pub round_trip_latency_samples: Option<u32>,
    pub graph_latency_samples: u32,
    pub estimated_output_latency_samples: u32,
    pub estimated_round_trip_latency_samples: Option<u32>,
    pub output_latency_ms: f32,
    pub graph_latency_ms: f32,
    pub estimated_output_latency_ms: f32,
    pub estimated_round_trip_latency_ms: Option<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeHostHardwareSummary {
    pub backend_identity: HardwareBackendIdentity,
    pub backend_name: String,
    pub linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub linux_backend_portability: RuntimeLinuxAudioBackendPortabilityBand,
    pub device_id: String,
    pub device_name: String,
    pub sample_rate: u32,
    pub buffer_size: usize,
    pub input_channels: u16,
    pub output_channels: u16,
    pub sample_format: AudioSampleFormat,
    pub simulated: bool,
    pub backend_health: BackendHealth,
    pub xrun_count: u64,
    pub callback_overrun_count: u64,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
}

impl RuntimeHostHardwareSummary {
    pub fn classify_linux_backend_identity(
        backend_identity: HardwareBackendIdentity,
    ) -> RuntimeLinuxAudioBackendIdentity {
        match backend_identity {
            HardwareBackendIdentity::CoreAudio => RuntimeLinuxAudioBackendIdentity::NotLinux,
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa) => {
                RuntimeLinuxAudioBackendIdentity::Alsa
            }
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack) => {
                RuntimeLinuxAudioBackendIdentity::Jack
            }
            HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire) => {
                RuntimeLinuxAudioBackendIdentity::PipeWire
            }
            HardwareBackendIdentity::Unsupported => RuntimeLinuxAudioBackendIdentity::Unsupported,
        }
    }

    pub fn classify_linux_backend_portability(
        backend_identity: HardwareBackendIdentity,
        simulated: bool,
        backend_health: BackendHealth,
        device_loss_count: u64,
        restart_attempt_count: u64,
        restart_failure_count: u64,
    ) -> RuntimeLinuxAudioBackendPortabilityBand {
        match Self::classify_linux_backend_identity(backend_identity) {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if simulated
                    || !matches!(backend_health, BackendHealth::Healthy)
                    || device_loss_count > 0
                    || restart_attempt_count > 0
                    || restart_failure_count > 0
                {
                    RuntimeLinuxAudioBackendPortabilityBand::Guarded
                } else {
                    RuntimeLinuxAudioBackendPortabilityBand::Portable
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendPortabilityBand::Unsupported
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostAudioPumpSummary {
    pub stream_state: RuntimeHostAudioStreamState,
    pub transfer_policy: RuntimeHostAudioTransferPolicy,
    pub callback_count: u64,
    pub total_callback_frames: u64,
    pub total_runtime_output_frames: u64,
    pub copied_output_samples: u64,
    pub zero_filled_output_samples: u64,
    pub dropped_output_samples: u64,
    pub last_callback_output_peak: Option<f32>,
    pub last_runtime_graph_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeHostIoSummary {
    pub hardware: RuntimeHostHardwareSummary,
    pub audio_pump: RuntimeHostAudioPumpSummary,
    pub clocking: RuntimeHostClockingSummary,
    pub latency: RuntimeHostLatencySummary,
    pub runtime_graph_id_matches_pump: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiDiscoveryState {
    Unavailable,
    Idle,
    Enumerated,
    Changed,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiGraphState {
    Unavailable,
    Empty,
    Stable,
    Guarded,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiDeviceLifecycleState {
    Unavailable,
    Discovered,
    Guarded,
    Detached,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiEndpointLifecycleState {
    Unavailable,
    Idle,
    Active,
    Guarded,
    Detached,
    Faulted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiEndpointDirection {
    Input,
    Output,
    Duplex,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiRouteState {
    Unavailable,
    Detached,
    InputObserved,
    OutputObserved,
    DuplexObserved,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiLiveOwnershipPosture {
    Unavailable,
    NoLiveOwnership,
    RuntimeDeclaredLiveOwnership,
    GuardedLiveOwnership,
    BackendAdvisoryLiveOwnership,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiAttachContinuity {
    Unavailable,
    Detached,
    Attached,
    Resumable,
    Restartable,
    Terminal,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiBackendParity {
    NotLinux,
    Unavailable,
    Portable,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeExternalMidiGuardedParityOutcome {
    NotLinux,
    Unavailable,
    Direct,
    BackendManaged,
    RecoveryGuarded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiLiveOwnershipSummary {
    pub ownership_posture: RuntimeExternalMidiLiveOwnershipPosture,
    pub attach_continuity: RuntimeExternalMidiAttachContinuity,
    pub backend_parity: RuntimeExternalMidiBackendParity,
    pub guarded_parity_outcome: RuntimeExternalMidiGuardedParityOutcome,
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}

impl RuntimeExternalMidiLiveOwnershipSummary {
    pub fn unavailable() -> Self {
        Self {
            ownership_posture: RuntimeExternalMidiLiveOwnershipPosture::Unavailable,
            attach_continuity: RuntimeExternalMidiAttachContinuity::Unavailable,
            backend_parity: RuntimeExternalMidiBackendParity::Unavailable,
            guarded_parity_outcome: RuntimeExternalMidiGuardedParityOutcome::Unavailable,
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "ownership=Unavailable continuity=Unavailable parity=Unavailable guarded=Unavailable backend=Unavailable"
                    .into(),
        }
    }

    pub fn detached_without_backend_context() -> Self {
        Self {
            ownership_posture: RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership,
            attach_continuity: RuntimeExternalMidiAttachContinuity::Detached,
            backend_parity: RuntimeExternalMidiBackendParity::Unavailable,
            guarded_parity_outcome: RuntimeExternalMidiGuardedParityOutcome::Unavailable,
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "ownership=NoLiveOwnership continuity=Detached parity=Unavailable guarded=Unavailable backend=Unavailable"
                    .into(),
        }
    }

    pub fn from_linux_session_and_interruption(
        linux_session: &RuntimeLinuxBackendSessionSnapshot,
        interruption_summary: &RuntimeInterruptionSummary,
        graph_state: RuntimeExternalMidiGraphState,
        device_count: usize,
        endpoint_count: usize,
    ) -> Self {
        let backend_identity = linux_session.backend_identity;
        let backend_parity = match backend_identity {
            RuntimeLinuxAudioBackendIdentity::NotLinux => {
                RuntimeExternalMidiBackendParity::NotLinux
            }
            RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeExternalMidiBackendParity::Unavailable
            }
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if linux_session.portability_band
                    == RuntimeLinuxAudioBackendPortabilityBand::Portable
                {
                    RuntimeExternalMidiBackendParity::Portable
                } else {
                    RuntimeExternalMidiBackendParity::Guarded
                }
            }
        };
        let guarded_parity_outcome = match backend_parity {
            RuntimeExternalMidiBackendParity::NotLinux => {
                RuntimeExternalMidiGuardedParityOutcome::NotLinux
            }
            RuntimeExternalMidiBackendParity::Unavailable => {
                RuntimeExternalMidiGuardedParityOutcome::Unavailable
            }
            RuntimeExternalMidiBackendParity::Portable => {
                RuntimeExternalMidiGuardedParityOutcome::Direct
            }
            RuntimeExternalMidiBackendParity::Guarded => match linux_session.ownership_fallback {
                RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded => {
                    RuntimeExternalMidiGuardedParityOutcome::BackendManaged
                }
                RuntimeLinuxBackendOwnershipFallbackState::Reacquiring
                | RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained => {
                    RuntimeExternalMidiGuardedParityOutcome::RecoveryGuarded
                }
                RuntimeLinuxBackendOwnershipFallbackState::Direct => {
                    RuntimeExternalMidiGuardedParityOutcome::Direct
                }
                RuntimeLinuxBackendOwnershipFallbackState::NotLinux => {
                    RuntimeExternalMidiGuardedParityOutcome::NotLinux
                }
                RuntimeLinuxBackendOwnershipFallbackState::Unavailable => {
                    RuntimeExternalMidiGuardedParityOutcome::Unavailable
                }
            },
        };
        let attach_continuity = match backend_parity {
            RuntimeExternalMidiBackendParity::Unavailable => {
                RuntimeExternalMidiAttachContinuity::Unavailable
            }
            _ if matches!(
                interruption_summary.class,
                RuntimeInterruptionClass::Terminal
            ) || graph_state == RuntimeExternalMidiGraphState::Faulted =>
            {
                RuntimeExternalMidiAttachContinuity::Terminal
            }
            _ if endpoint_count == 0 || graph_state == RuntimeExternalMidiGraphState::Empty => {
                RuntimeExternalMidiAttachContinuity::Detached
            }
            _ => match interruption_summary.class {
                RuntimeInterruptionClass::Steady => RuntimeExternalMidiAttachContinuity::Attached,
                RuntimeInterruptionClass::Resumable => {
                    RuntimeExternalMidiAttachContinuity::Resumable
                }
                RuntimeInterruptionClass::Restartable | RuntimeInterruptionClass::Recoverable => {
                    RuntimeExternalMidiAttachContinuity::Restartable
                }
                RuntimeInterruptionClass::Terminal => RuntimeExternalMidiAttachContinuity::Terminal,
            },
        };
        let ownership_posture = match backend_parity {
            RuntimeExternalMidiBackendParity::Unavailable => {
                RuntimeExternalMidiLiveOwnershipPosture::Unavailable
            }
            _ if device_count == 0 || endpoint_count == 0 => {
                RuntimeExternalMidiLiveOwnershipPosture::NoLiveOwnership
            }
            _ if linux_session.ownership
                == RuntimeLinuxBackendSessionOwnership::BackendManagedGraph =>
            {
                RuntimeExternalMidiLiveOwnershipPosture::BackendAdvisoryLiveOwnership
            }
            _ if guarded_parity_outcome != RuntimeExternalMidiGuardedParityOutcome::Direct
                || matches!(
                    interruption_summary.class,
                    RuntimeInterruptionClass::Resumable
                        | RuntimeInterruptionClass::Restartable
                        | RuntimeInterruptionClass::Recoverable
                        | RuntimeInterruptionClass::Terminal
                ) =>
            {
                RuntimeExternalMidiLiveOwnershipPosture::GuardedLiveOwnership
            }
            _ => RuntimeExternalMidiLiveOwnershipPosture::RuntimeDeclaredLiveOwnership,
        };

        let mut summary = Self {
            ownership_posture,
            attach_continuity,
            backend_parity,
            guarded_parity_outcome,
            backend_identity,
            device_loss_count: linux_session.device_loss_count,
            restart_attempt_count: linux_session.restart_attempt_count,
            restart_failure_count: linux_session.restart_failure_count,
            summary: String::new(),
        };
        summary.summary = format!(
            "ownership={:?} continuity={:?} parity={:?} guarded={:?} backend={:?} device_losses={} restart_attempts={} restart_failures={}",
            summary.ownership_posture,
            summary.attach_continuity,
            summary.backend_parity,
            summary.guarded_parity_outcome,
            summary.backend_identity,
            summary.device_loss_count,
            summary.restart_attempt_count,
            summary.restart_failure_count,
        );
        summary
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointCapabilitySummary {
    pub supports_bounded_midi_input: bool,
    pub supports_bounded_midi_output: bool,
    pub supports_transport_clock: bool,
    pub supports_note_events: bool,
    pub supports_controller_events: bool,
    pub supports_note_pressure_expression: bool,
    pub supports_note_timbre_expression: bool,
    pub supports_note_tuning_expression: bool,
    pub supports_mpe: bool,
    pub midi2_posture: RuntimeControllerExpressionMidi2Posture,
    pub control_surface_guarded: bool,
    pub summary: String,
}

impl RuntimeExternalMidiEndpointCapabilitySummary {
    pub fn unavailable() -> Self {
        Self {
            supports_bounded_midi_input: false,
            supports_bounded_midi_output: false,
            supports_transport_clock: false,
            supports_note_events: false,
            supports_controller_events: false,
            supports_note_pressure_expression: false,
            supports_note_timbre_expression: false,
            supports_note_tuning_expression: false,
            supports_mpe: false,
            midi2_posture: RuntimeControllerExpressionMidi2Posture::Unsupported,
            control_surface_guarded: true,
            summary: "midi-input=false midi-output=false transport-clock=false note-events=false controller-events=false pressure=false timbre=false tuning=false mpe=false midi2=Unsupported control-surface=guarded".into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiDeviceDescriptor {
    pub device_id: String,
    pub device_name: String,
    pub lifecycle_state: RuntimeExternalMidiDeviceLifecycleState,
    pub endpoint_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointDescriptor {
    pub endpoint_id: String,
    pub endpoint_name: String,
    pub device_id: String,
    pub direction: RuntimeExternalMidiEndpointDirection,
    pub lifecycle_state: RuntimeExternalMidiEndpointLifecycleState,
    pub route_state: RuntimeExternalMidiRouteState,
    pub capability: RuntimeExternalMidiEndpointCapabilitySummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExternalMidiEndpointGraphSnapshot {
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    pub graph_state: RuntimeExternalMidiGraphState,
    pub live_ownership: RuntimeExternalMidiLiveOwnershipSummary,
    pub provider_name: String,
    pub device_count: usize,
    pub endpoint_count: usize,
    pub input_endpoint_count: usize,
    pub output_endpoint_count: usize,
    pub duplex_endpoint_count: usize,
    pub active_route_count: usize,
    pub guarded_route_count: usize,
    pub devices: Vec<RuntimeExternalMidiDeviceDescriptor>,
    pub endpoints: Vec<RuntimeExternalMidiEndpointDescriptor>,
    pub summary: String,
}

impl RuntimeExternalMidiEndpointGraphSnapshot {
    pub fn unavailable() -> Self {
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Unavailable,
            graph_state: RuntimeExternalMidiGraphState::Unavailable,
            live_ownership: RuntimeExternalMidiLiveOwnershipSummary::unavailable(),
            provider_name: "runtime-unavailable".into(),
            device_count: 0,
            endpoint_count: 0,
            input_endpoint_count: 0,
            output_endpoint_count: 0,
            duplex_endpoint_count: 0,
            active_route_count: 0,
            guarded_route_count: 0,
            devices: Vec::new(),
            endpoints: Vec::new(),
            summary: "discovery=Unavailable graph=Unavailable ownership=Unavailable continuity=Unavailable parity=Unavailable provider=runtime-unavailable devices=0 endpoints=0 routes=0".into(),
        }
    }

    pub fn empty(provider_name: impl Into<String>) -> Self {
        let provider_name = provider_name.into();
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Idle,
            graph_state: RuntimeExternalMidiGraphState::Empty,
            live_ownership: RuntimeExternalMidiLiveOwnershipSummary::detached_without_backend_context(),
            provider_name: provider_name.clone(),
            device_count: 0,
            endpoint_count: 0,
            input_endpoint_count: 0,
            output_endpoint_count: 0,
            duplex_endpoint_count: 0,
            active_route_count: 0,
            guarded_route_count: 0,
            devices: Vec::new(),
            endpoints: Vec::new(),
            summary: format!(
                "discovery=Idle graph=Empty ownership=NoLiveOwnership continuity=Detached parity=Unavailable provider={} devices=0 endpoints=0 routes=0",
                provider_name,
            ),
        }
    }

    pub fn with_live_ownership_summary(
        mut self,
        linux_session: &RuntimeLinuxBackendSessionSnapshot,
        interruption_summary: &RuntimeInterruptionSummary,
    ) -> Self {
        self.live_ownership =
            RuntimeExternalMidiLiveOwnershipSummary::from_linux_session_and_interruption(
                linux_session,
                interruption_summary,
                self.graph_state,
                self.device_count,
                self.endpoint_count,
            );
        self.summary = format!(
            "discovery={:?} graph={:?} ownership={:?} continuity={:?} parity={:?} provider={} devices={} endpoints={} routes={}/{}",
            self.discovery_state,
            self.graph_state,
            self.live_ownership.ownership_posture,
            self.live_ownership.attach_continuity,
            self.live_ownership.backend_parity,
            self.provider_name,
            self.device_count,
            self.endpoint_count,
            self.active_route_count,
            self.guarded_route_count,
        );
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceGraphState {
    Unavailable,
    Empty,
    Ready,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceTransportPosture {
    Unavailable,
    InputOnly,
    FeedbackOnly,
    Duplex,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceMappingPosture {
    Unsupported,
    ObserveOnly,
    Guarded,
    Portable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceFeedbackReadiness {
    Unavailable,
    Guarded,
    Ready,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedHardwareGraphState {
    Unavailable,
    Empty,
    Ready,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeScriptingSafeDevicePolicyPosture {
    Unsupported,
    ContextOnly,
    Denied,
    Guarded,
    Portable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeGuardedFeedbackChannelPosture {
    Unavailable,
    Guarded,
    Portable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedHardwareActionClass {
    DisplayFeedback,
    MotorFeedback,
    HapticFeedback,
    BankNavigation,
    MacroTrigger,
    DeviceStateObservation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDisplayTransportPosture {
    NotPresent,
    GuardedDisplay,
    TextOnlyDisplay,
    PageAwareDisplay,
    UnavailableDisplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeDisplayContentClass {
    NoDisplayContent,
    StatusText,
    ParameterValueText,
    MeterBridgeText,
    PagedStatusView,
    GuardedVendorDisplay,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeMotorTransportPosture {
    NoMotorTransport,
    GuardedMotorTransport,
    PositionMotorTransport,
    BankAwareMotorTransport,
    UnavailableMotorTransport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeHapticTransportPosture {
    NoHapticTransport,
    GuardedHapticTransport,
    CueOnlyHapticTransport,
    StateAwareHapticTransport,
    UnavailableHapticTransport,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedControlFeedbackAuthority {
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    DeviceAdvisory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeAdvancedControlFeedbackOutcome {
    PreserveDeclaredFeedback,
    CollapseToGuardedFeedback,
    ObserveOnlyFeedback,
    BypassFeedbackTransport,
    TerminalFeedbackFailure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSceneMappingPosture {
    NoSceneMapping,
    GuardedSceneMapping,
    ContextualSceneMapping,
    PortableSceneMapping,
    UnavailableSceneMapping,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeFeedbackPagePosture {
    NoFeedbackPages,
    GuardedFeedbackPages,
    StatusFeedbackPages,
    SceneAwareFeedbackPages,
    UnavailableFeedbackPages,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeFeedbackPageClass {
    NoFeedbackPageClass,
    StatusPage,
    ParameterPage,
    MeterPage,
    ScenePage,
    GuardedVendorPage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSafeActionGraphPosture {
    NoSafeActionGraph,
    GuardedSafeActionGraph,
    TransportSafeActionGraph,
    SceneSafeActionGraph,
    UnavailableSafeActionGraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeControlSurfaceWorkflowAuthority {
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    DeviceAdvisory,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeSafeActionOutcome {
    PreserveDeclaredAction,
    CollapseToGuardedAction,
    ObserveOnlyAction,
    BypassUnsafeAction,
    TerminalActionFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceCapabilitySummary {
    pub supports_transport_control: bool,
    pub supports_mapping_input: bool,
    pub supports_feedback_output: bool,
    pub supports_widened_expression: bool,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceDeviceDescriptor {
    pub device_id: String,
    pub device_name: String,
    pub transport_posture: RuntimeControlSurfaceTransportPosture,
    pub mapping_posture: RuntimeControlSurfaceMappingPosture,
    pub feedback_readiness: RuntimeControlSurfaceFeedbackReadiness,
    pub capability: RuntimeControlSurfaceCapabilitySummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeControlSurfaceSnapshot {
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    pub graph_state: RuntimeControlSurfaceGraphState,
    pub provider_name: String,
    pub device_count: usize,
    pub mapped_device_count: usize,
    pub feedback_ready_device_count: usize,
    pub guarded_device_count: usize,
    pub devices: Vec<RuntimeControlSurfaceDeviceDescriptor>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareCapabilitySummary {
    pub supports_display_feedback: bool,
    pub supports_motor_feedback: bool,
    pub supports_haptic_feedback: bool,
    pub supports_bank_navigation: bool,
    pub supports_macro_triggers: bool,
    pub supports_device_state_observation: bool,
    pub action_classes: Vec<RuntimeAdvancedHardwareActionClass>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareDeviceDescriptor {
    pub device_id: String,
    pub device_name: String,
    pub scripting_safe_posture: RuntimeScriptingSafeDevicePolicyPosture,
    pub feedback_channel_posture: RuntimeGuardedFeedbackChannelPosture,
    pub display_transport_posture: RuntimeDisplayTransportPosture,
    pub display_content_class: RuntimeDisplayContentClass,
    pub motor_transport_posture: RuntimeMotorTransportPosture,
    pub haptic_transport_posture: RuntimeHapticTransportPosture,
    pub feedback_authority: RuntimeAdvancedControlFeedbackAuthority,
    pub feedback_outcome: RuntimeAdvancedControlFeedbackOutcome,
    pub scene_mapping_posture: RuntimeSceneMappingPosture,
    pub feedback_page_posture: RuntimeFeedbackPagePosture,
    pub feedback_page_class: RuntimeFeedbackPageClass,
    pub safe_action_graph_posture: RuntimeSafeActionGraphPosture,
    pub action_authority: RuntimeControlSurfaceWorkflowAuthority,
    pub safe_action_outcome: RuntimeSafeActionOutcome,
    pub capability: RuntimeAdvancedHardwareCapabilitySummary,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAdvancedHardwareSnapshot {
    pub discovery_state: RuntimeExternalMidiDiscoveryState,
    pub graph_state: RuntimeAdvancedHardwareGraphState,
    pub provider_name: String,
    pub device_count: usize,
    pub portable_device_count: usize,
    pub guarded_device_count: usize,
    pub context_only_device_count: usize,
    pub denied_device_count: usize,
    pub feedback_channel_device_count: usize,
    pub display_transport_device_count: usize,
    pub motor_transport_device_count: usize,
    pub haptic_transport_device_count: usize,
    pub scene_mapping_device_count: usize,
    pub feedback_page_device_count: usize,
    pub safe_action_graph_device_count: usize,
    pub devices: Vec<RuntimeAdvancedHardwareDeviceDescriptor>,
    pub summary: String,
}

impl RuntimeControlSurfaceSnapshot {
    pub fn unavailable() -> Self {
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Unavailable,
            graph_state: RuntimeControlSurfaceGraphState::Unavailable,
            provider_name: "runtime-unavailable".into(),
            device_count: 0,
            mapped_device_count: 0,
            feedback_ready_device_count: 0,
            guarded_device_count: 0,
            devices: Vec::new(),
            summary: "discovery=Unavailable graph=Unavailable provider=runtime-unavailable devices=0 mapped=0 feedback-ready=0 guarded=0".into(),
        }
    }

    pub fn empty(provider_name: impl Into<String>) -> Self {
        let provider_name = provider_name.into();
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Idle,
            graph_state: RuntimeControlSurfaceGraphState::Empty,
            provider_name: provider_name.clone(),
            device_count: 0,
            mapped_device_count: 0,
            feedback_ready_device_count: 0,
            guarded_device_count: 0,
            devices: Vec::new(),
            summary: format!(
                "discovery=Idle graph=Empty provider={} devices=0 mapped=0 feedback-ready=0 guarded=0",
                provider_name
            ),
        }
    }

    pub fn from_external_midi_snapshot(
        snapshot: &RuntimeExternalMidiEndpointGraphSnapshot,
    ) -> Self {
        if matches!(
            snapshot.discovery_state,
            RuntimeExternalMidiDiscoveryState::Unavailable
        ) || matches!(
            snapshot.graph_state,
            RuntimeExternalMidiGraphState::Unavailable
        ) {
            return Self::unavailable();
        }
        if snapshot.device_count == 0 {
            return Self::empty(snapshot.provider_name.clone());
        }

        let mut devices = Vec::with_capacity(snapshot.devices.len());
        let mut mapped_device_count = 0;
        let mut feedback_ready_device_count = 0;
        let mut guarded_device_count = 0;

        for device in &snapshot.devices {
            let endpoints = snapshot
                .endpoints
                .iter()
                .filter(|endpoint| endpoint.device_id == device.device_id)
                .collect::<Vec<_>>();
            let has_input = endpoints.iter().any(|endpoint| {
                matches!(
                    endpoint.direction,
                    RuntimeExternalMidiEndpointDirection::Input
                        | RuntimeExternalMidiEndpointDirection::Duplex
                )
            });
            let has_output = endpoints.iter().any(|endpoint| {
                matches!(
                    endpoint.direction,
                    RuntimeExternalMidiEndpointDirection::Output
                        | RuntimeExternalMidiEndpointDirection::Duplex
                )
            });
            let supports_transport_control = endpoints.iter().any(|endpoint| {
                endpoint.capability.supports_transport_clock
                    || endpoint.capability.supports_controller_events
            });
            let supports_mapping_input = endpoints.iter().any(|endpoint| {
                endpoint.capability.supports_controller_events
                    || endpoint.capability.supports_note_events
            });
            let supports_feedback_output = has_output;
            let supports_widened_expression = endpoints.iter().any(|endpoint| {
                endpoint.capability.supports_note_pressure_expression
                    || endpoint.capability.supports_note_timbre_expression
                    || endpoint.capability.supports_note_tuning_expression
                    || endpoint.capability.supports_mpe
                    || !matches!(
                        endpoint.capability.midi2_posture,
                        RuntimeControllerExpressionMidi2Posture::Unsupported
                    )
            });
            let guarded = endpoints
                .iter()
                .any(|endpoint| endpoint.capability.control_surface_guarded)
                || matches!(snapshot.graph_state, RuntimeExternalMidiGraphState::Guarded)
                || (!has_input && !has_output);

            let transport_posture = if endpoints.is_empty() {
                RuntimeControlSurfaceTransportPosture::Unavailable
            } else if guarded {
                RuntimeControlSurfaceTransportPosture::Guarded
            } else if has_input && has_output {
                RuntimeControlSurfaceTransportPosture::Duplex
            } else if has_input {
                RuntimeControlSurfaceTransportPosture::InputOnly
            } else if has_output {
                RuntimeControlSurfaceTransportPosture::FeedbackOnly
            } else {
                RuntimeControlSurfaceTransportPosture::Unavailable
            };
            let mapping_posture = if !supports_mapping_input {
                RuntimeControlSurfaceMappingPosture::Unsupported
            } else if guarded || supports_widened_expression {
                RuntimeControlSurfaceMappingPosture::Guarded
            } else if !supports_transport_control && !supports_feedback_output {
                RuntimeControlSurfaceMappingPosture::ObserveOnly
            } else {
                RuntimeControlSurfaceMappingPosture::Portable
            };
            let feedback_readiness = if !supports_feedback_output {
                RuntimeControlSurfaceFeedbackReadiness::Unavailable
            } else if guarded {
                RuntimeControlSurfaceFeedbackReadiness::Guarded
            } else {
                RuntimeControlSurfaceFeedbackReadiness::Ready
            };

            if !matches!(
                mapping_posture,
                RuntimeControlSurfaceMappingPosture::Unsupported
            ) {
                mapped_device_count += 1;
            }
            if matches!(
                feedback_readiness,
                RuntimeControlSurfaceFeedbackReadiness::Ready
            ) {
                feedback_ready_device_count += 1;
            }
            if guarded {
                guarded_device_count += 1;
            }

            let capability = RuntimeControlSurfaceCapabilitySummary {
                supports_transport_control,
                supports_mapping_input,
                supports_feedback_output,
                supports_widened_expression,
                summary: format!(
                    "transport-control={} mapping-input={} feedback-output={} widened-expression={}",
                    supports_transport_control,
                    supports_mapping_input,
                    supports_feedback_output,
                    supports_widened_expression
                ),
            };
            devices.push(RuntimeControlSurfaceDeviceDescriptor {
                device_id: device.device_id.clone(),
                device_name: device.device_name.clone(),
                transport_posture,
                mapping_posture,
                feedback_readiness,
                capability: capability.clone(),
                summary: format!(
                    "transport={:?} mapping={:?} feedback={:?} capability={}",
                    transport_posture, mapping_posture, feedback_readiness, capability.summary
                ),
            });
        }

        let graph_state = if guarded_device_count > 0 {
            RuntimeControlSurfaceGraphState::Guarded
        } else {
            RuntimeControlSurfaceGraphState::Ready
        };

        Self {
            discovery_state: snapshot.discovery_state,
            graph_state,
            provider_name: snapshot.provider_name.clone(),
            device_count: devices.len(),
            mapped_device_count,
            feedback_ready_device_count,
            guarded_device_count,
            devices,
            summary: format!(
                "discovery={:?} graph={:?} provider={} devices={} mapped={} feedback-ready={} guarded={}",
                snapshot.discovery_state,
                graph_state,
                snapshot.provider_name,
                snapshot.devices.len(),
                mapped_device_count,
                feedback_ready_device_count,
                guarded_device_count
            ),
        }
    }
}

impl RuntimeAdvancedHardwareSnapshot {
    pub fn unavailable() -> Self {
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Unavailable,
            graph_state: RuntimeAdvancedHardwareGraphState::Unavailable,
            provider_name: "runtime-unavailable".into(),
            device_count: 0,
            portable_device_count: 0,
            guarded_device_count: 0,
            context_only_device_count: 0,
            denied_device_count: 0,
            feedback_channel_device_count: 0,
            display_transport_device_count: 0,
            motor_transport_device_count: 0,
            haptic_transport_device_count: 0,
            scene_mapping_device_count: 0,
            feedback_page_device_count: 0,
            safe_action_graph_device_count: 0,
            devices: Vec::new(),
            summary: "discovery=Unavailable graph=Unavailable provider=runtime-unavailable devices=0 portable=0 guarded=0 context-only=0 denied=0 feedback-channels=0 display-transport=0 motor-transport=0 haptic-transport=0 scene-mapping=0 feedback-pages=0 safe-action-graphs=0".into(),
        }
    }

    pub fn empty(provider_name: impl Into<String>) -> Self {
        let provider_name = provider_name.into();
        Self {
            discovery_state: RuntimeExternalMidiDiscoveryState::Idle,
            graph_state: RuntimeAdvancedHardwareGraphState::Empty,
            provider_name: provider_name.clone(),
            device_count: 0,
            portable_device_count: 0,
            guarded_device_count: 0,
            context_only_device_count: 0,
            denied_device_count: 0,
            feedback_channel_device_count: 0,
            display_transport_device_count: 0,
            motor_transport_device_count: 0,
            haptic_transport_device_count: 0,
            scene_mapping_device_count: 0,
            feedback_page_device_count: 0,
            safe_action_graph_device_count: 0,
            devices: Vec::new(),
            summary: format!(
                "discovery=Idle graph=Empty provider={} devices=0 portable=0 guarded=0 context-only=0 denied=0 feedback-channels=0 display-transport=0 motor-transport=0 haptic-transport=0 scene-mapping=0 feedback-pages=0 safe-action-graphs=0",
                provider_name
            ),
        }
    }

    pub fn from_control_surface_snapshot(snapshot: &RuntimeControlSurfaceSnapshot) -> Self {
        if matches!(
            snapshot.discovery_state,
            RuntimeExternalMidiDiscoveryState::Unavailable
        ) || matches!(
            snapshot.graph_state,
            RuntimeControlSurfaceGraphState::Unavailable
        ) {
            return Self::unavailable();
        }
        if snapshot.device_count == 0 {
            return Self::empty(snapshot.provider_name.clone());
        }

        let mut devices = Vec::with_capacity(snapshot.devices.len());
        let mut portable_device_count = 0;
        let mut guarded_device_count = 0;
        let mut context_only_device_count = 0;
        let mut denied_device_count = 0;
        let mut feedback_channel_device_count = 0;
        let mut display_transport_device_count = 0;
        let mut motor_transport_device_count = 0;
        let mut haptic_transport_device_count = 0;
        let mut scene_mapping_device_count = 0;
        let mut feedback_page_device_count = 0;
        let mut safe_action_graph_device_count = 0;

        for device in &snapshot.devices {
            let scripting_safe_posture = match device.mapping_posture {
                RuntimeControlSurfaceMappingPosture::ObserveOnly => {
                    RuntimeScriptingSafeDevicePolicyPosture::ContextOnly
                }
                RuntimeControlSurfaceMappingPosture::Unsupported => {
                    RuntimeScriptingSafeDevicePolicyPosture::Denied
                }
                RuntimeControlSurfaceMappingPosture::Guarded => {
                    RuntimeScriptingSafeDevicePolicyPosture::Guarded
                }
                RuntimeControlSurfaceMappingPosture::Portable => {
                    if matches!(
                        device.feedback_readiness,
                        RuntimeControlSurfaceFeedbackReadiness::Ready
                    ) {
                        RuntimeScriptingSafeDevicePolicyPosture::Portable
                    } else {
                        RuntimeScriptingSafeDevicePolicyPosture::Guarded
                    }
                }
            };
            let feedback_channel_posture = if !device.capability.supports_feedback_output {
                RuntimeGuardedFeedbackChannelPosture::Unavailable
            } else if matches!(
                device.feedback_readiness,
                RuntimeControlSurfaceFeedbackReadiness::Ready
            ) && matches!(
                scripting_safe_posture,
                RuntimeScriptingSafeDevicePolicyPosture::Portable
            ) {
                RuntimeGuardedFeedbackChannelPosture::Portable
            } else {
                RuntimeGuardedFeedbackChannelPosture::Guarded
            };

            let display_transport_posture = if !device.capability.supports_feedback_output {
                RuntimeDisplayTransportPosture::NotPresent
            } else if matches!(
                feedback_channel_posture,
                RuntimeGuardedFeedbackChannelPosture::Portable
            ) {
                RuntimeDisplayTransportPosture::TextOnlyDisplay
            } else {
                RuntimeDisplayTransportPosture::GuardedDisplay
            };
            let display_content_class = match display_transport_posture {
                RuntimeDisplayTransportPosture::NotPresent
                | RuntimeDisplayTransportPosture::UnavailableDisplay => {
                    RuntimeDisplayContentClass::NoDisplayContent
                }
                RuntimeDisplayTransportPosture::GuardedDisplay => {
                    RuntimeDisplayContentClass::GuardedVendorDisplay
                }
                RuntimeDisplayTransportPosture::TextOnlyDisplay => {
                    RuntimeDisplayContentClass::StatusText
                }
                RuntimeDisplayTransportPosture::PageAwareDisplay => {
                    RuntimeDisplayContentClass::PagedStatusView
                }
            };
            let motor_transport_posture = RuntimeMotorTransportPosture::NoMotorTransport;
            let haptic_transport_posture = RuntimeHapticTransportPosture::NoHapticTransport;
            let feedback_authority = RuntimeAdvancedControlFeedbackAuthority::RuntimeDefault;
            let feedback_outcome = if !device.capability.supports_feedback_output {
                RuntimeAdvancedControlFeedbackOutcome::BypassFeedbackTransport
            } else if matches!(
                feedback_channel_posture,
                RuntimeGuardedFeedbackChannelPosture::Portable
            ) {
                RuntimeAdvancedControlFeedbackOutcome::PreserveDeclaredFeedback
            } else {
                RuntimeAdvancedControlFeedbackOutcome::CollapseToGuardedFeedback
            };
            let scene_mapping_posture = match device.mapping_posture {
                RuntimeControlSurfaceMappingPosture::Unsupported => {
                    RuntimeSceneMappingPosture::NoSceneMapping
                }
                RuntimeControlSurfaceMappingPosture::ObserveOnly => {
                    RuntimeSceneMappingPosture::ContextualSceneMapping
                }
                RuntimeControlSurfaceMappingPosture::Guarded => {
                    RuntimeSceneMappingPosture::GuardedSceneMapping
                }
                RuntimeControlSurfaceMappingPosture::Portable => {
                    RuntimeSceneMappingPosture::PortableSceneMapping
                }
            };
            let feedback_page_posture = match display_transport_posture {
                RuntimeDisplayTransportPosture::NotPresent => {
                    RuntimeFeedbackPagePosture::NoFeedbackPages
                }
                RuntimeDisplayTransportPosture::GuardedDisplay => {
                    RuntimeFeedbackPagePosture::GuardedFeedbackPages
                }
                RuntimeDisplayTransportPosture::TextOnlyDisplay => {
                    RuntimeFeedbackPagePosture::StatusFeedbackPages
                }
                RuntimeDisplayTransportPosture::PageAwareDisplay => {
                    RuntimeFeedbackPagePosture::SceneAwareFeedbackPages
                }
                RuntimeDisplayTransportPosture::UnavailableDisplay => {
                    RuntimeFeedbackPagePosture::UnavailableFeedbackPages
                }
            };
            let feedback_page_class = match feedback_page_posture {
                RuntimeFeedbackPagePosture::NoFeedbackPages
                | RuntimeFeedbackPagePosture::UnavailableFeedbackPages => {
                    RuntimeFeedbackPageClass::NoFeedbackPageClass
                }
                RuntimeFeedbackPagePosture::GuardedFeedbackPages => {
                    RuntimeFeedbackPageClass::GuardedVendorPage
                }
                RuntimeFeedbackPagePosture::StatusFeedbackPages => {
                    RuntimeFeedbackPageClass::StatusPage
                }
                RuntimeFeedbackPagePosture::SceneAwareFeedbackPages => {
                    RuntimeFeedbackPageClass::ScenePage
                }
            };
            let safe_action_graph_posture = match scene_mapping_posture {
                RuntimeSceneMappingPosture::NoSceneMapping => {
                    RuntimeSafeActionGraphPosture::NoSafeActionGraph
                }
                RuntimeSceneMappingPosture::GuardedSceneMapping
                | RuntimeSceneMappingPosture::ContextualSceneMapping => {
                    RuntimeSafeActionGraphPosture::GuardedSafeActionGraph
                }
                RuntimeSceneMappingPosture::PortableSceneMapping => {
                    if matches!(
                        feedback_page_posture,
                        RuntimeFeedbackPagePosture::SceneAwareFeedbackPages
                    ) {
                        RuntimeSafeActionGraphPosture::SceneSafeActionGraph
                    } else {
                        RuntimeSafeActionGraphPosture::TransportSafeActionGraph
                    }
                }
                RuntimeSceneMappingPosture::UnavailableSceneMapping => {
                    RuntimeSafeActionGraphPosture::UnavailableSafeActionGraph
                }
            };
            let action_authority = RuntimeControlSurfaceWorkflowAuthority::RuntimeDefault;
            let safe_action_outcome = match safe_action_graph_posture {
                RuntimeSafeActionGraphPosture::NoSafeActionGraph => {
                    RuntimeSafeActionOutcome::BypassUnsafeAction
                }
                RuntimeSafeActionGraphPosture::GuardedSafeActionGraph => {
                    RuntimeSafeActionOutcome::CollapseToGuardedAction
                }
                RuntimeSafeActionGraphPosture::TransportSafeActionGraph
                | RuntimeSafeActionGraphPosture::SceneSafeActionGraph => {
                    RuntimeSafeActionOutcome::PreserveDeclaredAction
                }
                RuntimeSafeActionGraphPosture::UnavailableSafeActionGraph => {
                    RuntimeSafeActionOutcome::ObserveOnlyAction
                }
            };

            let supports_display_feedback = !matches!(
                feedback_channel_posture,
                RuntimeGuardedFeedbackChannelPosture::Unavailable
            );
            let supports_motor_feedback = false;
            let supports_haptic_feedback = false;
            let supports_bank_navigation = !matches!(
                device.mapping_posture,
                RuntimeControlSurfaceMappingPosture::Unsupported
            );
            let supports_macro_triggers = device.capability.supports_transport_control;
            let supports_device_state_observation = true;

            let mut action_classes = Vec::new();
            if supports_display_feedback {
                action_classes.push(RuntimeAdvancedHardwareActionClass::DisplayFeedback);
            }
            if supports_bank_navigation {
                action_classes.push(RuntimeAdvancedHardwareActionClass::BankNavigation);
            }
            if supports_macro_triggers {
                action_classes.push(RuntimeAdvancedHardwareActionClass::MacroTrigger);
            }
            if supports_device_state_observation {
                action_classes.push(RuntimeAdvancedHardwareActionClass::DeviceStateObservation);
            }

            match scripting_safe_posture {
                RuntimeScriptingSafeDevicePolicyPosture::Portable => portable_device_count += 1,
                RuntimeScriptingSafeDevicePolicyPosture::Guarded => guarded_device_count += 1,
                RuntimeScriptingSafeDevicePolicyPosture::ContextOnly => {
                    context_only_device_count += 1
                }
                RuntimeScriptingSafeDevicePolicyPosture::Denied => denied_device_count += 1,
                RuntimeScriptingSafeDevicePolicyPosture::Unsupported => {}
            }
            if !matches!(
                feedback_channel_posture,
                RuntimeGuardedFeedbackChannelPosture::Unavailable
            ) {
                feedback_channel_device_count += 1;
            }
            if !matches!(
                display_transport_posture,
                RuntimeDisplayTransportPosture::NotPresent
                    | RuntimeDisplayTransportPosture::UnavailableDisplay
            ) {
                display_transport_device_count += 1;
            }
            if !matches!(
                motor_transport_posture,
                RuntimeMotorTransportPosture::NoMotorTransport
                    | RuntimeMotorTransportPosture::UnavailableMotorTransport
            ) {
                motor_transport_device_count += 1;
            }
            if !matches!(
                haptic_transport_posture,
                RuntimeHapticTransportPosture::NoHapticTransport
                    | RuntimeHapticTransportPosture::UnavailableHapticTransport
            ) {
                haptic_transport_device_count += 1;
            }
            if !matches!(
                scene_mapping_posture,
                RuntimeSceneMappingPosture::NoSceneMapping
                    | RuntimeSceneMappingPosture::UnavailableSceneMapping
            ) {
                scene_mapping_device_count += 1;
            }
            if !matches!(
                feedback_page_posture,
                RuntimeFeedbackPagePosture::NoFeedbackPages
                    | RuntimeFeedbackPagePosture::UnavailableFeedbackPages
            ) {
                feedback_page_device_count += 1;
            }
            if !matches!(
                safe_action_graph_posture,
                RuntimeSafeActionGraphPosture::NoSafeActionGraph
                    | RuntimeSafeActionGraphPosture::UnavailableSafeActionGraph
            ) {
                safe_action_graph_device_count += 1;
            }

            let capability = RuntimeAdvancedHardwareCapabilitySummary {
                supports_display_feedback,
                supports_motor_feedback,
                supports_haptic_feedback,
                supports_bank_navigation,
                supports_macro_triggers,
                supports_device_state_observation,
                action_classes: action_classes.clone(),
                summary: format!(
                    "display-feedback={} motor-feedback={} haptic-feedback={} bank-navigation={} macro-triggers={} device-state-observation={} action-classes={}",
                    supports_display_feedback,
                    supports_motor_feedback,
                    supports_haptic_feedback,
                    supports_bank_navigation,
                    supports_macro_triggers,
                    supports_device_state_observation,
                    action_classes.len()
                ),
            };
            devices.push(RuntimeAdvancedHardwareDeviceDescriptor {
                device_id: device.device_id.clone(),
                device_name: device.device_name.clone(),
                scripting_safe_posture,
                feedback_channel_posture,
                display_transport_posture,
                display_content_class,
                motor_transport_posture,
                haptic_transport_posture,
                feedback_authority,
                feedback_outcome,
                scene_mapping_posture,
                feedback_page_posture,
                feedback_page_class,
                safe_action_graph_posture,
                action_authority,
                safe_action_outcome,
                capability: capability.clone(),
                summary: format!(
                    "policy={:?} feedback={:?} display={:?}/{:?} motor={:?} haptic={:?} feedback_authority={:?} feedback_outcome={:?} scene={:?} page={:?}/{:?} action_graph={:?} action_authority={:?} action_outcome={:?} capability={}",
                    scripting_safe_posture,
                    feedback_channel_posture,
                    display_transport_posture,
                    display_content_class,
                    motor_transport_posture,
                    haptic_transport_posture,
                    feedback_authority,
                    feedback_outcome,
                    scene_mapping_posture,
                    feedback_page_posture,
                    feedback_page_class,
                    safe_action_graph_posture,
                    action_authority,
                    safe_action_outcome,
                    capability.summary
                ),
            });
        }

        let graph_state =
            if guarded_device_count > 0 || context_only_device_count > 0 || denied_device_count > 0
            {
                RuntimeAdvancedHardwareGraphState::Guarded
            } else {
                RuntimeAdvancedHardwareGraphState::Ready
            };

        let device_count = devices.len();
        let summary = format!(
            "discovery={:?} graph={:?} provider={} devices={} portable={} guarded={} context-only={} denied={} feedback-channels={} display-transport={} motor-transport={} haptic-transport={} scene-mapping={} feedback-pages={} safe-action-graphs={}",
            snapshot.discovery_state,
            graph_state,
            snapshot.provider_name,
            device_count,
            portable_device_count,
            guarded_device_count,
            context_only_device_count,
            denied_device_count,
            feedback_channel_device_count,
            display_transport_device_count,
            motor_transport_device_count,
            haptic_transport_device_count,
            scene_mapping_device_count,
            feedback_page_device_count,
            safe_action_graph_device_count
        );

        Self {
            discovery_state: snapshot.discovery_state,
            graph_state,
            provider_name: snapshot.provider_name.clone(),
            device_count,
            portable_device_count,
            guarded_device_count,
            context_only_device_count,
            denied_device_count,
            feedback_channel_device_count,
            display_transport_device_count,
            motor_transport_device_count,
            haptic_transport_device_count,
            scene_mapping_device_count,
            feedback_page_device_count,
            safe_action_graph_device_count,
            devices,
            summary,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendIdentity {
    NotLinux,
    Unavailable,
    Alsa,
    Jack,
    PipeWire,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendPortabilityBand {
    Portable,
    Guarded,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendClockingParityBand {
    Portable,
    Guarded,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendDuplexParityState {
    Aligned,
    Guarded,
    Partial,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxAudioBackendEndpointTopologyParityState {
    Portable,
    Guarded,
    Partial,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionOwnership {
    NotLinux,
    Unavailable,
    RuntimeOwnedDirect,
    HostBrokeredCallback,
    BackendManagedGraph,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionLifecycleState {
    NotLinux,
    Unavailable,
    Claimable,
    Attached,
    Running,
    Interrupted,
    Recovering,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendDeviceClaimPosture {
    NotLinux,
    Unavailable,
    Unclaimed,
    DirectClaim,
    SharedGraph,
    Lost,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendSessionRole {
    NotLinux,
    Unavailable,
    PrimaryAudioIo,
    MonitoringCapable,
    OfflineUnavailable,
    FallbackContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeLinuxBackendOwnershipFallbackState {
    NotLinux,
    Unavailable,
    Direct,
    BackendManagedGuarded,
    Reacquiring,
    RecoveryConstrained,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLinuxBackendSessionSnapshot {
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub backend_name: String,
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    pub ownership: RuntimeLinuxBackendSessionOwnership,
    pub lifecycle_state: RuntimeLinuxBackendSessionLifecycleState,
    pub device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture,
    pub session_role: RuntimeLinuxBackendSessionRole,
    pub ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState,
    pub device_id: String,
    pub device_name: String,
    pub stream_state: RuntimeHostAudioStreamState,
    pub backend_health: BackendHealth,
    pub simulated: bool,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}

impl RuntimeLinuxBackendSessionSnapshot {
    pub fn unavailable() -> Self {
        Self {
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            backend_name: "runtime-unavailable".into(),
            portability_band: RuntimeLinuxAudioBackendPortabilityBand::Unsupported,
            ownership: RuntimeLinuxBackendSessionOwnership::Unavailable,
            lifecycle_state: RuntimeLinuxBackendSessionLifecycleState::Unavailable,
            device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture::Unavailable,
            session_role: RuntimeLinuxBackendSessionRole::Unavailable,
            ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState::Unavailable,
            device_id: "runtime:unavailable".into(),
            device_name: "Unavailable Linux Backend Session".into(),
            stream_state: RuntimeHostAudioStreamState::Stopped,
            backend_health: BackendHealth::Healthy,
            simulated: false,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "backend=Unavailable ownership=Unavailable lifecycle=Unavailable claim=Unavailable role=Unavailable fallback=Unavailable"
                    .into(),
        }
    }

    pub fn from_host_io(host_io: &RuntimeHostIoSummary) -> Self {
        let backend_identity = host_io.hardware.linux_backend_identity;
        if backend_identity == RuntimeLinuxAudioBackendIdentity::NotLinux {
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                ownership: RuntimeLinuxBackendSessionOwnership::NotLinux,
                lifecycle_state: RuntimeLinuxBackendSessionLifecycleState::NotLinux,
                device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture::NotLinux,
                session_role: RuntimeLinuxBackendSessionRole::NotLinux,
                ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState::NotLinux,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                stream_state: host_io.audio_pump.stream_state,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                device_loss_count: host_io.hardware.device_loss_count,
                restart_attempt_count: host_io.hardware.restart_attempt_count,
                restart_failure_count: host_io.hardware.restart_failure_count,
                summary: format!(
                    "backend={:?} ownership=NotLinux lifecycle=NotLinux claim=NotLinux role=NotLinux fallback=NotLinux",
                    backend_identity
                ),
            };
        }

        if matches!(
            backend_identity,
            RuntimeLinuxAudioBackendIdentity::Unavailable
                | RuntimeLinuxAudioBackendIdentity::Unsupported
        ) {
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                ownership: RuntimeLinuxBackendSessionOwnership::Unavailable,
                lifecycle_state: RuntimeLinuxBackendSessionLifecycleState::Unavailable,
                device_claim_posture: RuntimeLinuxBackendDeviceClaimPosture::Unavailable,
                session_role: RuntimeLinuxBackendSessionRole::Unavailable,
                ownership_fallback: RuntimeLinuxBackendOwnershipFallbackState::Unavailable,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                stream_state: host_io.audio_pump.stream_state,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                device_loss_count: host_io.hardware.device_loss_count,
                restart_attempt_count: host_io.hardware.restart_attempt_count,
                restart_failure_count: host_io.hardware.restart_failure_count,
                summary: format!(
                    "backend={:?} ownership=Unavailable lifecycle=Unavailable claim=Unavailable role=Unavailable fallback=Unavailable",
                    backend_identity
                ),
            };
        }

        let recovering = host_io.hardware.device_loss_count > 0
            || host_io.hardware.restart_attempt_count > 0
            || matches!(
                host_io.hardware.backend_health,
                BackendHealth::Degraded | BackendHealth::Recovering
            )
            || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted;
        let release_like = host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Stopped
            && matches!(
                host_io.clocking.endpoint_topology,
                RuntimeHostEndpointTopology::Unconfigured
            );
        let ownership = if release_like {
            RuntimeLinuxBackendSessionOwnership::Unavailable
        } else {
            match host_io.clocking.ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => {
                    RuntimeLinuxBackendSessionOwnership::HostBrokeredCallback
                }
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
                }
            }
        };
        let lifecycle_state = if release_like {
            RuntimeLinuxBackendSessionLifecycleState::Released
        } else if recovering {
            RuntimeLinuxBackendSessionLifecycleState::Recovering
        } else {
            match host_io.audio_pump.stream_state {
                RuntimeHostAudioStreamState::Running => {
                    RuntimeLinuxBackendSessionLifecycleState::Running
                }
                RuntimeHostAudioStreamState::Stopped => {
                    RuntimeLinuxBackendSessionLifecycleState::Claimable
                }
                RuntimeHostAudioStreamState::Faulted => {
                    RuntimeLinuxBackendSessionLifecycleState::Interrupted
                }
            }
        };
        let device_claim_posture = if release_like {
            RuntimeLinuxBackendDeviceClaimPosture::Released
        } else if host_io.hardware.device_loss_count > 0 {
            RuntimeLinuxBackendDeviceClaimPosture::Lost
        } else if host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Stopped {
            RuntimeLinuxBackendDeviceClaimPosture::Unclaimed
        } else {
            match host_io.clocking.ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => {
                    RuntimeLinuxBackendDeviceClaimPosture::DirectClaim
                }
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    RuntimeLinuxBackendDeviceClaimPosture::SharedGraph
                }
            }
        };
        let fallback_active =
            host_io.clocking.fallback_state != RuntimeHostClockFallbackState::Direct;
        let session_role = if release_like {
            RuntimeLinuxBackendSessionRole::OfflineUnavailable
        } else if recovering || fallback_active {
            RuntimeLinuxBackendSessionRole::FallbackContinuation
        } else if matches!(
            host_io.clocking.endpoint_topology,
            RuntimeHostEndpointTopology::OutputOnly
        ) {
            RuntimeLinuxBackendSessionRole::MonitoringCapable
        } else {
            RuntimeLinuxBackendSessionRole::PrimaryAudioIo
        };
        let ownership_fallback = if release_like {
            RuntimeLinuxBackendOwnershipFallbackState::Unavailable
        } else if host_io.hardware.restart_failure_count > 0
            || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted
        {
            RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained
        } else if recovering {
            RuntimeLinuxBackendOwnershipFallbackState::Reacquiring
        } else if host_io.clocking.ownership
            == RuntimeHostLifecycleOwnership::BackendManagedCallback
        {
            RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
        } else {
            RuntimeLinuxBackendOwnershipFallbackState::Direct
        };

        Self {
            backend_identity,
            backend_name: host_io.hardware.backend_name.clone(),
            portability_band: host_io.hardware.linux_backend_portability,
            ownership,
            lifecycle_state,
            device_claim_posture,
            session_role,
            ownership_fallback,
            device_id: host_io.hardware.device_id.clone(),
            device_name: host_io.hardware.device_name.clone(),
            stream_state: host_io.audio_pump.stream_state,
            backend_health: host_io.hardware.backend_health,
            simulated: host_io.hardware.simulated,
            device_loss_count: host_io.hardware.device_loss_count,
            restart_attempt_count: host_io.hardware.restart_attempt_count,
            restart_failure_count: host_io.hardware.restart_failure_count,
            summary: format!(
                "backend={:?} ownership={:?} lifecycle={:?} claim={:?} role={:?} fallback={:?}",
                backend_identity,
                ownership,
                lifecycle_state,
                device_claim_posture,
                session_role,
                ownership_fallback
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaSessionRoleParity {
    NotPipeWireOrAlsa,
    Unavailable,
    PrimaryAudioIo,
    MonitoringCapable,
    OfflineUnavailable,
    FallbackContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaDeviceClaimParity {
    NotPipeWireOrAlsa,
    Unavailable,
    NoClaim,
    DirectClaim,
    SharedGraph,
    Lost,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaStreamPolicyParity {
    NotPipeWireOrAlsa,
    Unavailable,
    DirectHostCallback,
    BackendManagedGraph,
    Restarting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimePipeWireAlsaGuardedParityState {
    NotPipeWireOrAlsa,
    Unavailable,
    Direct,
    BackendManaged,
    ClockGuarded,
    TransferGuarded,
    RecoveryGuarded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePipeWireAlsaParitySnapshot {
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub backend_name: String,
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    pub session_role_parity: RuntimePipeWireAlsaSessionRoleParity,
    pub device_claim_parity: RuntimePipeWireAlsaDeviceClaimParity,
    pub stream_policy_parity: RuntimePipeWireAlsaStreamPolicyParity,
    pub guarded_state: RuntimePipeWireAlsaGuardedParityState,
    pub lifecycle_ownership: RuntimeHostLifecycleOwnership,
    pub restart_policy: RuntimeHostRestartPolicy,
    pub clock_domain: RuntimeHostClockDomain,
    pub fallback_state: RuntimeHostClockFallbackState,
    pub device_id: String,
    pub device_name: String,
    pub stream_state: RuntimeHostAudioStreamState,
    pub backend_health: BackendHealth,
    pub simulated: bool,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub summary: String,
}

impl RuntimePipeWireAlsaParitySnapshot {
    pub fn unavailable() -> Self {
        Self {
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            backend_name: "runtime-unavailable".into(),
            portability_band: RuntimeLinuxAudioBackendPortabilityBand::Unsupported,
            session_role_parity: RuntimePipeWireAlsaSessionRoleParity::Unavailable,
            device_claim_parity: RuntimePipeWireAlsaDeviceClaimParity::Unavailable,
            stream_policy_parity: RuntimePipeWireAlsaStreamPolicyParity::Unavailable,
            guarded_state: RuntimePipeWireAlsaGuardedParityState::Unavailable,
            lifecycle_ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
            restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
            clock_domain: RuntimeHostClockDomain::Degraded,
            fallback_state: RuntimeHostClockFallbackState::Unconfigured,
            device_id: "runtime:unavailable".into(),
            device_name: "Unavailable PipeWire/ALSA Parity".into(),
            stream_state: RuntimeHostAudioStreamState::Stopped,
            backend_health: BackendHealth::Healthy,
            simulated: false,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            summary:
                "backend=Unavailable role=Unavailable claim=Unavailable policy=Unavailable guard=Unavailable"
                    .into(),
        }
    }

    pub fn from_host_io(host_io: &RuntimeHostIoSummary) -> Self {
        let linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(host_io);
        Self::from_host_io_and_linux_session(host_io, &linux_session)
    }

    pub fn from_host_io_and_linux_session(
        host_io: &RuntimeHostIoSummary,
        linux_session: &RuntimeLinuxBackendSessionSnapshot,
    ) -> Self {
        let backend_identity = linux_session.backend_identity;
        let targets_pipewire_or_alsa = matches!(
            backend_identity,
            RuntimeLinuxAudioBackendIdentity::Alsa | RuntimeLinuxAudioBackendIdentity::PipeWire
        );
        if !targets_pipewire_or_alsa {
            let unavailable = matches!(
                backend_identity,
                RuntimeLinuxAudioBackendIdentity::Unavailable
                    | RuntimeLinuxAudioBackendIdentity::Unsupported
            );
            let session_role_parity = if unavailable {
                RuntimePipeWireAlsaSessionRoleParity::Unavailable
            } else {
                RuntimePipeWireAlsaSessionRoleParity::NotPipeWireOrAlsa
            };
            let device_claim_parity = if unavailable {
                RuntimePipeWireAlsaDeviceClaimParity::Unavailable
            } else {
                RuntimePipeWireAlsaDeviceClaimParity::NotPipeWireOrAlsa
            };
            let stream_policy_parity = if unavailable {
                RuntimePipeWireAlsaStreamPolicyParity::Unavailable
            } else {
                RuntimePipeWireAlsaStreamPolicyParity::NotPipeWireOrAlsa
            };
            let guarded_state = if unavailable {
                RuntimePipeWireAlsaGuardedParityState::Unavailable
            } else {
                RuntimePipeWireAlsaGuardedParityState::NotPipeWireOrAlsa
            };
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                session_role_parity,
                device_claim_parity,
                stream_policy_parity,
                guarded_state,
                lifecycle_ownership: host_io.clocking.ownership,
                restart_policy: host_io.clocking.restart_policy,
                clock_domain: host_io.clocking.clock_domain,
                fallback_state: host_io.clocking.fallback_state,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                stream_state: host_io.audio_pump.stream_state,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                device_loss_count: host_io.hardware.device_loss_count,
                restart_attempt_count: host_io.hardware.restart_attempt_count,
                restart_failure_count: host_io.hardware.restart_failure_count,
                summary: format!(
                    "backend={:?} role={:?} claim={:?} policy={:?} guard={:?}",
                    backend_identity,
                    session_role_parity,
                    device_claim_parity,
                    stream_policy_parity,
                    guarded_state
                ),
            };
        }

        let session_role_parity = match linux_session.session_role {
            RuntimeLinuxBackendSessionRole::PrimaryAudioIo => {
                RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
            }
            RuntimeLinuxBackendSessionRole::MonitoringCapable => {
                RuntimePipeWireAlsaSessionRoleParity::MonitoringCapable
            }
            RuntimeLinuxBackendSessionRole::OfflineUnavailable => {
                RuntimePipeWireAlsaSessionRoleParity::OfflineUnavailable
            }
            RuntimeLinuxBackendSessionRole::FallbackContinuation => {
                RuntimePipeWireAlsaSessionRoleParity::FallbackContinuation
            }
            RuntimeLinuxBackendSessionRole::Unavailable => {
                RuntimePipeWireAlsaSessionRoleParity::Unavailable
            }
            RuntimeLinuxBackendSessionRole::NotLinux => {
                RuntimePipeWireAlsaSessionRoleParity::NotPipeWireOrAlsa
            }
        };
        let device_claim_parity = match linux_session.device_claim_posture {
            RuntimeLinuxBackendDeviceClaimPosture::Unclaimed => {
                RuntimePipeWireAlsaDeviceClaimParity::NoClaim
            }
            RuntimeLinuxBackendDeviceClaimPosture::DirectClaim => {
                RuntimePipeWireAlsaDeviceClaimParity::DirectClaim
            }
            RuntimeLinuxBackendDeviceClaimPosture::SharedGraph => {
                RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
            }
            RuntimeLinuxBackendDeviceClaimPosture::Lost => {
                RuntimePipeWireAlsaDeviceClaimParity::Lost
            }
            RuntimeLinuxBackendDeviceClaimPosture::Released => {
                RuntimePipeWireAlsaDeviceClaimParity::Released
            }
            RuntimeLinuxBackendDeviceClaimPosture::Unavailable => {
                RuntimePipeWireAlsaDeviceClaimParity::Unavailable
            }
            RuntimeLinuxBackendDeviceClaimPosture::NotLinux => {
                RuntimePipeWireAlsaDeviceClaimParity::NotPipeWireOrAlsa
            }
        };

        let recovering = matches!(
            linux_session.lifecycle_state,
            RuntimeLinuxBackendSessionLifecycleState::Interrupted
                | RuntimeLinuxBackendSessionLifecycleState::Recovering
        ) || matches!(
            linux_session.ownership_fallback,
            RuntimeLinuxBackendOwnershipFallbackState::Reacquiring
                | RuntimeLinuxBackendOwnershipFallbackState::RecoveryConstrained
        ) || host_io.hardware.restart_attempt_count > 0
            || host_io.hardware.restart_failure_count > 0
            || host_io.hardware.device_loss_count > 0
            || host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted;
        let unavailable = matches!(
            session_role_parity,
            RuntimePipeWireAlsaSessionRoleParity::OfflineUnavailable
                | RuntimePipeWireAlsaSessionRoleParity::Unavailable
        );
        let stream_policy_parity = if unavailable {
            RuntimePipeWireAlsaStreamPolicyParity::Unavailable
        } else if recovering {
            RuntimePipeWireAlsaStreamPolicyParity::Restarting
        } else {
            match host_io.clocking.ownership {
                RuntimeHostLifecycleOwnership::HostDrivenCallback => {
                    RuntimePipeWireAlsaStreamPolicyParity::DirectHostCallback
                }
                RuntimeHostLifecycleOwnership::BackendManagedCallback => {
                    RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
                }
            }
        };

        let transfer_guarded = host_io.audio_pump.transfer_policy.max_callback_frames
            < host_io.hardware.buffer_size
            || host_io.audio_pump.transfer_policy.max_transfer_channels
                < host_io
                    .hardware
                    .input_channels
                    .max(host_io.hardware.output_channels)
            || !host_io
                .audio_pump
                .transfer_policy
                .zero_fill_unwritten_output;
        let clock_guarded = host_io.clocking.clock_domain != RuntimeHostClockDomain::SameClock
            || host_io.clocking.fallback_state != RuntimeHostClockFallbackState::Direct
            || host_io.clocking.transition_state != RuntimeHostClockTransitionState::Stable
            || host_io.clocking.drift_state != RuntimeHostClockDriftState::Stable
            || host_io.clocking.discontinuity_state
                != RuntimeHostClockDiscontinuityState::Continuous;
        let guarded_state = if unavailable {
            RuntimePipeWireAlsaGuardedParityState::Unavailable
        } else if recovering {
            RuntimePipeWireAlsaGuardedParityState::RecoveryGuarded
        } else if transfer_guarded {
            RuntimePipeWireAlsaGuardedParityState::TransferGuarded
        } else if clock_guarded {
            RuntimePipeWireAlsaGuardedParityState::ClockGuarded
        } else if host_io.clocking.ownership
            == RuntimeHostLifecycleOwnership::BackendManagedCallback
        {
            RuntimePipeWireAlsaGuardedParityState::BackendManaged
        } else {
            RuntimePipeWireAlsaGuardedParityState::Direct
        };

        Self {
            backend_identity,
            backend_name: host_io.hardware.backend_name.clone(),
            portability_band: host_io.hardware.linux_backend_portability,
            session_role_parity,
            device_claim_parity,
            stream_policy_parity,
            guarded_state,
            lifecycle_ownership: host_io.clocking.ownership,
            restart_policy: host_io.clocking.restart_policy,
            clock_domain: host_io.clocking.clock_domain,
            fallback_state: host_io.clocking.fallback_state,
            device_id: host_io.hardware.device_id.clone(),
            device_name: host_io.hardware.device_name.clone(),
            stream_state: host_io.audio_pump.stream_state,
            backend_health: host_io.hardware.backend_health,
            simulated: host_io.hardware.simulated,
            device_loss_count: host_io.hardware.device_loss_count,
            restart_attempt_count: host_io.hardware.restart_attempt_count,
            restart_failure_count: host_io.hardware.restart_failure_count,
            summary: format!(
                "backend={:?} role={:?} claim={:?} policy={:?} guard={:?}",
                backend_identity,
                session_role_parity,
                device_claim_parity,
                stream_policy_parity,
                guarded_state
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackTransportPosture {
    NotJack,
    Unavailable,
    Detached,
    FollowingExternal,
    RuntimeLed,
    Guarded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackGraphCoordinationState {
    NotJack,
    Unavailable,
    NotAttached,
    AttachedStable,
    AttachedGuarded,
    Recovering,
    Released,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackClientRole {
    NotJack,
    Unavailable,
    PrimaryAudioIo,
    MonitoringCapable,
    TransportFollower,
    FallbackContinuation,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RuntimeJackGuardedCoordinationState {
    NotJack,
    Unavailable,
    Direct,
    TransportGuarded,
    GraphGuarded,
    Recovering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeJackCoordinationSnapshot {
    pub backend_identity: RuntimeLinuxAudioBackendIdentity,
    pub backend_name: String,
    pub portability_band: RuntimeLinuxAudioBackendPortabilityBand,
    pub transport_posture: RuntimeJackTransportPosture,
    pub graph_state: RuntimeJackGraphCoordinationState,
    pub client_role: RuntimeJackClientRole,
    pub guarded_state: RuntimeJackGuardedCoordinationState,
    pub device_id: String,
    pub device_name: String,
    pub session_state: TransportSessionState,
    pub currently_attached: bool,
    pub heartbeat_freshness: TransportHeartbeatFreshness,
    pub dispatch_state: TransportDispatchState,
    pub attach_events: usize,
    pub detach_requested_events: usize,
    pub detached_events: usize,
    pub backend_health: BackendHealth,
    pub simulated: bool,
    pub summary: String,
}

impl RuntimeJackCoordinationSnapshot {
    pub fn unavailable() -> Self {
        Self {
            backend_identity: RuntimeLinuxAudioBackendIdentity::Unavailable,
            backend_name: "runtime-unavailable".into(),
            portability_band: RuntimeLinuxAudioBackendPortabilityBand::Unsupported,
            transport_posture: RuntimeJackTransportPosture::Unavailable,
            graph_state: RuntimeJackGraphCoordinationState::Unavailable,
            client_role: RuntimeJackClientRole::Unavailable,
            guarded_state: RuntimeJackGuardedCoordinationState::Unavailable,
            device_id: "runtime:unavailable".into(),
            device_name: "Unavailable JACK Coordination".into(),
            session_state: TransportSessionState::Detached,
            currently_attached: false,
            heartbeat_freshness: TransportHeartbeatFreshness::Unknown,
            dispatch_state: TransportDispatchState::Idle,
            attach_events: 0,
            detach_requested_events: 0,
            detached_events: 0,
            backend_health: BackendHealth::Healthy,
            simulated: false,
            summary:
                "backend=Unavailable transport=Unavailable graph=Unavailable role=Unavailable guard=Unavailable"
                    .into(),
        }
    }

    pub fn from_host_io_and_transport_session(
        host_io: &RuntimeHostIoSummary,
        transport_session: &TransportSessionSummary,
    ) -> Self {
        let linux_session = RuntimeLinuxBackendSessionSnapshot::from_host_io(host_io);
        let backend_identity = host_io.hardware.linux_backend_identity;

        if backend_identity != RuntimeLinuxAudioBackendIdentity::Jack {
            let unavailable = matches!(
                backend_identity,
                RuntimeLinuxAudioBackendIdentity::Unavailable
                    | RuntimeLinuxAudioBackendIdentity::Unsupported
            );
            let transport_posture = if unavailable {
                RuntimeJackTransportPosture::Unavailable
            } else {
                RuntimeJackTransportPosture::NotJack
            };
            let graph_state = if unavailable {
                RuntimeJackGraphCoordinationState::Unavailable
            } else {
                RuntimeJackGraphCoordinationState::NotJack
            };
            let client_role = if unavailable {
                RuntimeJackClientRole::Unavailable
            } else {
                RuntimeJackClientRole::NotJack
            };
            let guarded_state = if unavailable {
                RuntimeJackGuardedCoordinationState::Unavailable
            } else {
                RuntimeJackGuardedCoordinationState::NotJack
            };
            return Self {
                backend_identity,
                backend_name: host_io.hardware.backend_name.clone(),
                portability_band: host_io.hardware.linux_backend_portability,
                transport_posture,
                graph_state,
                client_role,
                guarded_state,
                device_id: host_io.hardware.device_id.clone(),
                device_name: host_io.hardware.device_name.clone(),
                session_state: TransportSessionState::Detached,
                currently_attached: false,
                heartbeat_freshness: TransportHeartbeatFreshness::Unknown,
                dispatch_state: TransportDispatchState::Idle,
                attach_events: 0,
                detach_requested_events: 0,
                detached_events: 0,
                backend_health: host_io.hardware.backend_health,
                simulated: host_io.hardware.simulated,
                summary: format!(
                    "backend={:?} transport={:?} graph={:?} role={:?} guard={:?}",
                    backend_identity, transport_posture, graph_state, client_role, guarded_state
                ),
            };
        }

        let recovering = host_io.audio_pump.stream_state == RuntimeHostAudioStreamState::Faulted
            || matches!(
                host_io.hardware.backend_health,
                BackendHealth::Degraded | BackendHealth::Recovering
            )
            || host_io.hardware.device_loss_count > 0
            || host_io.hardware.restart_attempt_count > 0
            || host_io.hardware.restart_failure_count > 0;
        let graph_attached = matches!(
            host_io.audio_pump.stream_state,
            RuntimeHostAudioStreamState::Running | RuntimeHostAudioStreamState::Stopped
        );
        let released = !graph_attached
            && !transport_session.currently_attached
            && transport_session.attach_events > 0
            && transport_session.detached_events > 0;

        let transport_posture = if recovering {
            RuntimeJackTransportPosture::Guarded
        } else if !transport_session.currently_attached {
            RuntimeJackTransportPosture::Detached
        } else if matches!(
            transport_session.dispatch_state,
            TransportDispatchState::Requested | TransportDispatchState::Completed
        ) || matches!(
            transport_session.heartbeat_freshness,
            TransportHeartbeatFreshness::Requested | TransportHeartbeatFreshness::Fresh
        ) {
            RuntimeJackTransportPosture::FollowingExternal
        } else {
            RuntimeJackTransportPosture::Guarded
        };

        let graph_state = if recovering {
            RuntimeJackGraphCoordinationState::Recovering
        } else if released {
            RuntimeJackGraphCoordinationState::Released
        } else if !graph_attached {
            RuntimeJackGraphCoordinationState::NotAttached
        } else if linux_session.ownership
            == RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
            || linux_session.ownership_fallback
                == RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
            || host_io.hardware.linux_backend_portability
                == RuntimeLinuxAudioBackendPortabilityBand::Guarded
        {
            RuntimeJackGraphCoordinationState::AttachedGuarded
        } else {
            RuntimeJackGraphCoordinationState::AttachedStable
        };

        let client_role = if linux_session.session_role
            == RuntimeLinuxBackendSessionRole::FallbackContinuation
        {
            RuntimeJackClientRole::FallbackContinuation
        } else if transport_session.currently_attached {
            RuntimeJackClientRole::TransportFollower
        } else if linux_session.session_role == RuntimeLinuxBackendSessionRole::MonitoringCapable {
            RuntimeJackClientRole::MonitoringCapable
        } else {
            RuntimeJackClientRole::PrimaryAudioIo
        };

        let guarded_state = if recovering {
            RuntimeJackGuardedCoordinationState::Recovering
        } else if transport_session.currently_attached {
            RuntimeJackGuardedCoordinationState::TransportGuarded
        } else if graph_state == RuntimeJackGraphCoordinationState::AttachedGuarded {
            RuntimeJackGuardedCoordinationState::GraphGuarded
        } else {
            RuntimeJackGuardedCoordinationState::Direct
        };

        Self {
            backend_identity,
            backend_name: host_io.hardware.backend_name.clone(),
            portability_band: host_io.hardware.linux_backend_portability,
            transport_posture,
            graph_state,
            client_role,
            guarded_state,
            device_id: host_io.hardware.device_id.clone(),
            device_name: host_io.hardware.device_name.clone(),
            session_state: transport_session.current_state,
            currently_attached: transport_session.currently_attached,
            heartbeat_freshness: transport_session.heartbeat_freshness,
            dispatch_state: transport_session.dispatch_state,
            attach_events: transport_session.attach_events,
            detach_requested_events: transport_session.detach_requested_events,
            detached_events: transport_session.detached_events,
            backend_health: host_io.hardware.backend_health,
            simulated: host_io.hardware.simulated,
            summary: format!(
                "backend={:?} transport={:?} graph={:?} role={:?} guard={:?} session={:?}/{} heartbeat={:?} dispatch={:?}",
                backend_identity,
                transport_posture,
                graph_state,
                client_role,
                guarded_state,
                transport_session.current_state,
                transport_session.currently_attached,
                transport_session.heartbeat_freshness,
                transport_session.dispatch_state,
            ),
        }
    }
}

impl RuntimeHostIoSummary {
    pub fn classify_linux_clocking_parity(
        linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
        backend_health: BackendHealth,
        stream_state: RuntimeHostAudioStreamState,
        clock_domain: RuntimeHostClockDomain,
        fallback_state: RuntimeHostClockFallbackState,
        transition_state: RuntimeHostClockTransitionState,
        drift_state: RuntimeHostClockDriftState,
        discontinuity_state: RuntimeHostClockDiscontinuityState,
    ) -> RuntimeLinuxAudioBackendClockingParityBand {
        match linux_backend_identity {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if !matches!(backend_health, BackendHealth::Healthy)
                    || stream_state == RuntimeHostAudioStreamState::Faulted
                    || clock_domain != RuntimeHostClockDomain::SameClock
                    || fallback_state != RuntimeHostClockFallbackState::Direct
                    || transition_state != RuntimeHostClockTransitionState::Stable
                    || drift_state != RuntimeHostClockDriftState::Stable
                    || discontinuity_state != RuntimeHostClockDiscontinuityState::Continuous
                {
                    RuntimeLinuxAudioBackendClockingParityBand::Guarded
                } else {
                    RuntimeLinuxAudioBackendClockingParityBand::Portable
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendClockingParityBand::Unsupported
            }
        }
    }

    pub fn classify_linux_duplex_parity(
        linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
        backend_health: BackendHealth,
        stream_state: RuntimeHostAudioStreamState,
        clock_domain: RuntimeHostClockDomain,
        fallback_state: RuntimeHostClockFallbackState,
        transition_state: RuntimeHostClockTransitionState,
        duplex_mismatch_state: RuntimeHostDuplexMismatchState,
        endpoint_topology: RuntimeHostEndpointTopology,
        partial_availability: bool,
    ) -> RuntimeLinuxAudioBackendDuplexParityState {
        match linux_backend_identity {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if matches!(endpoint_topology, RuntimeHostEndpointTopology::Unconfigured) {
                    RuntimeLinuxAudioBackendDuplexParityState::Unsupported
                } else if partial_availability
                    || matches!(
                        endpoint_topology,
                        RuntimeHostEndpointTopology::OutputOnly
                            | RuntimeHostEndpointTopology::InputOnly
                    )
                {
                    RuntimeLinuxAudioBackendDuplexParityState::Partial
                } else if !matches!(backend_health, BackendHealth::Healthy)
                    || stream_state == RuntimeHostAudioStreamState::Faulted
                    || clock_domain != RuntimeHostClockDomain::SameClock
                    || fallback_state != RuntimeHostClockFallbackState::Direct
                    || transition_state != RuntimeHostClockTransitionState::Stable
                    || !matches!(
                        duplex_mismatch_state,
                        RuntimeHostDuplexMismatchState::NotApplicable
                            | RuntimeHostDuplexMismatchState::Aligned
                    )
                    || endpoint_topology == RuntimeHostEndpointTopology::Aggregate
                {
                    RuntimeLinuxAudioBackendDuplexParityState::Guarded
                } else {
                    RuntimeLinuxAudioBackendDuplexParityState::Aligned
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendDuplexParityState::Unsupported
            }
        }
    }

    pub fn classify_linux_endpoint_topology_parity(
        linux_backend_identity: RuntimeLinuxAudioBackendIdentity,
        backend_health: BackendHealth,
        transition_state: RuntimeHostClockTransitionState,
        discontinuity_state: RuntimeHostClockDiscontinuityState,
        duplex_mismatch_state: RuntimeHostDuplexMismatchState,
        endpoint_topology: RuntimeHostEndpointTopology,
        partial_availability: bool,
    ) -> RuntimeLinuxAudioBackendEndpointTopologyParityState {
        match linux_backend_identity {
            RuntimeLinuxAudioBackendIdentity::Alsa
            | RuntimeLinuxAudioBackendIdentity::Jack
            | RuntimeLinuxAudioBackendIdentity::PipeWire => {
                if endpoint_topology == RuntimeHostEndpointTopology::Unconfigured {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
                } else if partial_availability {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Partial
                } else if !matches!(backend_health, BackendHealth::Healthy)
                    || transition_state != RuntimeHostClockTransitionState::Stable
                    || discontinuity_state != RuntimeHostClockDiscontinuityState::Continuous
                    || endpoint_topology == RuntimeHostEndpointTopology::Aggregate
                    || duplex_mismatch_state == RuntimeHostDuplexMismatchState::CrossClockDiverged
                {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Guarded
                } else {
                    RuntimeLinuxAudioBackendEndpointTopologyParityState::Portable
                }
            }
            RuntimeLinuxAudioBackendIdentity::NotLinux
            | RuntimeLinuxAudioBackendIdentity::Unavailable
            | RuntimeLinuxAudioBackendIdentity::Unsupported => {
                RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
            }
        }
    }

    fn restart_failure_count(&self) -> u64 {
        self.hardware.restart_failure_count
    }
}

impl From<HardwareLifecycleOwnership> for RuntimeHostLifecycleOwnership {
    fn from(value: HardwareLifecycleOwnership) -> Self {
        match value {
            HardwareLifecycleOwnership::HostDrivenCallback => Self::HostDrivenCallback,
            HardwareLifecycleOwnership::BackendManagedCallback => Self::BackendManagedCallback,
        }
    }
}

impl From<HardwareRestartPolicy> for RuntimeHostRestartPolicy {
    fn from(value: HardwareRestartPolicy) -> Self {
        match value {
            HardwareRestartPolicy::HostMustRestart => Self::HostMustRestart,
            HardwareRestartPolicy::BackendMayRestart => Self::BackendMayRestart,
        }
    }
}

impl From<HardwareClockSource> for RuntimeHostClockSource {
    fn from(value: HardwareClockSource) -> Self {
        match value {
            HardwareClockSource::Internal => Self::Internal,
            HardwareClockSource::ExternalWordClock => Self::ExternalWordClock,
            HardwareClockSource::DigitalInput => Self::DigitalInput,
            HardwareClockSource::Virtual => Self::Virtual,
        }
    }
}

impl From<HardwareClockTopology> for RuntimeHostClockDomain {
    fn from(value: HardwareClockTopology) -> Self {
        match value {
            HardwareClockTopology::SingleEndpoint => Self::SameClock,
            HardwareClockTopology::Aggregate => Self::Aggregate,
        }
    }
}

impl RuntimeObservationReport {
    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        RuntimePerformanceSnapshot::capture(
            &self.effective_config,
            &self.diagnostics_snapshot,
            &self.engine_block_snapshot,
            self.last_deferred_service_receipt.as_ref(),
        )
    }
}

impl RuntimeSupervisorReport {
    pub fn performance_snapshot(&self) -> RuntimePerformanceSnapshot {
        self.observation.performance_snapshot()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionLaneSummary {
    pub lane: GraphExecutionLane,
    pub groups: Vec<GraphNodePlanningGroup>,
    pub node_ids: Vec<String>,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub track_lane_ids: Vec<String>,
    pub bus_group_ids: Vec<String>,
    pub console_group_ids: Vec<String>,
    pub send_return_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeRoutedPluginChainSummary {
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
    pub chain_ids: Vec<String>,
    pub node_ids: Vec<String>,
    pub sandbox_ids: Vec<String>,
    pub chains: Vec<RuntimePluginExecutionChainSummary>,
}

impl Default for RuntimeRoutedPluginChainSummary {
    fn default() -> Self {
        Self {
            chain_count: 0,
            stage_count: 0,
            pending_render_stage_count: 0,
            settling_stage_count: 0,
            compensated_stage_count: 0,
            degraded_stage_count: 0,
            bypassed_stage_count: 0,
            missing_binding_stage_count: 0,
            total_planned_latency_samples: 0,
            total_realized_latency_samples: 0,
            total_tail_samples: 0,
            chain_ids: Vec::new(),
            node_ids: Vec::new(),
            sandbox_ids: Vec::new(),
            chains: Vec::new(),
        }
    }
}

impl RuntimeRoutedPluginChainSummary {
    fn include_chain(&mut self, chain: &RuntimePluginExecutionChainSummary) {
        if !self.chain_ids.contains(&chain.chain_id) {
            self.chain_count = self.chain_count.saturating_add(1);
            self.chain_ids.push(chain.chain_id.clone());
            self.chains.push(chain.clone());
        }
        self.stage_count = self.stage_count.saturating_add(chain.stage_count);
        self.pending_render_stage_count = self
            .pending_render_stage_count
            .saturating_add(chain.pending_render_stage_count);
        self.settling_stage_count = self
            .settling_stage_count
            .saturating_add(chain.settling_stage_count);
        self.compensated_stage_count = self
            .compensated_stage_count
            .saturating_add(chain.compensated_stage_count);
        self.degraded_stage_count = self
            .degraded_stage_count
            .saturating_add(chain.degraded_stage_count);
        self.bypassed_stage_count = self
            .bypassed_stage_count
            .saturating_add(chain.bypassed_stage_count);
        self.missing_binding_stage_count = self
            .missing_binding_stage_count
            .saturating_add(chain.missing_binding_stage_count);
        self.total_planned_latency_samples = self
            .total_planned_latency_samples
            .saturating_add(chain.total_planned_latency_samples);
        self.total_realized_latency_samples = self
            .total_realized_latency_samples
            .saturating_add(chain.total_realized_latency_samples);
        self.total_tail_samples = self
            .total_tail_samples
            .saturating_add(chain.total_tail_samples);
        for stage in &chain.stages {
            if !self.node_ids.contains(&stage.node_id) {
                self.node_ids.push(stage.node_id.clone());
            }
            if let Some(sandbox_id) = &stage.sandbox_id {
                if !self.sandbox_ids.contains(sandbox_id) {
                    self.sandbox_ids.push(sandbox_id.clone());
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerTrackLaneSummary {
    pub track_lane_id: String,
    pub node_ids: Vec<String>,
    pub bus_group_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerBusGroupSummary {
    pub bus_group_id: String,
    pub topology_roles: Vec<GraphNodeTopologyRole>,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerConsoleGroupSummary {
    pub console_group_id: String,
    pub node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMixerSendReturnSummary {
    pub send_return_id: String,
    pub send_node_ids: Vec<String>,
    pub return_node_ids: Vec<String>,
    pub input_bus_ids: Vec<String>,
    pub output_bus_ids: Vec<String>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeExecutionNodeSummary {
    pub node_id: String,
    pub lane: GraphExecutionLane,
    pub group: GraphNodePlanningGroup,
    pub execution_class: GraphNodeExecutionClass,
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
    pub plugin_recall_state: Option<RuntimePluginRecallState>,
    pub plugin_recall: Option<RuntimePluginRecallSnapshot>,
    pub plugin_compensation_state: Option<RuntimePluginCompensationState>,
    pub plugin_realized_latency_samples: Option<u32>,
    pub plugin_tail_samples: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeExecutionTopologySummary {
    pub node_count: usize,
    pub utility_node_count: usize,
    pub track_lane_node_count: usize,
    pub bus_node_count: usize,
    pub send_return_node_count: usize,
    pub console_node_count: usize,
    pub lane_count: usize,
    pub track_lane_group_count: usize,
    pub bus_group_count: usize,
    pub send_return_group_count: usize,
    pub console_group_count: usize,
    pub secondary_input_count: usize,
    pub required_secondary_input_count: usize,
    pub optional_secondary_input_count: usize,
    pub disabled_secondary_input_count: usize,
    pub terminal_fallback_secondary_input_count: usize,
    pub bus_connection_count: usize,
    pub auxiliary_path_count: usize,
    pub spatial_node_count: usize,
    pub active_spatial_node_count: usize,
    pub bypassed_spatial_node_count: usize,
    pub fallback_spatial_node_count: usize,
    pub surround_bed_spatial_node_count: usize,
    pub object_aware_spatial_node_count: usize,
    pub expanded_fallback_spatial_node_count: usize,
    pub immersive_spatial_node_count: usize,
    pub room_policy_aware_spatial_node_count: usize,
    pub fallback_room_policy_spatial_node_count: usize,
    pub deployment_spatial_node_count: usize,
    pub folded_down_spatial_node_count: usize,
    pub fallback_monitoring_scene_spatial_node_count: usize,
    pub renderer_capability_spatial_node_count: usize,
    pub negotiated_renderer_spatial_node_count: usize,
    pub immersive_export_spatial_node_count: usize,
    pub fallback_immersive_export_spatial_node_count: usize,
    pub lanes: Vec<RuntimeExecutionLaneSummary>,
    pub track_lanes: Vec<RuntimeMixerTrackLaneSummary>,
    pub bus_groups: Vec<RuntimeMixerBusGroupSummary>,
    pub console_groups: Vec<RuntimeMixerConsoleGroupSummary>,
    pub send_returns: Vec<RuntimeMixerSendReturnSummary>,
    pub secondary_inputs: Vec<RuntimeSecondaryInputRouteSummary>,
    pub bus_connections: Vec<RuntimeBusConnectionSummary>,
    pub auxiliary_paths: Vec<RuntimeAuxiliaryPathSummary>,
    pub nodes: Vec<RuntimeExecutionNodeSummary>,
    pub plugin_chain: RuntimeRoutedPluginChainSummary,
}

#[derive(Clone)]
struct RuntimePlannedGraphNodeTopologyEndpoint<'a> {
    node_id: &'a str,
    topology_role: GraphNodeTopologyRole,
    input_bus_id: &'a str,
    output_bus_id: &'a str,
    input_bus_intent: RuntimeBusIntent,
    output_bus_intent: RuntimeBusIntent,
    bus_group_id: Option<&'a str>,
    send_return_id: Option<&'a str>,
}

fn runtime_auxiliary_path_for_connection(
    source: &RuntimePlannedGraphNodeTopologyEndpoint<'_>,
    target: &RuntimePlannedGraphNodeTopologyEndpoint<'_>,
) -> Option<(
    RuntimeAuxiliaryPathKind,
    String,
    RuntimeBusRole,
    RuntimeBusIntent,
)> {
    if let Some(send_return_id) = source.send_return_id.or(target.send_return_id) {
        return Some((
            RuntimeAuxiliaryPathKind::SendReturn,
            format!("send_return:{send_return_id}"),
            RuntimeBusRole::AuxSend,
            RuntimeBusIntent::AuxSend,
        ));
    }
    if let Some(bus_group_id) = source.bus_group_id.or(target.bus_group_id) {
        return Some((
            RuntimeAuxiliaryPathKind::Submix,
            format!("bus_group:{bus_group_id}"),
            RuntimeBusRole::Submix,
            RuntimeBusIntent::MainProgram,
        ));
    }
    let source_role = runtime_bus_role_for_endpoint(source.topology_role, source.output_bus_intent);
    let target_role = runtime_bus_role_for_endpoint(target.topology_role, target.input_bus_intent);
    if source_role == RuntimeBusRole::AnalysisTap || target_role == RuntimeBusRole::AnalysisTap {
        return Some((
            RuntimeAuxiliaryPathKind::Analysis,
            format!("analysis:{}", source.output_bus_id),
            RuntimeBusRole::AnalysisTap,
            RuntimeBusIntent::AnalysisTap,
        ));
    }
    None
}

fn derive_runtime_bus_connections(
    planned_nodes: &[RuntimePlannedGraphNode],
) -> (
    Vec<RuntimeBusConnectionSummary>,
    Vec<RuntimeAuxiliaryPathSummary>,
) {
    let mut producers_by_bus =
        std::collections::BTreeMap::<&str, Vec<RuntimePlannedGraphNodeTopologyEndpoint<'_>>>::new();
    for node in planned_nodes {
        producers_by_bus
            .entry(node.output_bus_id.as_str())
            .or_default()
            .push(RuntimePlannedGraphNodeTopologyEndpoint {
                node_id: node.node_id.as_str(),
                topology_role: node.topology_role,
                input_bus_id: node.input_bus_id.as_str(),
                output_bus_id: node.output_bus_id.as_str(),
                input_bus_intent: node.input_bus_intent,
                output_bus_intent: node.output_bus_intent,
                bus_group_id: node.bus_group_id.as_deref(),
                send_return_id: node.send_return_id.as_deref(),
            });
    }

    let mut connections = Vec::new();
    let mut auxiliary_paths =
        std::collections::BTreeMap::<String, RuntimeAuxiliaryPathSummary>::new();

    for node in planned_nodes {
        let Some(producers) = producers_by_bus.get(node.input_bus_id.as_str()) else {
            continue;
        };
        let target = RuntimePlannedGraphNodeTopologyEndpoint {
            node_id: node.node_id.as_str(),
            topology_role: node.topology_role,
            input_bus_id: node.input_bus_id.as_str(),
            output_bus_id: node.output_bus_id.as_str(),
            input_bus_intent: node.input_bus_intent,
            output_bus_intent: node.output_bus_intent,
            bus_group_id: node.bus_group_id.as_deref(),
            send_return_id: node.send_return_id.as_deref(),
        };
        for source in producers {
            let auxiliary_path = runtime_auxiliary_path_for_connection(source, &target).map(
                |(path_kind, auxiliary_path_id, bus_role, material_bus_intent)| {
                    (path_kind, auxiliary_path_id, bus_role, material_bus_intent)
                },
            );
            let source_bus_role =
                runtime_bus_role_for_endpoint(source.topology_role, source.output_bus_intent);
            let target_bus_role =
                runtime_bus_role_for_endpoint(target.topology_role, target.input_bus_intent);
            let connection_id = format!(
                "{}:{}->{}:{}",
                source.node_id, source.output_bus_id, target.node_id, target.input_bus_id
            );
            let attachment_class = RuntimeBusConnectionAttachmentClass::Required;
            let fallback_outcome = RuntimeBusConnectionFallbackOutcome::NoFallback;
            let summary = format!(
                "connection={} source={}:{}/{:?} target={}:{}/{:?} path={:?} attachment={:?} fallback={:?}",
                connection_id,
                source.node_id,
                source.output_bus_id,
                source_bus_role,
                target.node_id,
                target.input_bus_id,
                target_bus_role,
                auxiliary_path.as_ref().map(|(kind, path_id, _, _)| format!("{kind:?}:{path_id}")),
                attachment_class,
                fallback_outcome,
            );
            connections.push(RuntimeBusConnectionSummary {
                connection_id: connection_id.clone(),
                source_node_id: source.node_id.into(),
                source_bus_id: source.output_bus_id.into(),
                source_bus_role,
                target_node_id: target.node_id.into(),
                target_bus_id: target.input_bus_id.into(),
                target_bus_role,
                auxiliary_path_kind: auxiliary_path.as_ref().map(|(kind, _, _, _)| *kind),
                auxiliary_path_id: auxiliary_path
                    .as_ref()
                    .map(|(_, path_id, _, _)| path_id.clone()),
                attachment_class,
                fallback_outcome,
                summary,
            });

            if let Some((path_kind, auxiliary_path_id, bus_role, material_bus_intent)) =
                auxiliary_path
            {
                let path = auxiliary_paths
                    .entry(auxiliary_path_id.clone())
                    .or_insert_with(|| RuntimeAuxiliaryPathSummary {
                        auxiliary_path_id: auxiliary_path_id.clone(),
                        path_kind,
                        bus_role,
                        material_bus_intent,
                        source_node_ids: Vec::new(),
                        target_node_ids: Vec::new(),
                        bus_ids: Vec::new(),
                        connection_ids: Vec::new(),
                        attachment_class,
                        fallback_outcome,
                        summary: String::new(),
                    });
                if !path.source_node_ids.contains(&source.node_id.to_string()) {
                    path.source_node_ids.push(source.node_id.to_string());
                }
                if !path.target_node_ids.contains(&target.node_id.to_string()) {
                    path.target_node_ids.push(target.node_id.to_string());
                }
                if !path.bus_ids.contains(&source.output_bus_id.to_string()) {
                    path.bus_ids.push(source.output_bus_id.to_string());
                }
                if !path.bus_ids.contains(&target.input_bus_id.to_string()) {
                    path.bus_ids.push(target.input_bus_id.to_string());
                }
                if !path.connection_ids.contains(&connection_id) {
                    path.connection_ids.push(connection_id.clone());
                }
            }
        }
    }

    let mut auxiliary_paths = auxiliary_paths.into_values().collect::<Vec<_>>();
    for path in &mut auxiliary_paths {
        path.summary = format!(
            "path={} kind={:?} role={:?} material={:?} sources={:?} targets={:?} buses={:?} connections={} attachment={:?} fallback={:?}",
            path.auxiliary_path_id,
            path.path_kind,
            path.bus_role,
            path.material_bus_intent,
            path.source_node_ids,
            path.target_node_ids,
            path.bus_ids,
            path.connection_ids.len(),
            path.attachment_class,
            path.fallback_outcome,
        );
    }

    (connections, auxiliary_paths)
}

impl RuntimeExecutionTopologySummary {
    pub fn from_snapshot(snapshot: &RuntimeEngineBlockSnapshot) -> Self {
        let mut track_lane_ids = std::collections::BTreeSet::new();
        let mut bus_group_ids = std::collections::BTreeSet::new();
        let mut send_return_ids = std::collections::BTreeSet::new();
        let mut console_group_ids = std::collections::BTreeSet::new();
        let mut lanes = Vec::new();

        for lane in &snapshot.lane_order {
            let mut groups = Vec::new();
            let mut node_ids = Vec::new();
            let mut topology_roles = Vec::new();
            let mut lane_ids = Vec::new();
            let mut bus_groups = Vec::new();
            let mut console_groups = Vec::new();
            let mut send_returns = Vec::new();

            for node in snapshot
                .planned_nodes
                .iter()
                .filter(|node| runtime_lane_for_group(node.group) == *lane)
            {
                if !groups.contains(&node.group) {
                    groups.push(node.group);
                }
                node_ids.push(node.node_id.clone());
                if !topology_roles.contains(&node.topology_role) {
                    topology_roles.push(node.topology_role);
                }
                if let Some(track_lane_id) = &node.track_lane_id {
                    if !lane_ids.contains(track_lane_id) {
                        lane_ids.push(track_lane_id.clone());
                    }
                    track_lane_ids.insert(track_lane_id.clone());
                }
                if let Some(bus_group_id) = &node.bus_group_id {
                    if !bus_groups.contains(bus_group_id) {
                        bus_groups.push(bus_group_id.clone());
                    }
                    bus_group_ids.insert(bus_group_id.clone());
                }
                if let Some(console_group_id) = &node.console_group_id {
                    if !console_groups.contains(console_group_id) {
                        console_groups.push(console_group_id.clone());
                    }
                    console_group_ids.insert(console_group_id.clone());
                }
                if let Some(send_return_id) = &node.send_return_id {
                    if !send_returns.contains(send_return_id) {
                        send_returns.push(send_return_id.clone());
                    }
                    send_return_ids.insert(send_return_id.clone());
                }
            }

            lanes.push(RuntimeExecutionLaneSummary {
                lane: *lane,
                groups,
                node_ids,
                topology_roles,
                track_lane_ids: lane_ids,
                bus_group_ids: bus_groups,
                console_group_ids: console_groups,
                send_return_ids: send_returns,
            });
        }

        let mut track_lanes_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerTrackLaneSummary>::new();
        let mut bus_groups_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerBusGroupSummary>::new();
        let mut console_groups_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerConsoleGroupSummary>::new();
        let mut send_returns_by_id =
            std::collections::BTreeMap::<String, RuntimeMixerSendReturnSummary>::new();

        let mut nodes = Vec::with_capacity(snapshot.planned_nodes.len());
        let mut utility_node_count = 0usize;
        let mut track_lane_node_count = 0usize;
        let mut bus_node_count = 0usize;
        let mut send_return_node_count = 0usize;
        let mut console_node_count = 0usize;
        let mut secondary_inputs = Vec::new();
        let mut required_secondary_input_count = 0usize;
        let mut optional_secondary_input_count = 0usize;
        let mut disabled_secondary_input_count = 0usize;
        let mut terminal_fallback_secondary_input_count = 0usize;
        let mut spatial_node_count = 0usize;
        let mut active_spatial_node_count = 0usize;
        let mut bypassed_spatial_node_count = 0usize;
        let mut fallback_spatial_node_count = 0usize;
        let mut surround_bed_spatial_node_count = 0usize;
        let mut object_aware_spatial_node_count = 0usize;
        let mut expanded_fallback_spatial_node_count = 0usize;
        let mut immersive_spatial_node_count = 0usize;
        let mut room_policy_aware_spatial_node_count = 0usize;
        let mut fallback_room_policy_spatial_node_count = 0usize;
        let mut deployment_spatial_node_count = 0usize;
        let mut folded_down_spatial_node_count = 0usize;
        let mut fallback_monitoring_scene_spatial_node_count = 0usize;
        let mut renderer_capability_spatial_node_count = 0usize;
        let mut negotiated_renderer_spatial_node_count = 0usize;
        let mut immersive_export_spatial_node_count = 0usize;
        let mut fallback_immersive_export_spatial_node_count = 0usize;

        for node in &snapshot.planned_nodes {
            match node.topology_role {
                GraphNodeTopologyRole::Utility => utility_node_count += 1,
                GraphNodeTopologyRole::TrackLane => track_lane_node_count += 1,
                GraphNodeTopologyRole::Bus => bus_node_count += 1,
                GraphNodeTopologyRole::Send | GraphNodeTopologyRole::Return => {
                    send_return_node_count += 1;
                }
                GraphNodeTopologyRole::ConsoleNode => console_node_count += 1,
            }
            if let Some(track_lane_id) = &node.track_lane_id {
                let summary = track_lanes_by_id
                    .entry(track_lane_id.clone())
                    .or_insert_with(|| RuntimeMixerTrackLaneSummary {
                        track_lane_id: track_lane_id.clone(),
                        node_ids: Vec::new(),
                        bus_group_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                summary.node_ids.push(node.node_id.clone());
                if let Some(bus_group_id) = &node.bus_group_id {
                    if !summary.bus_group_ids.contains(bus_group_id) {
                        summary.bus_group_ids.push(bus_group_id.clone());
                    }
                }
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            if let Some(bus_group_id) = &node.bus_group_id {
                let summary = bus_groups_by_id
                    .entry(bus_group_id.clone())
                    .or_insert_with(|| RuntimeMixerBusGroupSummary {
                        bus_group_id: bus_group_id.clone(),
                        topology_roles: Vec::new(),
                        node_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                if !summary.topology_roles.contains(&node.topology_role) {
                    summary.topology_roles.push(node.topology_role);
                }
                summary.node_ids.push(node.node_id.clone());
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            if let Some(console_group_id) = &node.console_group_id {
                let summary = console_groups_by_id
                    .entry(console_group_id.clone())
                    .or_insert_with(|| RuntimeMixerConsoleGroupSummary {
                        console_group_id: console_group_id.clone(),
                        node_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                summary.node_ids.push(node.node_id.clone());
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            if let Some(send_return_id) = &node.send_return_id {
                let summary = send_returns_by_id
                    .entry(send_return_id.clone())
                    .or_insert_with(|| RuntimeMixerSendReturnSummary {
                        send_return_id: send_return_id.clone(),
                        send_node_ids: Vec::new(),
                        return_node_ids: Vec::new(),
                        input_bus_ids: Vec::new(),
                        output_bus_ids: Vec::new(),
                        plugin_chain: RuntimeRoutedPluginChainSummary::default(),
                    });
                match node.topology_role {
                    GraphNodeTopologyRole::Send => summary.send_node_ids.push(node.node_id.clone()),
                    GraphNodeTopologyRole::Return => {
                        summary.return_node_ids.push(node.node_id.clone());
                    }
                    _ => {}
                }
                if !summary.input_bus_ids.contains(&node.input_bus_id) {
                    summary.input_bus_ids.push(node.input_bus_id.clone());
                }
                if !summary.output_bus_ids.contains(&node.output_bus_id) {
                    summary.output_bus_ids.push(node.output_bus_id.clone());
                }
            }
            if let Some(secondary_input) = &node.secondary_input {
                secondary_inputs.push(secondary_input.clone());
                match secondary_input.attachment_policy {
                    RuntimeSecondaryInputAttachmentPolicy::Required => {
                        required_secondary_input_count += 1;
                    }
                    RuntimeSecondaryInputAttachmentPolicy::Optional => {
                        optional_secondary_input_count += 1;
                    }
                    RuntimeSecondaryInputAttachmentPolicy::Disabled => {
                        disabled_secondary_input_count += 1;
                    }
                }
                if secondary_input.fallback_outcome
                    == RuntimeSecondaryInputFallbackOutcome::TerminalRoutingFailure
                {
                    terminal_fallback_secondary_input_count += 1;
                }
            }
            if let Some(spatial_execution) = &node.spatial_execution {
                spatial_node_count += 1;
                if spatial_execution.execution_mode == RuntimeSpatialExecutionMode::Bypassed {
                    bypassed_spatial_node_count += 1;
                } else {
                    active_spatial_node_count += 1;
                }
                if spatial_execution.fallback_outcome.is_some() {
                    fallback_spatial_node_count += 1;
                }
                if spatial_execution.bed_class == RuntimeSpatialBedClass::CanonicalSurroundBed {
                    surround_bed_spatial_node_count += 1;
                }
                if spatial_execution.object_count > 0 || spatial_execution.object_role.is_some() {
                    object_aware_spatial_node_count += 1;
                }
                if spatial_execution.expanded_fallback_outcome.is_some() {
                    expanded_fallback_spatial_node_count += 1;
                }
                if let Some(immersive_room_policy) = &spatial_execution.immersive_room_policy {
                    immersive_spatial_node_count += 1;
                    if immersive_room_policy.object_rendering_posture
                        == RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
                    {
                        room_policy_aware_spatial_node_count += 1;
                    }
                    if immersive_room_policy.room_policy_class
                        == RuntimeRoomPolicyClass::FallbackRoom
                    {
                        fallback_room_policy_spatial_node_count += 1;
                    }
                }
                if let Some(deployment_monitoring) = &spatial_execution.deployment_monitoring {
                    deployment_spatial_node_count += 1;
                    if matches!(
                        deployment_monitoring.fold_down_policy,
                        RuntimeFoldDownPolicy::FoldDownToReferenceBed
                            | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring
                            | RuntimeFoldDownPolicy::FoldDownToPortablePreview
                    ) {
                        folded_down_spatial_node_count += 1;
                    }
                    if deployment_monitoring.monitoring_scene_class
                        == RuntimeMonitoringSceneClass::FallbackScene
                    {
                        fallback_monitoring_scene_spatial_node_count += 1;
                    }
                }
                if let Some(renderer_export) = &spatial_execution.renderer_export {
                    renderer_capability_spatial_node_count += 1;
                    if renderer_export.renderer_capability_posture
                        == RuntimeRendererCapabilityNegotiationPosture::NegotiatedCompatible
                    {
                        negotiated_renderer_spatial_node_count += 1;
                    }
                    if renderer_export.immersive_export_class
                        != RuntimeImmersiveExportClass::NoImmersiveExport
                    {
                        immersive_export_spatial_node_count += 1;
                    }
                    if renderer_export.immersive_export_class
                        == RuntimeImmersiveExportClass::FallbackExport
                    {
                        fallback_immersive_export_spatial_node_count += 1;
                    }
                }
            }
            nodes.push(RuntimeExecutionNodeSummary {
                node_id: node.node_id.clone(),
                lane: runtime_lane_for_group(node.group),
                group: node.group,
                execution_class: node.execution_class,
                topology_role: node.topology_role,
                track_lane_id: node.track_lane_id.clone(),
                bus_group_id: node.bus_group_id.clone(),
                console_group_id: node.console_group_id.clone(),
                send_return_id: node.send_return_id.clone(),
                input_bus_id: node.input_bus_id.clone(),
                output_bus_id: node.output_bus_id.clone(),
                input_channels: node.input_channels,
                output_channels: node.output_channels,
                input_layout: node.input_layout.clone(),
                output_layout: node.output_layout.clone(),
                input_bus_intent: node.input_bus_intent,
                output_bus_intent: node.output_bus_intent,
                secondary_input: node.secondary_input.clone(),
                spatial_execution: node.spatial_execution.clone(),
                plugin_sandbox_id: node.plugin_sandbox_id.clone(),
                plugin_recall_state: None,
                plugin_recall: None,
                plugin_compensation_state: None,
                plugin_realized_latency_samples: None,
                plugin_tail_samples: None,
            });
        }
        let (bus_connections, auxiliary_paths) =
            derive_runtime_bus_connections(&snapshot.planned_nodes);

        Self {
            node_count: snapshot.planned_nodes.len(),
            utility_node_count,
            track_lane_node_count,
            bus_node_count,
            send_return_node_count,
            console_node_count,
            lane_count: lanes.len(),
            track_lane_group_count: track_lane_ids.len(),
            bus_group_count: bus_group_ids.len(),
            send_return_group_count: send_return_ids.len(),
            console_group_count: console_group_ids.len(),
            secondary_input_count: secondary_inputs.len(),
            required_secondary_input_count,
            optional_secondary_input_count,
            disabled_secondary_input_count,
            terminal_fallback_secondary_input_count,
            bus_connection_count: bus_connections.len(),
            auxiliary_path_count: auxiliary_paths.len(),
            spatial_node_count,
            active_spatial_node_count,
            bypassed_spatial_node_count,
            fallback_spatial_node_count,
            surround_bed_spatial_node_count,
            object_aware_spatial_node_count,
            expanded_fallback_spatial_node_count,
            immersive_spatial_node_count,
            room_policy_aware_spatial_node_count,
            fallback_room_policy_spatial_node_count,
            deployment_spatial_node_count,
            folded_down_spatial_node_count,
            fallback_monitoring_scene_spatial_node_count,
            renderer_capability_spatial_node_count,
            negotiated_renderer_spatial_node_count,
            immersive_export_spatial_node_count,
            fallback_immersive_export_spatial_node_count,
            lanes,
            track_lanes: track_lanes_by_id.into_values().collect(),
            bus_groups: bus_groups_by_id.into_values().collect(),
            console_groups: console_groups_by_id.into_values().collect(),
            send_returns: send_returns_by_id.into_values().collect(),
            secondary_inputs,
            bus_connections,
            auxiliary_paths,
            nodes,
            plugin_chain: RuntimeRoutedPluginChainSummary::default(),
        }
    }

    pub fn with_plugin_chain_snapshot(mut self, snapshot: &RuntimePluginChainSnapshot) -> Self {
        let mut stage_by_node =
            std::collections::BTreeMap::<&str, &RuntimePluginChainStageSnapshot>::new();
        for chain in &snapshot.chains {
            self.plugin_chain.include_chain(chain);
            if let Some(track_lane_id) = chain.track_lane_id.as_deref() {
                if let Some(summary) = self
                    .track_lanes
                    .iter_mut()
                    .find(|summary| summary.track_lane_id == track_lane_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(bus_group_id) = chain.bus_group_id.as_deref() {
                if let Some(summary) = self
                    .bus_groups
                    .iter_mut()
                    .find(|summary| summary.bus_group_id == bus_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(console_group_id) = chain.console_group_id.as_deref() {
                if let Some(summary) = self
                    .console_groups
                    .iter_mut()
                    .find(|summary| summary.console_group_id == console_group_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            if let Some(send_return_id) = chain.send_return_id.as_deref() {
                if let Some(summary) = self
                    .send_returns
                    .iter_mut()
                    .find(|summary| summary.send_return_id == send_return_id)
                {
                    summary.plugin_chain.include_chain(chain);
                }
            }
            for stage in &chain.stages {
                stage_by_node.insert(stage.node_id.as_str(), stage);
            }
        }

        for node in &mut self.nodes {
            if let Some(stage) = stage_by_node.get(node.node_id.as_str()) {
                node.spatial_execution = stage.spatial_execution.clone();
                node.plugin_recall_state = Some(stage.recall_state);
                node.plugin_recall = Some(stage.recall.clone());
                node.plugin_compensation_state = Some(stage.compensation_state);
                node.plugin_realized_latency_samples = stage.realized_latency_samples;
                node.plugin_tail_samples = stage.tail_samples;
            }
        }

        self
    }
}

impl RuntimeOfflineRenderContractPreview {
    pub fn chain_contract_from_runtime_state(
        topology: &RuntimeExecutionTopologySummary,
        recall_handoff: &RuntimePluginRecallHandoffSnapshot,
    ) -> Result<RuntimeOfflineRenderChainDependencyPreview, RuntimeError> {
        let plugin_chain = &topology.plugin_chain;
        if plugin_chain.stage_count != recall_handoff.stage_count {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                format!(
                    "offline render contract requires aligned plugin chain and recall handoff stages (chain={} recall={})",
                    plugin_chain.stage_count, recall_handoff.stage_count
                ),
            ));
        }

        let complex_io_stages = plugin_chain
            .chains
            .iter()
            .flat_map(|chain| {
                chain.stages.iter().filter_map(|stage| {
                    if stage.complex_io_summary.has_complex_topology {
                        Some(RuntimeOfflineRenderComplexIoStageSummary {
                            chain_id: chain.chain_id.clone(),
                            node_id: stage.node_id.clone(),
                            stage_index: stage.stage_index,
                            plugin_type_id: stage.recall.payload.plugin_type_id.clone(),
                            topology: stage.complex_io_summary.clone(),
                            summary: format!(
                                "chain={} node={} stage={} plugin_type={:?} complex_io={}",
                                chain.chain_id,
                                stage.node_id,
                                stage.stage_index,
                                stage.recall.payload.plugin_type_id,
                                stage.complex_io_summary.summary
                            ),
                        })
                    } else {
                        None
                    }
                })
            })
            .collect::<Vec<_>>();
        let multi_output_instrument_stage_count = complex_io_stages
            .iter()
            .filter(|stage| stage.topology.multi_output_instrument)
            .count();
        let complex_io_stage_count = complex_io_stages.len();
        let bus_capable_fx_stage_count = complex_io_stages
            .iter()
            .filter(|stage| stage.topology.bus_capable_fx_class.is_some())
            .count();
        let sidechain_capable_fx_stage_count = complex_io_stages
            .iter()
            .filter(|stage| {
                stage.topology.bus_capable_fx_class
                    == Some(RuntimePluginBusCapableFxClass::SidechainCapableFx)
                    || stage.topology.bus_capable_fx_class
                        == Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
            })
            .count();
        let spatial_stages = plugin_chain
            .chains
            .iter()
            .flat_map(|chain| {
                chain.stages.iter().filter_map(|stage| {
                    stage.spatial_execution.as_ref().map(|spatial| {
                        RuntimeOfflineRenderSpatialStageSummary {
                            chain_id: chain.chain_id.clone(),
                            node_id: stage.node_id.clone(),
                            stage_index: stage.stage_index,
                            plugin_type_id: stage.recall.payload.plugin_type_id.clone(),
                            spatial: spatial.clone(),
                            summary: format!(
                                "chain={} node={} stage={} plugin_type={:?} spatial={}",
                                chain.chain_id,
                                stage.node_id,
                                stage.stage_index,
                                stage.recall.payload.plugin_type_id,
                                spatial.summary
                            ),
                        }
                    })
                })
            })
            .collect::<Vec<_>>();
        let spatial_stage_count = spatial_stages.len();
        let active_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.execution_mode != RuntimeSpatialExecutionMode::Bypassed)
            .count();
        let bypassed_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.execution_mode == RuntimeSpatialExecutionMode::Bypassed)
            .count();
        let fallback_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.fallback_outcome.is_some())
            .count();
        let surround_bed_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.bed_class == RuntimeSpatialBedClass::CanonicalSurroundBed)
            .count();
        let object_aware_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.object_count > 0 || stage.spatial.object_role.is_some())
            .count();
        let expanded_fallback_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.expanded_fallback_outcome.is_some())
            .count();
        let immersive_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.immersive_room_policy.is_some())
            .count();
        let room_policy_aware_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .immersive_room_policy
                    .as_ref()
                    .is_some_and(|immersive| {
                        immersive.object_rendering_posture
                            == RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
                    })
            })
            .count();
        let fallback_room_policy_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .immersive_room_policy
                    .as_ref()
                    .is_some_and(|immersive| {
                        immersive.room_policy_class == RuntimeRoomPolicyClass::FallbackRoom
                    })
            })
            .count();
        let renderer_capability_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.renderer_export.is_some())
            .count();
        let negotiated_renderer_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .renderer_export
                    .as_ref()
                    .is_some_and(|renderer| {
                        renderer.renderer_capability_posture
                            == RuntimeRendererCapabilityNegotiationPosture::NegotiatedCompatible
                    })
            })
            .count();
        let immersive_export_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .renderer_export
                    .as_ref()
                    .is_some_and(|renderer| {
                        renderer.immersive_export_class
                            != RuntimeImmersiveExportClass::NoImmersiveExport
                    })
            })
            .count();
        let fallback_immersive_export_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .renderer_export
                    .as_ref()
                    .is_some_and(|renderer| {
                        renderer.immersive_export_class
                            == RuntimeImmersiveExportClass::FallbackExport
                    })
            })
            .count();
        let deployment_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| stage.spatial.deployment_monitoring.is_some())
            .count();
        let folded_down_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .deployment_monitoring
                    .as_ref()
                    .is_some_and(|monitoring| {
                        matches!(
                            monitoring.fold_down_policy,
                            RuntimeFoldDownPolicy::FoldDownToReferenceBed
                                | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring
                                | RuntimeFoldDownPolicy::FoldDownToPortablePreview
                        )
                    })
            })
            .count();
        let fallback_monitoring_scene_spatial_stage_count = spatial_stages
            .iter()
            .filter(|stage| {
                stage
                    .spatial
                    .deployment_monitoring
                    .as_ref()
                    .is_some_and(|monitoring| {
                        monitoring.monitoring_scene_class
                            == RuntimeMonitoringSceneClass::FallbackScene
                    })
            })
            .count();

        Ok(RuntimeOfflineRenderChainDependencyPreview {
            chain_count: plugin_chain.chain_count,
            stage_count: plugin_chain.stage_count,
            pending_render_stage_count: plugin_chain.pending_render_stage_count,
            settling_stage_count: plugin_chain.settling_stage_count,
            compensated_stage_count: plugin_chain.compensated_stage_count,
            degraded_stage_count: plugin_chain.degraded_stage_count,
            bypassed_stage_count: plugin_chain.bypassed_stage_count,
            missing_binding_stage_count: plugin_chain.missing_binding_stage_count,
            total_planned_latency_samples: plugin_chain.total_planned_latency_samples,
            total_realized_latency_samples: plugin_chain.total_realized_latency_samples,
            total_tail_samples: plugin_chain.total_tail_samples,
            recall_stage_count: recall_handoff.stage_count,
            unbound_recall_stage_count: recall_handoff.unbound_stage_count,
            cold_recall_stage_count: recall_handoff.cold_stage_count,
            warm_recall_stage_count: recall_handoff.warm_stage_count,
            recovered_recall_stage_count: recall_handoff.recovered_stage_count,
            unavailable_recall_stage_count: recall_handoff.unavailable_stage_count,
            secondary_input_count: topology.secondary_input_count,
            required_secondary_input_count: topology.required_secondary_input_count,
            optional_secondary_input_count: topology.optional_secondary_input_count,
            disabled_secondary_input_count: topology.disabled_secondary_input_count,
            terminal_fallback_secondary_input_count: topology
                .terminal_fallback_secondary_input_count,
            bus_connection_count: topology.bus_connection_count,
            auxiliary_path_count: topology.auxiliary_path_count,
            complex_io_stage_count,
            multi_output_instrument_stage_count,
            bus_capable_fx_stage_count,
            sidechain_capable_fx_stage_count,
            spatial_stage_count,
            active_spatial_stage_count,
            bypassed_spatial_stage_count,
            fallback_spatial_stage_count,
            surround_bed_spatial_stage_count,
            object_aware_spatial_stage_count,
            expanded_fallback_spatial_stage_count,
            immersive_spatial_stage_count,
            room_policy_aware_spatial_stage_count,
            fallback_room_policy_spatial_stage_count,
            deployment_spatial_stage_count,
            folded_down_spatial_stage_count,
            fallback_monitoring_scene_spatial_stage_count,
            renderer_capability_spatial_stage_count,
            negotiated_renderer_spatial_stage_count,
            immersive_export_spatial_stage_count,
            fallback_immersive_export_spatial_stage_count,
            secondary_inputs: topology
                .secondary_inputs
                .iter()
                .cloned()
                .map(|mut route| {
                    route.target_kind = RuntimeSecondaryInputTargetKind::RenderInput;
                    route.target_id = "offline-render".into();
                    route.summary = format!(
                        "source={:?}:{}/{} target={:?}:{}/{} policy={:?} fallback={:?}",
                        route.source_kind,
                        route.source_id,
                        route.source_bus_id.as_deref().unwrap_or("none"),
                        route.target_kind,
                        route.target_id,
                        route.target_bus_id,
                        route.attachment_policy,
                        route.fallback_outcome,
                    );
                    route
                })
                .collect(),
            bus_connections: topology.bus_connections.clone(),
            auxiliary_paths: topology.auxiliary_paths.clone(),
            complex_io_stages,
            spatial_stages,
            summary: format!(
                "chains={} stages={} pending={} settling={} compensated={} degraded={} bypassed={} missing={} latency={}/{} tail={} recall={}/unbound={} cold={} warm={} recovered={} unavailable={} secondary_inputs={}/required={}/optional={}/disabled={}/terminal={} bus_connections={} auxiliary_paths={} complex_io_stages={} multi_output_instruments={} bus_capable_fx={} sidechain_capable_fx={} spatial_stages={}/active={}/bypassed={}/fallback={} surround_beds={} object_aware={} expanded_fallbacks={} immersive={} room_policy_aware={} fallback_room_policy={} deployment={} folded_down={} fallback_monitoring_scene={} renderer_capability={} negotiated_renderer={} immersive_export={} fallback_immersive_export={}",
                plugin_chain.chain_count,
                plugin_chain.stage_count,
                plugin_chain.pending_render_stage_count,
                plugin_chain.settling_stage_count,
                plugin_chain.compensated_stage_count,
                plugin_chain.degraded_stage_count,
                plugin_chain.bypassed_stage_count,
                plugin_chain.missing_binding_stage_count,
                plugin_chain.total_planned_latency_samples,
                plugin_chain.total_realized_latency_samples,
                plugin_chain.total_tail_samples,
                recall_handoff.stage_count,
                recall_handoff.unbound_stage_count,
                recall_handoff.cold_stage_count,
                recall_handoff.warm_stage_count,
                recall_handoff.recovered_stage_count,
                recall_handoff.unavailable_stage_count,
                topology.secondary_input_count,
                topology.required_secondary_input_count,
                topology.optional_secondary_input_count,
                topology.disabled_secondary_input_count,
                topology.terminal_fallback_secondary_input_count,
                topology.bus_connection_count,
                topology.auxiliary_path_count,
                complex_io_stage_count,
                multi_output_instrument_stage_count,
                bus_capable_fx_stage_count,
                sidechain_capable_fx_stage_count,
                spatial_stage_count,
                active_spatial_stage_count,
                bypassed_spatial_stage_count,
                fallback_spatial_stage_count,
                surround_bed_spatial_stage_count,
                object_aware_spatial_stage_count,
                expanded_fallback_spatial_stage_count,
                immersive_spatial_stage_count,
                room_policy_aware_spatial_stage_count,
                fallback_room_policy_spatial_stage_count,
                deployment_spatial_stage_count,
                folded_down_spatial_stage_count,
                fallback_monitoring_scene_spatial_stage_count,
                renderer_capability_spatial_stage_count,
                negotiated_renderer_spatial_stage_count,
                immersive_export_spatial_stage_count,
                fallback_immersive_export_spatial_stage_count,
            ),
        })
    }

    pub fn from_runtime_state(
        request: &RuntimeOfflineRenderRequest,
        topology: &RuntimeExecutionTopologySummary,
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
        media_pipeline: &RuntimeMediaPipelineSnapshot,
        tempo_map: &RuntimeTempoMapSnapshot,
        marker_analysis: &RuntimeMarkerAnalysisSnapshot,
        recall_handoff: &RuntimePluginRecallHandoffSnapshot,
    ) -> Result<Self, RuntimeError> {
        if request.request_id.trim().is_empty() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a non-empty request id",
            ));
        }
        if request.duration_samples == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a non-zero duration",
            ));
        }
        if request.export_sample_rate_hz == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "offline render requests require a positive export sample rate",
            ));
        }

        let mut seen_stem_ids = std::collections::BTreeSet::new();
        let mut stem_targets = Vec::with_capacity(request.stem_targets.len());
        for stem in &request.stem_targets {
            if stem.stem_id.trim().is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "offline render stem targets require a non-empty stem id",
                ));
            }
            if !seen_stem_ids.insert(stem.stem_id.clone()) {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!("offline render stem id `{}` is duplicated", stem.stem_id),
                ));
            }
            stem_targets.push(resolve_offline_render_stem_target(stem, topology)?);
        }

        let mut freeze_artifacts = Vec::with_capacity(request.freeze_artifacts.len());
        for artifact in &request.freeze_artifacts {
            if artifact.artifact_id.trim().is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "offline freeze artifacts require a non-empty artifact id",
                ));
            }
            if !request.include_main_mix
                && !stem_targets
                    .iter()
                    .any(|stem| stem.stem_id == artifact.source_stem_id)
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline freeze artifact `{}` references unknown stem `{}`",
                        artifact.artifact_id, artifact.source_stem_id
                    ),
                ));
            }
            let resolved_selection = recall_handoff
                .resolve_selection(&artifact.recall_selection)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline freeze artifact `{}` references an unknown recall handoff stage",
                            artifact.artifact_id
                        ),
                    )
                })?;
            freeze_artifacts.push(RuntimeOfflineFreezeArtifactPreview {
                artifact_id: artifact.artifact_id.clone(),
                source_stem_id: artifact.source_stem_id.clone(),
                recall_stage_count: resolved_selection.len(),
                recall_stage_ids: resolved_selection
                    .iter()
                    .map(|stage| stage.stage_id.clone())
                    .collect(),
                recall_states: resolved_selection
                    .iter()
                    .map(|stage| stage.recall_state)
                    .collect(),
                summary: format!(
                    "artifact={} source_stem={} recall_stages={} recall_states={:?}",
                    artifact.artifact_id,
                    artifact.source_stem_id,
                    resolved_selection.len(),
                    resolved_selection
                        .iter()
                        .map(|stage| stage.recall_state)
                        .collect::<Vec<_>>(),
                ),
            });
        }

        let timeline_end_samples = request
            .timeline_start_samples
            .saturating_add(request.duration_samples as i64);
        let chain_contract = Self::chain_contract_from_runtime_state(topology, recall_handoff)?;
        let stretch_engine_snapshot =
            RuntimeStretchEngineSnapshot::from_clip_processing_pipeline(clip_processing);
        let transform_artifact_snapshot =
            RuntimeTransformArtifactSnapshot::from_runtime_transform_state(
                clip_processing,
                &stretch_engine_snapshot,
                marker_analysis,
                media_pipeline,
            );
        let preview_transform_snapshot =
            RuntimePreviewTransformServiceSnapshot::from_runtime_preview_state(
                clip_processing,
                &RuntimeMediaServiceSnapshot {
                    indexed_asset_count: media_pipeline.asset_count,
                    analysis_ready_asset_count: 0,
                    waveform_ready_asset_count: 0,
                    waveform_pending_asset_count: 0,
                    previewable_asset_count: media_pipeline.ready_asset_count,
                    invalidated_asset_count: media_pipeline.invalid_asset_count,
                    invalidation_active: media_pipeline.invalid_asset_count > 0,
                    indexing_state: if media_pipeline.asset_count == 0 {
                        RuntimeMediaIndexingState::Empty
                    } else if media_pipeline.invalid_asset_count > 0 {
                        RuntimeMediaIndexingState::Invalidated
                    } else {
                        RuntimeMediaIndexingState::Ready
                    },
                    preview_state: if media_pipeline.ready_asset_count > 0 {
                        RuntimeMediaPreviewState::Ready
                    } else if media_pipeline.invalid_asset_count > 0 {
                        RuntimeMediaPreviewState::Invalidated
                    } else {
                        RuntimeMediaPreviewState::Unavailable
                    },
                    previewing_asset_id: None,
                    last_invalidated_asset_id: None,
                    last_invalidation_error: None,
                    last_preview_error: None,
                    summary: "offline preview derived from runtime media pipeline".into(),
                },
                &stretch_engine_snapshot,
                marker_analysis,
                &transform_artifact_snapshot,
            );
        let mut preview = Self {
            request_id: request.request_id.clone(),
            timeline_start_samples: request.timeline_start_samples,
            timeline_end_samples,
            duration_samples: request.duration_samples,
            export_sample_rate_hz: request.export_sample_rate_hz,
            include_main_mix: request.include_main_mix,
            clip_count: clip_processing.clip_count,
            ready_clip_count: clip_processing.ready_clip_count,
            stretch_engine_snapshot,
            preview_transform_snapshot,
            transform_artifact_snapshot,
            stem_count: stem_targets.len(),
            freeze_artifact_count: freeze_artifacts.len(),
            resolved_tempo_bpm: tempo_map.resolved_tempo_bpm,
            resolved_tempo_source: tempo_map.tempo_source,
            chain_contract,
            stem_targets,
            freeze_artifacts,
            summary: String::new(),
        };
        preview.summary = format!(
            "request={} timeline={}..{} duration={} export_sample_rate={} clips={}/{} stretch={}/fallback={} preview_transform={}/artifact_backed={}/fallback={} transform_artifacts={}/reusable={} stems={} freeze_artifacts={} tempo={:.3}/{:?} chain_contract={}",
            preview.request_id,
            preview.timeline_start_samples,
            preview.timeline_end_samples,
            preview.duration_samples,
            preview.export_sample_rate_hz,
            preview.ready_clip_count,
            preview.clip_count,
            preview.stretch_engine_snapshot.ready_clip_count,
            preview.stretch_engine_snapshot.fallback_clip_count,
            preview.preview_transform_snapshot.ready_clip_count,
            preview.preview_transform_snapshot.artifact_backed_clip_count,
            preview.preview_transform_snapshot.fallback_clip_count,
            preview.transform_artifact_snapshot.ready_clip_count,
            preview.transform_artifact_snapshot.reusable_clip_count,
            preview.stem_count,
            preview.freeze_artifact_count,
            preview.resolved_tempo_bpm,
            preview.resolved_tempo_source,
            preview.chain_contract.summary,
        );
        Ok(preview)
    }
}

fn resolve_offline_render_stem_target(
    stem: &RuntimeOfflineRenderStemTarget,
    topology: &RuntimeExecutionTopologySummary,
) -> Result<RuntimeOfflineRenderStemPreview, RuntimeError> {
    let (target_id, resolved_node_ids, resolved_output_bus_ids) = match stem.target_kind {
        RuntimeOfflineRenderTargetKind::MainMix => (
            None,
            topology
                .nodes
                .iter()
                .map(|node| node.node_id.clone())
                .collect::<Vec<_>>(),
            topology
                .nodes
                .iter()
                .map(|node| node.output_bus_id.clone())
                .collect::<Vec<_>>(),
        ),
        RuntimeOfflineRenderTargetKind::TrackLane => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a track lane id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .track_lanes
                .iter()
                .find(|summary| summary.track_lane_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown track lane `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::BusGroup => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a bus group id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .bus_groups
                .iter()
                .find(|summary| summary.bus_group_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown bus group `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::ConsoleGroup => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a console group id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .console_groups
                .iter()
                .find(|summary| summary.console_group_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown console group `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            (
                Some(target_id.to_string()),
                summary.node_ids.clone(),
                summary.output_bus_ids.clone(),
            )
        }
        RuntimeOfflineRenderTargetKind::SendReturn => {
            let target_id = stem.target_id.as_deref().ok_or_else(|| {
                RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    format!(
                        "offline render stem `{}` requires a send/return id",
                        stem.stem_id
                    ),
                )
            })?;
            let summary = topology
                .send_returns
                .iter()
                .find(|summary| summary.send_return_id == target_id)
                .ok_or_else(|| {
                    RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        format!(
                            "offline render stem `{}` references unknown send/return `{}`",
                            stem.stem_id, target_id
                        ),
                    )
                })?;
            let mut node_ids = summary.send_node_ids.clone();
            node_ids.extend(summary.return_node_ids.clone());
            (
                Some(target_id.to_string()),
                node_ids,
                summary.output_bus_ids.clone(),
            )
        }
    };

    let resolved_node_count = resolved_node_ids.len();
    let resolved_output_bus_count = resolved_output_bus_ids.len();
    Ok(RuntimeOfflineRenderStemPreview {
        stem_id: stem.stem_id.clone(),
        target_kind: stem.target_kind,
        target_id,
        resolved_node_ids,
        resolved_output_bus_ids,
        summary: format!(
            "stem={} target={:?}/{:?} nodes={} output_buses={}",
            stem.stem_id,
            stem.target_kind,
            stem.target_id,
            resolved_node_count,
            resolved_output_bus_count,
        ),
    })
}

impl RuntimeOfflinePluginExecutionBoundary {
    pub fn delegated_execution_request(&self) -> RuntimeOfflinePluginDelegatedExecutionRequest {
        let stages = self
            .stages
            .iter()
            .filter(|stage| stage.host_delegate_required)
            .map(|stage| {
                let mut request = RuntimeOfflinePluginDelegatedExecutionStageRequest {
                    stage_id: stage.stage_id.clone(),
                    node_id: stage.node_id.clone(),
                    chain_id: stage.chain_id.clone(),
                    stage_index: stage.stage_index,
                    sandbox_id: stage.sandbox_id.clone(),
                    plugin_type_id: stage.plugin_type_id.clone(),
                    plugin_format: stage.plugin_format,
                    recall_state: stage.recall_state,
                    recall_payload: stage.recall_payload.clone(),
                    override_state: stage.override_state,
                    latest_override_processing_epoch: stage.latest_override_processing_epoch,
                    latest_override_block_sequence: stage.latest_override_block_sequence,
                    summary: String::new(),
                };
                request.summary = format!(
                    "stage={}:{} sandbox={:?} recall={:?} override={:?}",
                    request.chain_id,
                    request.stage_index,
                    request.sandbox_id.as_deref(),
                    request.recall_state,
                    request.override_state,
                );
                request
            })
            .collect::<Vec<_>>();
        let stage_count = stages.len();
        let mut request = RuntimeOfflinePluginDelegatedExecutionRequest {
            request_id: self.request_id.clone(),
            timeline_start_samples: self.timeline_start_samples,
            duration_samples: self.duration_samples,
            runtime_sample_rate_hz: self.runtime_sample_rate_hz,
            export_sample_rate_hz: self.export_sample_rate_hz,
            block_size: self.block_size,
            block_count: self.block_count,
            stage_count,
            stages,
            summary: String::new(),
        };
        request.summary = format!(
            "request={} delegated_stages={} blocks={} sample_rate={}->{}",
            request.request_id,
            request.stage_count,
            request.block_count,
            request.runtime_sample_rate_hz,
            request.export_sample_rate_hz,
        );
        request
    }
}

impl RuntimeOfflineRenderManifest {
    pub fn apply_delegated_execution_receipt(
        &mut self,
        receipt: RuntimeOfflinePluginDelegatedExecutionReceipt,
    ) -> Result<(), RuntimeError> {
        if receipt.request_id != self.request_id {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline plugin execution receipt request `{}` does not match manifest request `{}`",
                    receipt.request_id, self.request_id
                ),
            ));
        }
        if receipt.stage_count != receipt.stages.len() {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt stage_count must match stage receipt count",
            ));
        }
        if self.delegated_execution_request.stage_count != receipt.stage_count {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                format!(
                    "delegated offline plugin execution receipt stages do not match request (request={} receipt={})",
                    self.delegated_execution_request.stage_count, receipt.stage_count
                ),
            ));
        }

        let expected_stage_ids = self
            .delegated_execution_request
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect::<BTreeSet<_>>();
        let receipt_stage_ids = receipt
            .stages
            .iter()
            .map(|stage| stage.stage_id.clone())
            .collect::<BTreeSet<_>>();
        if expected_stage_ids != receipt_stage_ids {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt stage ids must match the delegated request",
            ));
        }
        let completed_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Completed)
            .count();
        let rejected_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Rejected)
            .count();
        let unavailable_stage_count = receipt
            .stages
            .iter()
            .filter(|stage| {
                stage.status == RuntimeOfflinePluginDelegatedExecutionStatus::Unavailable
            })
            .count();
        if receipt.completed_stage_count != completed_stage_count
            || receipt.rejected_stage_count != rejected_stage_count
            || receipt.unavailable_stage_count != unavailable_stage_count
        {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "delegated offline plugin execution receipt status counters must match stage receipt statuses",
            ));
        }

        self.delegated_execution_receipt = Some(receipt);
        self.summary = format!(
            "request={} root={:?} materialized={} artifacts={} report={} delegated_request_stages={} delegated_receipt={}",
            self.request_id,
            self.artifact_root_path.as_deref(),
            self.materialized,
            self.artifact_count,
            self.report.is_some(),
            self.delegated_execution_request.stage_count,
            self.delegated_execution_receipt.is_some(),
        );
        Ok(())
    }
}

fn runtime_lane_for_group(group: GraphNodePlanningGroup) -> GraphExecutionLane {
    match group {
        GraphNodePlanningGroup::InlineRealtime | GraphNodePlanningGroup::StatefulRealtime => {
            GraphExecutionLane::Realtime
        }
        GraphNodePlanningGroup::AnticipativeEligible => GraphExecutionLane::Anticipative,
    }
}

fn apply_transport_session_state(
    summary: &mut TransportSessionSummary,
    active_sessions: &mut BTreeMap<(String, String, String), ActiveTransportSessionRecord>,
    record: &PluginSandboxTransportRecord,
) -> (String, String, String) {
    let key = (
        record.sandbox_id.clone(),
        record.lease_id.clone(),
        record.region_id.clone(),
    );
    let prior_session = active_sessions.remove(&key);
    match record.stage {
        PluginSandboxTransportStage::Attached => {
            summary.current_state = TransportSessionState::AttachActive;
            summary.currently_attached = true;
            summary.active_sandbox_id = Some(record.sandbox_id.clone());
            summary.active_lease_id = Some(record.lease_id.clone());
            summary.active_region_id = Some(record.region_id.clone());
            active_sessions.insert(
                key.clone(),
                ActiveTransportSessionRecord {
                    sandbox_id: record.sandbox_id.clone(),
                    lease_id: record.lease_id.clone(),
                    region_id: record.region_id.clone(),
                    state: TransportSessionState::AttachActive,
                    currently_attached: true,
                    heartbeat_freshness: prior_session
                        .as_ref()
                        .map_or(TransportHeartbeatFreshness::Unknown, |session| {
                            session.heartbeat_freshness
                        }),
                    dispatch_state: prior_session
                        .as_ref()
                        .map_or(TransportDispatchState::Idle, |session| {
                            session.dispatch_state
                        }),
                    processing_epoch: record.processing_epoch.or(prior_session
                        .as_ref()
                        .and_then(|session| session.processing_epoch)),
                    active_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.active_block_sequence),
                    transport_fault_count: prior_session
                        .as_ref()
                        .map_or(0, |session| session.transport_fault_count),
                    last_transport_fault_source: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_source),
                    last_transport_fault_stage: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_stage),
                    last_transport_fault_phase: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_phase),
                    last_transport_fault_processing_epoch: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_processing_epoch),
                    last_transport_fault_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_block_sequence),
                },
            );
        }
        PluginSandboxTransportStage::DetachRequested => {
            summary.current_state = TransportSessionState::DetachRequested;
            summary.currently_attached = true;
            summary.active_sandbox_id = Some(record.sandbox_id.clone());
            summary.active_lease_id = Some(record.lease_id.clone());
            summary.active_region_id = Some(record.region_id.clone());
            active_sessions.insert(
                key.clone(),
                ActiveTransportSessionRecord {
                    sandbox_id: record.sandbox_id.clone(),
                    lease_id: record.lease_id.clone(),
                    region_id: record.region_id.clone(),
                    state: TransportSessionState::DetachRequested,
                    currently_attached: true,
                    heartbeat_freshness: prior_session
                        .as_ref()
                        .map_or(TransportHeartbeatFreshness::Unknown, |session| {
                            session.heartbeat_freshness
                        }),
                    dispatch_state: prior_session
                        .as_ref()
                        .map_or(TransportDispatchState::Idle, |session| {
                            session.dispatch_state
                        }),
                    processing_epoch: record.processing_epoch.or(prior_session
                        .as_ref()
                        .and_then(|session| session.processing_epoch)),
                    active_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.active_block_sequence),
                    transport_fault_count: prior_session
                        .as_ref()
                        .map_or(0, |session| session.transport_fault_count),
                    last_transport_fault_source: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_source),
                    last_transport_fault_stage: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_stage),
                    last_transport_fault_phase: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_phase),
                    last_transport_fault_processing_epoch: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_processing_epoch),
                    last_transport_fault_block_sequence: prior_session
                        .as_ref()
                        .and_then(|session| session.last_transport_fault_block_sequence),
                },
            );
        }
        PluginSandboxTransportStage::Detached => {
            summary.current_state = TransportSessionState::Detached;
            summary.currently_attached = false;
            summary.active_sandbox_id = None;
            summary.active_lease_id = None;
            summary.active_region_id = None;
            summary.active_block_sequence = None;
            active_sessions.remove(&key);
        }
        PluginSandboxTransportStage::DetachFault => {
            summary.current_state = TransportSessionState::DetachFaulted;
            summary.currently_attached = false;
            summary.active_sandbox_id = None;
            summary.active_lease_id = None;
            summary.active_region_id = None;
            summary.active_block_sequence = None;
            active_sessions.remove(&key);
        }
    }
    summary.max_concurrent_attached_sessions = summary
        .max_concurrent_attached_sessions
        .max(active_sessions.len());
    key
}

fn resolve_active_session_mut<'a>(
    active_sessions: &'a mut BTreeMap<(String, String, String), ActiveTransportSessionRecord>,
    sandbox_id: &str,
    lease_id: Option<&str>,
    last_transport_key: Option<&(String, String, String)>,
) -> Option<&'a mut ActiveTransportSessionRecord> {
    if let Some(lease_id) = lease_id {
        if let Some(key) = active_sessions
            .keys()
            .find(|(sandbox, lease, _)| sandbox == sandbox_id && lease == lease_id)
            .cloned()
        {
            return active_sessions.get_mut(&key);
        }
    }

    if let Some(key) = last_transport_key {
        if key.0 == sandbox_id {
            return active_sessions.get_mut(key);
        }
    }

    let fallback_key = active_sessions
        .keys()
        .rev()
        .find(|(sandbox, _, _)| sandbox == sandbox_id)
        .cloned()?;
    active_sessions.get_mut(&fallback_key)
}

fn update_transport_session_epoch_bounds(
    summary: &mut TransportSessionSummary,
    epoch: Option<u64>,
) {
    if let Some(epoch) = epoch {
        summary.first_processing_epoch = Some(
            summary
                .first_processing_epoch
                .map_or(epoch, |current| current.min(epoch)),
        );
        summary.last_processing_epoch = Some(
            summary
                .last_processing_epoch
                .map_or(epoch, |current| current.max(epoch)),
        );
    }
}

fn update_transport_session_block_bounds(
    summary: &mut TransportSessionSummary,
    block_sequence: Option<u64>,
) {
    if let Some(block_sequence) = block_sequence {
        summary.first_block_sequence = Some(
            summary
                .first_block_sequence
                .map_or(block_sequence, |current| current.min(block_sequence)),
        );
        summary.last_block_sequence = Some(
            summary
                .last_block_sequence
                .map_or(block_sequence, |current| current.max(block_sequence)),
        );
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeEvent {
    ReadinessChanged(RuntimeReadiness),
    EffectiveConfigChanged(EffectiveRuntimeConfig),
    SupervisionChanged(RuntimeSupervisionSnapshot),
    PluginSandboxChanged {
        active_sandboxes: u32,
    },
    PluginSandboxFault {
        sandbox_id: String,
        kind: PluginFaultKind,
        detail: String,
        processing_epoch: Option<u64>,
    },
    RecoveryCycle {
        sandbox_id: String,
        intent: RecoveryRestartIntent,
        stop_reason: StopReason,
        processing_epoch: Option<u64>,
    },
    PluginSandboxLifecycle {
        sandbox_id: String,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    },
    PluginSandboxInstanceState {
        state: PluginSandboxInstanceStateRecord,
    },
    PluginSandboxTransport {
        sandbox_id: String,
        lease_id: String,
        region_id: String,
        stage: PluginSandboxTransportStage,
        processing_epoch: Option<u64>,
        detail: Option<String>,
    },
    HeartbeatCycle {
        sandbox_id: String,
        stage: HeartbeatCycleStage,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
    },
    BlockDispatch {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: u64,
        frame_count: u32,
        stage: BlockDispatchStage,
        completion_state: Option<CompletionState>,
    },
    LeaseRollover {
        sandbox_id: String,
        previous_lease_id: String,
        lease_id: String,
        processing_epoch: u64,
        first_block_sequence: u64,
    },
    BrokerInvalidation {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: Option<u64>,
        stage: BrokerInvalidationStage,
        reason: String,
    },
    CompletionSlotTransition {
        sandbox_id: String,
        lease_id: String,
        processing_epoch: u64,
        block_sequence: u64,
        stage: CompletionSlotStage,
    },
    BrokerFailure {
        sandbox_id: String,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        block_sequence: Option<u64>,
        stage: BrokerFailureStage,
        detail: String,
    },
    SandboxOperationFailure {
        sandbox_id: String,
        lease_id: Option<String>,
        processing_epoch: Option<u64>,
        operation: String,
        error_kind: String,
        stage: SandboxOperationFailureStage,
        detail: String,
    },
    HardwareDeviceChanged {
        device_id: Option<String>,
    },
}

pub trait RuntimeEventSink: Send {
    fn push(&mut self, event: RuntimeEvent);
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginFaultRecord {
    pub sandbox_id: String,
    pub kind: PluginFaultKind,
    pub detail: String,
    pub processing_epoch: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeObservationDiagnostics {
    pub total_events: usize,
    pub supervision_updates: Vec<RuntimeSupervisionSnapshot>,
    pub plugin_faults: Vec<PluginFaultRecord>,
    pub plugin_instance_states: Vec<PluginSandboxInstanceStateRecord>,
    pub recovery_events: Vec<RecoveryRecord>,
    pub lifecycle_events: Vec<PluginSandboxLifecycleRecord>,
    pub transport_events: Vec<PluginSandboxTransportRecord>,
    pub heartbeat_events: Vec<HeartbeatCycleRecord>,
    pub block_dispatch_events: Vec<BlockDispatchRecord>,
    pub lease_rollover_events: Vec<LeaseRolloverRecord>,
    pub invalidation_events: Vec<BrokerInvalidationRecord>,
    pub completion_slot_events: Vec<CompletionSlotRecord>,
    pub transport_fault_events: Vec<TransportFaultRecord>,
    pub broker_failure_events: Vec<BrokerFailureRecord>,
    pub sandbox_operation_failure_events: Vec<SandboxOperationFailureRecord>,
}

impl RuntimeObservationDiagnostics {
    pub fn supervision_update_count(&self) -> usize {
        self.supervision_updates.len()
    }

    pub fn plugin_fault_count(&self) -> usize {
        self.plugin_faults.len()
    }

    pub fn plugin_instance_state_event_count(&self) -> usize {
        self.plugin_instance_states.len()
    }

    pub fn recovery_event_count(&self) -> usize {
        self.recovery_events.len()
    }

    pub fn lifecycle_event_count(&self) -> usize {
        self.lifecycle_events.len()
    }

    pub fn transport_event_count(&self) -> usize {
        self.transport_events.len()
    }

    pub fn heartbeat_event_count(&self) -> usize {
        self.heartbeat_events.len()
    }

    pub fn block_dispatch_event_count(&self) -> usize {
        self.block_dispatch_events.len()
    }

    pub fn lease_rollover_event_count(&self) -> usize {
        self.lease_rollover_events.len()
    }

    pub fn invalidation_event_count(&self) -> usize {
        self.invalidation_events.len()
    }

    pub fn completion_slot_event_count(&self) -> usize {
        self.completion_slot_events.len()
    }

    pub fn transport_fault_event_count(&self) -> usize {
        self.transport_fault_events.len()
    }

    pub fn broker_failure_event_count(&self) -> usize {
        self.broker_failure_events.len()
    }

    pub fn sandbox_operation_failure_event_count(&self) -> usize {
        self.sandbox_operation_failure_events.len()
    }

    pub fn fault_detail_count_containing(&self, needle: &str) -> usize {
        self.plugin_faults
            .iter()
            .filter(|fault| fault.detail.contains(needle))
            .count()
    }

    pub fn last_supervision_update(&self) -> Option<&RuntimeSupervisionSnapshot> {
        self.supervision_updates.last()
    }

    pub fn last_recovery_event(&self) -> Option<&RecoveryRecord> {
        self.recovery_events.last()
    }

    pub fn last_plugin_instance_state(&self) -> Option<&PluginSandboxInstanceStateRecord> {
        self.plugin_instance_states.last()
    }

    pub fn last_lifecycle_event(&self) -> Option<&PluginSandboxLifecycleRecord> {
        self.lifecycle_events.last()
    }

    pub fn last_transport_event(&self) -> Option<&PluginSandboxTransportRecord> {
        self.transport_events.last()
    }

    pub fn last_heartbeat_event(&self) -> Option<&HeartbeatCycleRecord> {
        self.heartbeat_events.last()
    }

    pub fn last_block_dispatch_event(&self) -> Option<&BlockDispatchRecord> {
        self.block_dispatch_events.last()
    }

    pub fn last_lease_rollover_event(&self) -> Option<&LeaseRolloverRecord> {
        self.lease_rollover_events.last()
    }

    pub fn last_invalidation_event(&self) -> Option<&BrokerInvalidationRecord> {
        self.invalidation_events.last()
    }

    pub fn last_completion_slot_event(&self) -> Option<&CompletionSlotRecord> {
        self.completion_slot_events.last()
    }

    pub fn last_transport_fault_event(&self) -> Option<&TransportFaultRecord> {
        self.transport_fault_events.last()
    }

    pub fn last_broker_failure_event(&self) -> Option<&BrokerFailureRecord> {
        self.broker_failure_events.last()
    }

    pub fn last_sandbox_operation_failure_event(&self) -> Option<&SandboxOperationFailureRecord> {
        self.sandbox_operation_failure_events.last()
    }

    pub fn render_compact(&self) -> String {
        let last_trigger = self
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
            .map(|trigger| format!("{trigger:?}"))
            .unwrap_or_else(|| "none".into());
        let last_fault = self
            .plugin_faults
            .last()
            .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
            .unwrap_or_else(|| "none".into());
        let last_recovery = self
            .last_recovery_event()
            .map(|recovery| {
                format!(
                    "{}:{:?}:{:?}@{:?}",
                    recovery.sandbox_id,
                    recovery.intent,
                    recovery.stop_reason,
                    recovery.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_plugin_instance_state = self
            .last_plugin_instance_state()
            .map(|state| {
                format!(
                    "{}:{}:{}/{}/active={}@{:?}",
                    state.sandbox_id,
                    state.instance_id,
                    state.lifecycle_state,
                    state.readiness_state,
                    state.active,
                    state.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_lifecycle = self
            .last_lifecycle_event()
            .map(|lifecycle| {
                format!(
                    "{}:{:?}@{:?}",
                    lifecycle.sandbox_id, lifecycle.stage, lifecycle.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_transport = self
            .last_transport_event()
            .map(|transport| {
                format!(
                    "{}:{}:{}:{:?}@{:?}",
                    transport.sandbox_id,
                    transport.lease_id,
                    transport.region_id,
                    transport.stage,
                    transport.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_heartbeat = self
            .last_heartbeat_event()
            .map(|heartbeat| {
                format!(
                    "{}:{:?}@{:?}/block={:?}",
                    heartbeat.sandbox_id,
                    heartbeat.stage,
                    heartbeat.processing_epoch,
                    heartbeat.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_dispatch = self
            .last_block_dispatch_event()
            .map(|dispatch| {
                format!(
                    "{}:{}:{:?}/block={}@{}",
                    dispatch.sandbox_id,
                    dispatch.lease_id,
                    dispatch.stage,
                    dispatch.block_sequence,
                    dispatch.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_rollover = self
            .last_lease_rollover_event()
            .map(|rollover| {
                format!(
                    "{}:{}->{}@{}/block={}",
                    rollover.sandbox_id,
                    rollover.previous_lease_id,
                    rollover.lease_id,
                    rollover.processing_epoch,
                    rollover.first_block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_invalidation = self
            .last_invalidation_event()
            .map(|invalidation| {
                format!(
                    "{}:{}:{:?}@{}/block={:?}",
                    invalidation.sandbox_id,
                    invalidation.lease_id,
                    invalidation.stage,
                    invalidation.processing_epoch,
                    invalidation.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_completion_slot = self
            .last_completion_slot_event()
            .map(|completion| {
                format!(
                    "{}:{}:{:?}@{}/block={}",
                    completion.sandbox_id,
                    completion.lease_id,
                    completion.stage,
                    completion.processing_epoch,
                    completion.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_transport_fault = self
            .last_transport_fault_event()
            .map(|failure| {
                format!(
                    "{}:{:?}:{:?}:{:?}:{:?}:lease={:?}@{:?}/block={:?}",
                    failure.sandbox_id,
                    failure.source,
                    failure.stage,
                    failure.phase,
                    failure.resource,
                    failure.lease_id,
                    failure.processing_epoch,
                    failure.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_broker_failure = self
            .last_broker_failure_event()
            .map(|failure| {
                format!(
                    "{}:{:?}:lease={:?}@{:?}/block={:?}",
                    failure.sandbox_id,
                    failure.stage,
                    failure.lease_id,
                    failure.processing_epoch,
                    failure.block_sequence
                )
            })
            .unwrap_or_else(|| "none".into());
        let last_sandbox_operation_failure = self
            .last_sandbox_operation_failure_event()
            .map(|failure| {
                format!(
                    "{}:{}:{:?}:lease={:?}@{:?}",
                    failure.sandbox_id,
                    failure.operation,
                    failure.stage,
                    failure.lease_id,
                    failure.processing_epoch
                )
            })
            .unwrap_or_else(|| "none".into());

        format!(
            "events={} supervision_updates={} plugin_faults={} plugin_instance_states={} recovery_events={} lifecycle_events={} transport_events={} heartbeat_events={} block_dispatch_events={} lease_rollover_events={} invalidation_events={} completion_slot_events={} transport_fault_events={} broker_failure_events={} sandbox_operation_failure_events={} last_watchdog={} last_fault={} last_plugin_instance_state={} last_recovery={} last_lifecycle={} last_transport={} last_heartbeat={} last_dispatch={} last_rollover={} last_invalidation={} last_completion_slot={} last_transport_fault={} last_broker_failure={} last_sandbox_operation_failure={}",
            self.total_events,
            self.supervision_update_count(),
            self.plugin_fault_count(),
            self.plugin_instance_state_event_count(),
            self.recovery_event_count(),
            self.lifecycle_event_count(),
            self.transport_event_count(),
            self.heartbeat_event_count(),
            self.block_dispatch_event_count(),
            self.lease_rollover_event_count(),
            self.invalidation_event_count(),
            self.completion_slot_event_count(),
            self.transport_fault_event_count(),
            self.broker_failure_event_count(),
            self.sandbox_operation_failure_event_count(),
            last_trigger,
            last_fault,
            last_plugin_instance_state,
            last_recovery,
            last_lifecycle,
            last_transport,
            last_heartbeat,
            last_dispatch,
            last_rollover,
            last_invalidation,
            last_completion_slot,
            last_transport_fault,
            last_broker_failure,
            last_sandbox_operation_failure,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeObservationReport {
    pub readiness: RuntimeReadiness,
    pub effective_config: EffectiveRuntimeConfig,
    pub control_snapshot: RuntimeControlSnapshot,
    pub scheduler_snapshot: RuntimeSchedulerSnapshot,
    pub diagnostics_snapshot: RuntimeDiagnosticsSnapshot,
    pub metering_snapshot: RuntimeMeteringSnapshot,
    pub supervision_snapshot: RuntimeSupervisionSnapshot,
    pub fault_status: RuntimeFaultStatusSnapshot,
    pub fault_diagnostic_receipt: RuntimeFaultDiagnosticReceipt,
    pub interruption_summary: RuntimeInterruptionSummary,
    pub device_supervision_snapshot: RuntimeDeviceSupervisionSnapshot,
    pub external_io_snapshot: RuntimeExternalIoSnapshot,
    pub linux_backend_session_snapshot: RuntimeLinuxBackendSessionSnapshot,
    pub pipewire_alsa_parity_snapshot: RuntimePipeWireAlsaParitySnapshot,
    pub jack_coordination_snapshot: RuntimeJackCoordinationSnapshot,
    pub external_midi_snapshot: RuntimeExternalMidiEndpointGraphSnapshot,
    pub control_surface_snapshot: RuntimeControlSurfaceSnapshot,
    pub advanced_hardware_snapshot: RuntimeAdvancedHardwareSnapshot,
    pub timeline_snapshot: RuntimeTimelineSnapshot,
    pub tempo_map_snapshot: RuntimeTempoMapSnapshot,
    pub warp_pipeline_snapshot: RuntimeWarpPipelineSnapshot,
    pub clip_processing_pipeline_snapshot: RuntimeClipProcessingPipelineSnapshot,
    pub stretch_engine_snapshot: RuntimeStretchEngineSnapshot,
    pub marker_analysis_snapshot: RuntimeMarkerAnalysisSnapshot,
    pub transform_artifact_snapshot: RuntimeTransformArtifactSnapshot,
    pub preview_transform_snapshot: RuntimePreviewTransformServiceSnapshot,
    pub recording_capture_snapshot: RuntimeRecordingCaptureSnapshot,
    pub media_pipeline_snapshot: RuntimeMediaPipelineSnapshot,
    pub media_service_snapshot: RuntimeMediaServiceSnapshot,
    pub media_library_snapshot: RuntimeMediaLibraryServiceSnapshot,
    pub offline_render_session_snapshot: RuntimeOfflineRenderSessionSnapshot,
    pub automation_snapshot: RuntimeAutomationSnapshot,
    pub plugin_event_snapshot: RuntimePluginEventSnapshot,
    pub engine_block_snapshot: RuntimeEngineBlockSnapshot,
    pub transport_concurrency_snapshot: RuntimeTransportConcurrencySnapshot,
    pub plugin_discovery_snapshot: RuntimePluginDiscoverySnapshot,
    pub plugin_lifecycle_snapshot: RuntimePluginLifecycleSnapshot,
    pub lv2_extension_snapshot: RuntimeLv2ExtensionSnapshot,
    pub plugin_pin_matrix_snapshot: RuntimePluginPinMatrixSnapshot,
    pub plugin_chain_snapshot: RuntimePluginChainSnapshot,
    pub scheduler_summary: RuntimeSchedulerExportSummary,
    pub block_summary: RuntimeBlockExecutionSummary,
    pub degradation_summary: RuntimeDegradationSummary,
    pub execution_topology_summary: RuntimeExecutionTopologySummary,
    pub transport_fault_summary: TransportFaultSummary,
    pub transport_session_summary: TransportSessionSummary,
    pub last_deferred_service_receipt: Option<RuntimeDeferredServiceReceipt>,
    pub observation: RuntimeObservationDiagnostics,
}

impl RuntimeObservationReport {
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        let observation = recorder.diagnostics();
        let readiness = runtime.get_readiness();
        let effective_config = runtime.get_effective_config();
        let control_snapshot = runtime.get_control_snapshot();
        let scheduler_snapshot = runtime.get_scheduler_snapshot();
        let diagnostics_snapshot = runtime.get_diagnostics_snapshot();
        let metering_snapshot = runtime.get_metering_snapshot();
        let supervision_snapshot = runtime.get_supervision_snapshot();
        let timeline_snapshot = runtime.get_timeline_snapshot();
        let tempo_map_snapshot = runtime.get_tempo_map_snapshot();
        let warp_pipeline_snapshot = runtime.get_warp_pipeline_snapshot();
        let clip_processing_pipeline_snapshot = runtime.get_clip_processing_pipeline_snapshot();
        let stretch_engine_snapshot = runtime.get_stretch_engine_snapshot();
        let marker_analysis_snapshot = runtime.get_marker_analysis_snapshot();
        let transform_artifact_snapshot = runtime.get_transform_artifact_snapshot();
        let preview_transform_snapshot = runtime.get_preview_transform_snapshot();
        let recording_capture_snapshot = runtime.get_recording_capture_snapshot();
        let media_pipeline_snapshot = runtime.get_media_pipeline_snapshot();
        let media_service_snapshot = runtime.get_media_service_snapshot();
        let media_library_snapshot = runtime.get_media_library_service_snapshot();
        let offline_render_session_snapshot = runtime.get_offline_render_session_snapshot();
        let automation_snapshot = runtime.get_automation_snapshot();
        let plugin_event_snapshot = runtime.get_plugin_event_snapshot();
        let engine_block_snapshot = runtime.get_engine_block_snapshot();
        let execution_topology_summary = runtime.get_execution_topology_summary();
        let transport_concurrency_snapshot = runtime.get_transport_concurrency_snapshot();
        let plugin_discovery_snapshot = runtime.get_plugin_discovery_snapshot();
        let plugin_lifecycle_snapshot = runtime.get_plugin_lifecycle_snapshot();
        let plugin_chain_snapshot = runtime.get_plugin_chain_snapshot();
        let lv2_extension_snapshot = RuntimeLv2ExtensionSnapshot::capture(
            &plugin_discovery_snapshot,
            &plugin_lifecycle_snapshot,
        );
        let plugin_pin_matrix_snapshot = RuntimePluginPinMatrixSnapshot::capture(
            &plugin_discovery_snapshot,
            &plugin_lifecycle_snapshot,
            &plugin_chain_snapshot,
        );
        let last_deferred_service_receipt = runtime.get_last_deferred_service_receipt();
        let scheduler_summary =
            RuntimeSchedulerExportSummary::from_snapshot(&engine_block_snapshot);
        let block_summary = RuntimeBlockExecutionSummary::from_snapshot(&engine_block_snapshot);
        let fault_status = RuntimeFaultStatusSnapshot::capture(
            readiness.clone(),
            &control_snapshot,
            &diagnostics_snapshot,
            &supervision_snapshot,
            &engine_block_snapshot,
            &transport_concurrency_snapshot,
            &plugin_lifecycle_snapshot,
            false,
            0,
        );
        let degradation_summary = RuntimeDegradationSummary::capture(
            &readiness,
            diagnostics_snapshot,
            &supervision_snapshot,
            &engine_block_snapshot,
            &transport_concurrency_snapshot,
            &observation,
        );
        let interruption_summary = RuntimeInterruptionSummary::capture(
            &fault_status,
            last_deferred_service_receipt.as_ref(),
        );
        let fault_diagnostic_receipt = RuntimeFaultDiagnosticReceipt::capture(
            &fault_status,
            &interruption_summary,
            &degradation_summary,
            &engine_block_snapshot,
            last_deferred_service_receipt.as_ref(),
            None,
        );
        let device_supervision_snapshot = RuntimeDeviceSupervisionSnapshot::capture(
            &effective_config,
            &supervision_snapshot,
            &fault_status,
            &interruption_summary,
            None,
        );
        let external_io_snapshot = RuntimeHostIoSummary::unavailable_external_io_snapshot(
            &effective_config,
            &device_supervision_snapshot,
        );
        let linux_backend_session_snapshot = RuntimeLinuxBackendSessionSnapshot::unavailable();
        let pipewire_alsa_parity_snapshot = RuntimePipeWireAlsaParitySnapshot::unavailable();
        let jack_coordination_snapshot = RuntimeJackCoordinationSnapshot::unavailable();
        let external_midi_snapshot = RuntimeExternalMidiEndpointGraphSnapshot::unavailable();
        let control_surface_snapshot =
            RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(&external_midi_snapshot);
        let advanced_hardware_snapshot =
            RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
                &control_surface_snapshot,
            );
        Self {
            readiness: readiness.clone(),
            effective_config,
            control_snapshot,
            scheduler_snapshot,
            diagnostics_snapshot,
            metering_snapshot,
            supervision_snapshot: supervision_snapshot.clone(),
            fault_status,
            fault_diagnostic_receipt,
            interruption_summary,
            device_supervision_snapshot,
            external_io_snapshot,
            linux_backend_session_snapshot,
            pipewire_alsa_parity_snapshot,
            jack_coordination_snapshot,
            external_midi_snapshot,
            control_surface_snapshot,
            advanced_hardware_snapshot,
            timeline_snapshot,
            tempo_map_snapshot,
            warp_pipeline_snapshot,
            clip_processing_pipeline_snapshot,
            stretch_engine_snapshot,
            marker_analysis_snapshot,
            transform_artifact_snapshot,
            preview_transform_snapshot,
            recording_capture_snapshot,
            media_pipeline_snapshot,
            media_service_snapshot,
            media_library_snapshot,
            offline_render_session_snapshot,
            automation_snapshot,
            plugin_event_snapshot,
            engine_block_snapshot,
            transport_concurrency_snapshot,
            plugin_discovery_snapshot,
            plugin_lifecycle_snapshot,
            lv2_extension_snapshot,
            plugin_pin_matrix_snapshot,
            plugin_chain_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            execution_topology_summary,
            transport_fault_summary: TransportFaultSummary::from_records(
                &observation.transport_fault_events,
            ),
            transport_session_summary: TransportSessionSummary::from_diagnostics(&observation),
            last_deferred_service_receipt,
            observation,
        }
    }

    pub fn with_host_device_supervision(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.device_supervision_snapshot = RuntimeDeviceSupervisionSnapshot::capture(
            &self.effective_config,
            &self.supervision_snapshot,
            &self.fault_status,
            &self.interruption_summary,
            Some(host_io),
        );
        self
    }

    pub fn with_host_external_io(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.external_io_snapshot = host_io.build_external_io_snapshot();
        self
    }

    pub fn with_linux_backend_session_snapshot(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.linux_backend_session_snapshot =
            RuntimeLinuxBackendSessionSnapshot::from_host_io(host_io);
        self
    }

    pub fn with_pipewire_alsa_parity_snapshot(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.pipewire_alsa_parity_snapshot =
            RuntimePipeWireAlsaParitySnapshot::from_host_io_and_linux_session(
                host_io,
                &self.linux_backend_session_snapshot,
            );
        self
    }

    pub fn with_jack_coordination_snapshot(mut self, host_io: &RuntimeHostIoSummary) -> Self {
        self.jack_coordination_snapshot =
            RuntimeJackCoordinationSnapshot::from_host_io_and_transport_session(
                host_io,
                &self.transport_session_summary,
            );
        self
    }

    pub fn with_external_midi_snapshot(
        mut self,
        external_midi_snapshot: RuntimeExternalMidiEndpointGraphSnapshot,
    ) -> Self {
        let external_midi_snapshot = external_midi_snapshot.with_live_ownership_summary(
            &self.linux_backend_session_snapshot,
            &self.interruption_summary,
        );
        self.control_surface_snapshot =
            RuntimeControlSurfaceSnapshot::from_external_midi_snapshot(&external_midi_snapshot);
        self.advanced_hardware_snapshot =
            RuntimeAdvancedHardwareSnapshot::from_control_surface_snapshot(
                &self.control_surface_snapshot,
            );
        self.external_midi_snapshot = external_midi_snapshot;
        self
    }

    pub fn render_compact(&self) -> String {
        render_runtime_observation_report_compact(self)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSupervisorReport {
    pub observation: RuntimeObservationReport,
    pub events: Vec<RuntimeEvent>,
}

impl RuntimeSupervisorReport {
    pub fn capture(runtime: &impl RuntimeObservationApi, recorder: &RuntimeEventRecorder) -> Self {
        Self {
            observation: RuntimeObservationReport::capture(runtime, recorder),
            events: recorder.snapshot(),
        }
    }

    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    pub fn supervision_update_count(&self) -> usize {
        self.observation.observation.supervision_update_count()
    }

    pub fn plugin_fault_count(&self) -> usize {
        self.observation.observation.plugin_fault_count()
    }

    pub fn plugin_instance_state_event_count(&self) -> usize {
        self.observation
            .observation
            .plugin_instance_state_event_count()
    }

    pub fn recovery_event_count(&self) -> usize {
        self.observation.observation.recovery_event_count()
    }

    pub fn lifecycle_event_count(&self) -> usize {
        self.observation.observation.lifecycle_event_count()
    }

    pub fn transport_event_count(&self) -> usize {
        self.observation.observation.transport_event_count()
    }

    pub fn heartbeat_event_count(&self) -> usize {
        self.observation.observation.heartbeat_event_count()
    }

    pub fn block_dispatch_event_count(&self) -> usize {
        self.observation.observation.block_dispatch_event_count()
    }

    pub fn lease_rollover_event_count(&self) -> usize {
        self.observation.observation.lease_rollover_event_count()
    }

    pub fn invalidation_event_count(&self) -> usize {
        self.observation.observation.invalidation_event_count()
    }

    pub fn completion_slot_event_count(&self) -> usize {
        self.observation.observation.completion_slot_event_count()
    }

    pub fn transport_fault_event_count(&self) -> usize {
        self.observation.observation.transport_fault_event_count()
    }

    pub fn broker_failure_event_count(&self) -> usize {
        self.observation.observation.broker_failure_event_count()
    }

    pub fn sandbox_operation_failure_event_count(&self) -> usize {
        self.observation
            .observation
            .sandbox_operation_failure_event_count()
    }

    pub fn last_watchdog_trigger(&self) -> Option<RuntimeWatchdogTrigger> {
        self.observation
            .observation
            .last_supervision_update()
            .and_then(|snapshot| snapshot.last_watchdog_trigger)
    }

    pub fn render_compact(&self) -> String {
        format!(
            "{} event_stream={}",
            self.observation.render_compact(),
            self.event_count()
        )
    }

    pub fn render_multiline(&self) -> String {
        let tempo_map = (self.observation.tempo_map_snapshot.segment_count > 0)
            .then(|| {
                format_runtime_tempo_map_snapshot_multiline(&self.observation.tempo_map_snapshot)
            })
            .unwrap_or_default();
        let warp = (self.observation.warp_pipeline_snapshot.clip_count > 0)
            .then(|| {
                format_runtime_warp_pipeline_snapshot_multiline(
                    &self.observation.warp_pipeline_snapshot,
                )
            })
            .unwrap_or_default();
        let clip_processing = (self
            .observation
            .clip_processing_pipeline_snapshot
            .clip_count
            > 0)
        .then(|| {
            format_runtime_clip_processing_pipeline_snapshot_multiline(
                &self.observation.clip_processing_pipeline_snapshot,
            )
        })
        .unwrap_or_default();
        let stretch_engine = (self.observation.stretch_engine_snapshot.clip_count > 0)
            .then(|| {
                format_runtime_stretch_engine_snapshot_multiline(
                    &self.observation.stretch_engine_snapshot,
                )
            })
            .unwrap_or_default();
        let marker_analysis = (self.observation.marker_analysis_snapshot.clip_count > 0)
            .then(|| {
                format_runtime_marker_analysis_snapshot_multiline(
                    &self.observation.marker_analysis_snapshot,
                )
            })
            .unwrap_or_default();
        let transform_artifact = (self.observation.transform_artifact_snapshot.clip_count > 0)
            .then(|| {
                format_runtime_transform_artifact_snapshot_multiline(
                    &self.observation.transform_artifact_snapshot,
                )
            })
            .unwrap_or_default();
        let media_pipeline = (self.observation.media_pipeline_snapshot.asset_count > 0)
            .then(|| {
                format_runtime_media_pipeline_snapshot_multiline(
                    &self.observation.media_pipeline_snapshot,
                )
            })
            .unwrap_or_default();
        let media_service = (self.observation.media_service_snapshot.indexed_asset_count > 0
            || self.observation.media_service_snapshot.invalidation_active
            || matches!(
                self.observation.media_service_snapshot.preview_state,
                RuntimeMediaPreviewState::Previewing | RuntimeMediaPreviewState::Invalidated
            ))
        .then(|| {
            format_runtime_media_service_snapshot_multiline(
                &self.observation.media_service_snapshot,
            )
        })
        .unwrap_or_default();
        let media_library = (self.observation.media_library_snapshot.indexed_asset_count > 0)
            .then(|| {
                format_runtime_media_library_service_snapshot_multiline(
                    &self.observation.media_library_snapshot,
                )
            })
            .unwrap_or_default();
        let plugin_discovery = (self.observation.plugin_discovery_snapshot.scan_count > 0)
            .then(|| {
                format_runtime_plugin_discovery_snapshot_multiline(
                    &self.observation.plugin_discovery_snapshot,
                )
            })
            .unwrap_or_default();
        let plugin_lifecycle = (self.observation.plugin_lifecycle_snapshot.sandbox_count > 0)
            .then(|| {
                format_runtime_plugin_lifecycle_snapshot_multiline(
                    &self.observation.plugin_lifecycle_snapshot,
                )
            })
            .unwrap_or_default();
        let lv2_extension = (self.observation.lv2_extension_snapshot.plugin_type_count > 0)
            .then(|| {
                format_runtime_lv2_extension_snapshot_multiline(
                    &self.observation.lv2_extension_snapshot,
                )
            })
            .unwrap_or_default();
        let plugin_pin_matrix = (self
            .observation
            .plugin_pin_matrix_snapshot
            .plugin_type_count
            > 0)
        .then(|| {
            format_runtime_plugin_pin_matrix_snapshot_multiline(
                &self.observation.plugin_pin_matrix_snapshot,
            )
        })
        .unwrap_or_default();
        let plugin_chain = (self.observation.plugin_chain_snapshot.chain_count > 0)
            .then(|| {
                format_runtime_plugin_chain_snapshot_multiline(
                    &self.observation.plugin_chain_snapshot,
                )
            })
            .unwrap_or_default();
        let _automation = (self.observation.automation_snapshot.parameter_id != 0
            || self.observation.automation_snapshot.lane_count > 0
            || self.observation.automation_snapshot.last_batch_epoch.is_some())
            .then(|| {
                let snapshot = &self.observation.automation_snapshot;
                format!(
                    "\nautomation_param={}\nautomation_lane_count={}\nautomation_point_count={}\nautomation_projected_segment_count={}\nautomation_mapped_lanes={}\nautomation_unmapped_lanes={}\nautomation_hold_lanes={}\nautomation_linear_lanes={}\nautomation_last_batch_epoch={:?}\nautomation_last_batch_event_count={}\nautomation_last_batch_ignored_event_count={}\nautomation_last_batch_sub_block_count={}\nautomation_last_batch_coalesced_event_count={}\nautomation_last_batch_strategy_max_sub_blocks={}\nautomation_last_batch_min_ramp_step_samples={:?}\nautomation_last_batch_max_sample_offset={:?}\nautomation_last_block_sequence={:?}\nautomation_last_timeline_position_samples={:?}\nautomation_transport_playing={:?}\nautomation_value_events={}\nautomation_modulation_events={}\nautomation_gesture_begin_events={}\nautomation_gesture_end_events={}\nautomation_first_value={:?}\nautomation_last_value={:?}\nautomation_last_modulation={:?}\nautomation_first_epoch={:?}\nautomation_last_epoch={:?}\nautomation_segments={}\nautomation_segment_epochs={:?}\nautomation_lease_rollovers={}",
                    snapshot.parameter_id,
                    snapshot.lane_count,
                    snapshot.point_count,
                    snapshot.projected_segment_count,
                    snapshot.mapped_lane_count,
                    snapshot.unmapped_lane_count,
                    snapshot.hold_lane_count,
                    snapshot.linear_lane_count,
                    snapshot.last_batch_epoch,
                    snapshot.last_batch_event_count,
                    snapshot.last_batch_ignored_event_count,
                    snapshot.last_batch_sub_block_count,
                    snapshot.last_batch_coalesced_event_count,
                    snapshot.last_batch_strategy_max_sub_blocks,
                    snapshot.last_batch_min_ramp_step_samples,
                    snapshot.last_batch_max_sample_offset,
                    snapshot.last_block_sequence,
                    snapshot.last_timeline_position_samples,
                    snapshot.transport_playing,
                    snapshot.value_events,
                    snapshot.modulation_events,
                    snapshot.gesture_begin_events,
                    snapshot.gesture_end_events,
                    snapshot.first_value,
                    snapshot.last_value,
                    snapshot.last_modulation,
                    snapshot.first_epoch,
                    snapshot.last_epoch,
                    snapshot.segment_count,
                    snapshot.segment_epochs,
                    snapshot.lease_rollovers,
                )
            })
            .unwrap_or_default();
        let _transport_timeline = format!(
            "\ntransport_epoch={}\ntransport_transition={:?}\ntransport_transition_epoch={:?}\ntransport_transition_block={:?}\ntransport_playing={:?}\ntransport_tempo_bpm={:?}\ntransport_timeline_position_samples={:?}\ntransport_loop_start_samples={:?}\ntransport_loop_end_samples={:?}\ntransport_last_block_start_samples={:?}\ntransport_last_block_end_samples={:?}\ntransport_loop_wrap_count={}",
            self.observation.timeline_snapshot.transport_epoch,
            self.observation.timeline_snapshot.last_transport_transition,
            self.observation
                .timeline_snapshot
                .last_transport_transition_processing_epoch,
            self.observation
                .timeline_snapshot
                .last_transport_transition_block_sequence,
            self.observation.timeline_snapshot.last_transport_playing,
            self.observation.timeline_snapshot.last_transport_tempo_bpm,
            self.observation
                .timeline_snapshot
                .last_transport_timeline_position_samples,
            self.observation.timeline_snapshot.last_transport_loop_start_samples,
            self.observation.timeline_snapshot.last_transport_loop_end_samples,
            self.observation.timeline_snapshot.last_engine_block_start_samples,
            self.observation.timeline_snapshot.last_engine_block_end_samples,
            self.observation.timeline_snapshot.loop_wrap_count,
        );
        let _engine_transport = format!(
            "\nengine_transport_epoch={}\nengine_transport_transition={:?}\nengine_transport_block_start_samples={:?}\nengine_transport_block_end_samples={:?}\nengine_transport_loop_wrapped={}",
            self.observation.engine_block_snapshot.transport_epoch,
            self.observation.engine_block_snapshot.transport_transition,
            self.observation
                .engine_block_snapshot
                .transport_block_start_samples,
            self.observation.engine_block_snapshot.transport_block_end_samples,
            self.observation.engine_block_snapshot.transport_loop_wrapped,
        );
        let _scheduler_topology = format_scheduler_topology_multiline(
            &self.observation.engine_block_snapshot.scheduler_topology,
        );
        let _scheduler_snapshot =
            format_runtime_scheduler_snapshot_multiline(&self.observation.scheduler_snapshot);
        let _scheduler_summary =
            format_runtime_scheduler_summary_multiline(&self.observation.scheduler_summary);
        let _block_summary =
            format_runtime_block_summary_multiline(&self.observation.block_summary);
        let _degradation_summary =
            format_runtime_degradation_summary_multiline(&self.observation.degradation_summary);
        let _fault_status = format_runtime_fault_status_multiline(&self.observation.fault_status);
        let _fault_diagnostic_receipt = format_runtime_fault_diagnostic_receipt_multiline(
            &self.observation.fault_diagnostic_receipt,
        );
        let _interruption_summary =
            format_runtime_interruption_summary_multiline(&self.observation.interruption_summary);
        let device_supervision_summary = format_runtime_device_supervision_snapshot_multiline(
            &self.observation.device_supervision_snapshot,
        );
        let external_io_summary =
            format_runtime_external_io_snapshot_multiline(&self.observation.external_io_snapshot);
        let linux_backend_session_summary = format_runtime_linux_backend_session_snapshot_multiline(
            &self.observation.linux_backend_session_snapshot,
        );
        let pipewire_alsa_parity_summary = format_runtime_pipewire_alsa_parity_snapshot_multiline(
            &self.observation.pipewire_alsa_parity_snapshot,
        );
        let jack_coordination_summary = format_runtime_jack_coordination_snapshot_multiline(
            &self.observation.jack_coordination_snapshot,
        );
        let external_midi_summary = format_runtime_external_midi_snapshot_multiline(
            &self.observation.external_midi_snapshot,
        );
        let control_surface_summary = format_runtime_control_surface_snapshot_multiline(
            &self.observation.control_surface_snapshot,
        );
        let advanced_hardware_summary = format_runtime_advanced_hardware_snapshot_multiline(
            &self.observation.advanced_hardware_snapshot,
        );
        let execution_topology_summary = format_runtime_execution_topology_summary_multiline(
            &self.observation.execution_topology_summary,
        );
        let metering_summary =
            format_runtime_metering_snapshot_multiline(&self.observation.metering_snapshot);
        let recording_capture = format_runtime_recording_capture_snapshot_multiline(
            &self.observation.recording_capture_snapshot,
        );
        let offline_render_session = format_runtime_offline_render_session_snapshot_multiline(
            &self.observation.offline_render_session_snapshot,
        );
        let deferred_service = self
            .observation
            .last_deferred_service_receipt
            .as_ref()
            .map(|receipt| format!("\nlast_deferred_service=\n{}", receipt.render_multiline()))
            .unwrap_or_default();
        /* let multiline = format!(
            "readiness={:?}\nsample_rate={}\nblock_size={}\nhandshaken={}\nconfigured={}\nrunning={}\nhandshake_count={}\nconfigure_count={}\nstart_count={}\nstop_count={}\nrestart_count={:?}\nlast_client_version={:?}\nlast_stop_reason={:?}\nlast_reconfigure={:?}\nxruns={}\nactive_sandboxes={}\nsafe_mode={}\nnext_block_sequence={}\nsequence_segments={}\nsequence_segment_epochs={:?}\nsequence_first_block={:?}\nsequence_last_block={:?}\nsequence_gaps={}\nsequence_lease_rollovers={}{}{}{}{}{}{}{}{}{}\nengine_graph_id={:?}\nengine_node_count={}\nengine_stateful_nodes={}\nengine_latency_nodes={}\nengine_plugin_backed_nodes={}\nengine_planning_anticipative={}\nengine_inline_realtime_nodes={}\nengine_stateful_realtime_nodes={}\nengine_anticipative_eligible_nodes={}\nengine_phase_count={}\nengine_anticipative_phases={}\nengine_phase_order={:?}\nengine_lane_count={}\nengine_anticipative_lanes={}\nengine_lane_order={:?}\nengine_dispatch_count={}\nengine_dispatch_boundaries={}\nengine_dispatch_order={:?}\nengine_prepared_dispatches={}\nengine_realtime_dispatches={}\nengine_dispatch_handoffs={}{}\nengine_prework_cache_enabled={}\nengine_prework_cache_state={:?}\nengine_prework_service_state={:?}\nengine_prework_service_pressure={:?}\nengine_prework_service_semantic_policy={:?}\nengine_prework_service_active_plugin_sandboxes={}\nengine_prework_service_bound_plugin_sandboxes={}\nengine_prework_service_active_bound_plugin_sandboxes={}\nengine_prework_service_degraded_bound_plugin_sandboxes={}\nengine_prework_service_missing_bound_plugin_sandboxes={}\nengine_prework_service_plugin_gate_active={}\nengine_prework_pending_targets={}\nengine_prework_pending_immediate_targets={}\nengine_prework_pending_near_term_targets={}\nengine_prework_pending_deferred_targets={}\nengine_prework_next_pending_target_block={:?}\nengine_prework_service_cycles={}\nengine_prework_service_prepared_targets={}\nengine_prework_service_pauses={}\nengine_prework_service_resumes={}\nengine_prework_service_starvations={}\nengine_prework_service_throttles={}\nengine_prework_service_yields={}\nengine_last_prework_service_epoch={:?}\nengine_last_prework_service_requested_cycles={}\nengine_last_prework_service_effective_cycles={}\nengine_last_prework_service_cycle_count={}\nengine_last_prework_service_budget={:?}\nengine_last_prework_service_effective_budget={:?}\nengine_last_prework_service_prepared_targets={}\nengine_last_prework_serviced_target_block={:?}\nengine_last_prework_serviced_backlog_class={:?}\nengine_prework_requested_mode={:?}\nengine_prework_mode={:?}\nengine_prework_policy_configured={}\nengine_prework_profile={:?}\nengine_prework_profile_source={:?}\nengine_prework_profile_window_override={:?}\nengine_prework_policy_window_blocks={:?}\nengine_prework_queue_capacity={}\nengine_prework_queue_depth={}\nengine_prework_peak_queue_depth={}\nengine_prework_window_targets={}\nengine_prework_window_blocks={:?}\nengine_prework_freshness_state={:?}\nengine_prework_block_window={}\nengine_prework_remaining_valid_blocks={:?}\nengine_prework_cache_admissions={}\nengine_prework_cache_consumptions={}\nengine_prework_queued_admissions={}\nengine_prework_queued_consumptions={}\nengine_prework_cache_hits={}\nengine_prework_cache_misses={}\nengine_prework_cache_invalidations={}\nengine_last_prework_cache_hit={}\nengine_last_prework_invalidation={:?}\nengine_prework_cache_valid_until={:?}\nengine_prework_cache_valid_until_block={:?}\nengine_last_prework_source_epoch={:?}\nengine_last_prework_source_block={:?}\nengine_last_prework_admission_epoch={:?}\nengine_last_prework_admission_block={:?}\nengine_last_prework_admitted_from_block={:?}\nengine_last_prework_consumption_epoch={:?}\nengine_last_prework_consumption_block={:?}\nengine_last_prework_consumed_from_block={:?}\nengine_planned_nodes={:?}\nengine_stage_count={}\nengine_dynamic_kernel_stages={}\nengine_dynamic_stage_state_model={:?}\nengine_total_latency_samples={}\nengine_max_node_latency_samples={}\nengine_total_tail_samples={}\nengine_max_node_tail_samples={}\nengine_output_tail_samples={}\nengine_max_bus_tail_samples={}\nengine_processed_blocks={}\nengine_last_processing_epoch={:?}\nengine_last_block_sequence={:?}\nengine_last_frame_count={}\nengine_last_channel_count={}\nengine_last_input_peak={:?}\nengine_last_prework_output_peak={:?}\nengine_last_realtime_input_peak={:?}\nengine_last_output_peak={:?}\nengine_last_output_rms={:?}\nengine_last_first_output_sample={:?}\nengine_projection_epoch={:?}\nengine_parameter_epoch={:?}\nengine_context_anticipative={:?}\nengine_transport_playing={:?}\nengine_transport_tempo_bpm={:?}\nengine_timeline_position_samples={:?}{}{}\ntransport_concurrency_steady_limit={}\ntransport_concurrency_recovery_limit={}\ntransport_concurrency_current_attached={}\ntransport_concurrency_peak_attached={}\ntransport_concurrency_current_recovery_overlap={}\ntransport_concurrency_peak_recovery_overlap={}\ntransport_concurrency_current_lingering={}\ntransport_concurrency_peak_lingering={}\ntransport_concurrency_current_detach_requested={}\ntransport_concurrency_current_detach_faulted={}\ntransport_concurrency_active_sessions={:?}\ntransport_concurrency_pending_cleanup_waves={:?}\ntransport_concurrency_last_admitted_sandbox_id={:?}\ntransport_concurrency_last_rejected_sandbox_id={:?}\ntransport_concurrency_last_rejection_reason={:?}\ntransport_fault_boundary={:?}\ntransport_fault_host_broker_events={}\ntransport_fault_sandbox_operation_events={}\ntransport_fault_runtime_dispatch_events={}\ntransport_fault_prepare_events={}\ntransport_fault_dispatch_events={}\ntransport_fault_teardown_events={}\ntransport_fault_control_events={}\ntransport_fault_first_epoch={:?}\ntransport_fault_last_epoch={:?}\ntransport_fault_first_block={:?}\ntransport_fault_last_block={:?}\ntransport_session_boundary={:?}\ntransport_session_state={:?}\ntransport_session_currently_attached={}\ntransport_session_heartbeat_state={:?}\ntransport_session_dispatch_state={:?}\ntransport_session_current_attached_sessions={}\ntransport_session_max_attached_sessions={}\ntransport_session_attach_events={}\ntransport_session_detach_requested_events={}\ntransport_session_detached_events={}\ntransport_session_detach_fault_events={}\ntransport_session_heartbeat_requested_events={}\ntransport_session_heartbeat_responded_events={}\ntransport_session_heartbeat_missed_events={}\ntransport_session_dispatch_requested_events={}\ntransport_session_dispatch_completed_events={}\ntransport_session_dispatch_timed_out_events={}\ntransport_session_first_epoch={:?}\ntransport_session_last_epoch={:?}\ntransport_session_first_block={:?}\ntransport_session_last_block={:?}\ntransport_session_active_sandbox_id={:?}\ntransport_session_active_lease_id={:?}\ntransport_session_active_region_id={:?}\ntransport_session_active_block_sequence={:?}\ntransport_session_active_sessions={:?}\ntransport_session_last_sandbox_id={:?}\ntransport_session_last_lease_id={:?}\ntransport_session_last_region_id={:?}\nevent_stream={}\nsupervision_updates={}\nplugin_faults={}\nrecovery_events={}\nlifecycle_events={}\ntransport_events={}\nheartbeat_events={}\nblock_dispatch_events={}\nlease_rollover_events={}\ninvalidation_events={}\ncompletion_slot_events={}\ntransport_fault_events={}\nbroker_failure_events={}\nsandbox_operation_failure_events={}\nlast_watchdog={}\nlast_fault={}\nlast_recovery={:?}\nlast_lifecycle={:?}\nlast_transport={:?}\nlast_heartbeat={:?}\nlast_dispatch={:?}\nlast_rollover={:?}\nlast_invalidation={:?}\nlast_completion_slot={:?}\nlast_transport_fault={:?}\nlast_broker_failure={:?}\nlast_sandbox_operation_failure={:?}\nrecovery_sequence={:?}\nlifecycle_sequence={:?}\ntransport_sequence={:?}\nheartbeat_sequence={:?}\nblock_dispatch_sequence={:?}\nlease_rollover_sequence={:?}\ninvalidation_sequence={:?}\ncompletion_slot_sequence={:?}\ntransport_fault_sequence={:?}\nbroker_failure_sequence={:?}\nsandbox_operation_failure_sequence={:?}",
            self.observation.readiness,
            self.observation.effective_config.sample_rate.0,
            self.observation.effective_config.block_size,
            self.observation.control_snapshot.handshaken,
            self.observation.control_snapshot.configured,
            self.observation.control_snapshot.running,
            self.observation.control_snapshot.handshake_count,
            self.observation.control_snapshot.configure_count,
            self.observation.control_snapshot.start_count,
            self.observation.control_snapshot.stop_count,
            self.observation.control_snapshot.restart_count,
            self.observation.control_snapshot.last_client_version,
            self.observation.control_snapshot.last_stop_reason,
            self.observation.control_snapshot.last_reconfigure,
            self.observation.diagnostics_snapshot.xruns,
            self.observation.diagnostics_snapshot.active_plugin_sandboxes,
            self.observation.supervision_snapshot.safe_mode_enabled,
            self.observation.timeline_snapshot.next_block_sequence,
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .segment_count(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .segment_epochs(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .first_block_sequence(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .last_block_sequence(),
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .sequence_gaps,
            self.observation
                .timeline_snapshot
                .block_sequence_continuity
                .lease_rollovers,
            automation,
            transport_timeline,
            scheduler_snapshot,
            scheduler_summary,
            block_summary,
            degradation_summary,
            fault_status,
            fault_diagnostic_receipt,
            interruption_summary,
            self.observation.engine_block_snapshot.graph_id,
            self.observation.engine_block_snapshot.node_count,
            self.observation.engine_block_snapshot.stateful_node_count,
            self.observation.engine_block_snapshot.latency_node_count,
            self.observation.engine_block_snapshot.plugin_backed_node_count,
            self.observation
                .engine_block_snapshot
                .anticipative_planning_enabled,
            self.observation.engine_block_snapshot.inline_realtime_node_count,
            self.observation.engine_block_snapshot.stateful_realtime_node_count,
            self.observation
                .engine_block_snapshot
                .anticipative_eligible_node_count,
            self.observation.engine_block_snapshot.phase_count,
            self.observation
                .engine_block_snapshot
                .anticipative_phase_count,
            self.observation.engine_block_snapshot.phase_order,
            self.observation.engine_block_snapshot.lane_count,
            self.observation.engine_block_snapshot.anticipative_lane_count,
            self.observation.engine_block_snapshot.lane_order,
            self.observation.engine_block_snapshot.dispatch_count,
            self.observation
                .engine_block_snapshot
                .dispatch_boundary_count,
            self.observation.engine_block_snapshot.dispatch_order,
            self.observation.engine_block_snapshot.prepared_dispatch_count,
            self.observation.engine_block_snapshot.realtime_dispatch_count,
            self.observation.engine_block_snapshot.dispatch_handoff_count,
            scheduler_topology,
            self.observation.engine_block_snapshot.prework_cache_enabled,
            self.observation.engine_block_snapshot.prework_cache_state,
            self.observation.engine_block_snapshot.prework_service_state,
            self.observation.engine_block_snapshot.prework_service_pressure,
            self.observation
                .engine_block_snapshot
                .prework_service_semantic_policy,
            self.observation
                .engine_block_snapshot
                .prework_service_active_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_active_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_degraded_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_missing_bound_plugin_sandboxes,
            self.observation
                .engine_block_snapshot
                .prework_service_plugin_gate_active,
            self.observation
                .engine_block_snapshot
                .prework_pending_target_count,
            self.observation
                .engine_block_snapshot
                .prework_pending_immediate_target_count,
            self.observation
                .engine_block_snapshot
                .prework_pending_near_term_target_count,
            self.observation
                .engine_block_snapshot
                .prework_pending_deferred_target_count,
            self.observation
                .engine_block_snapshot
                .prework_next_pending_target_block_sequence,
            self.observation.engine_block_snapshot.prework_service_cycle_count,
            self.observation
                .engine_block_snapshot
                .prework_service_prepared_targets,
            self.observation.engine_block_snapshot.prework_service_pause_count,
            self.observation.engine_block_snapshot.prework_service_resume_count,
            self.observation
                .engine_block_snapshot
                .prework_service_starvation_count,
            self.observation
                .engine_block_snapshot
                .prework_service_throttle_count,
            self.observation.engine_block_snapshot.prework_service_yield_count,
            self.observation
                .engine_block_snapshot
                .last_prework_service_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_service_requested_cycles,
            self.observation
                .engine_block_snapshot
                .last_prework_service_effective_cycles,
            self.observation
                .engine_block_snapshot
                .last_prework_service_cycle_count,
            self.observation
                .engine_block_snapshot
                .last_prework_service_budget_per_cycle,
            self.observation
                .engine_block_snapshot
                .last_prework_service_effective_budget_per_cycle,
            self.observation
                .engine_block_snapshot
                .last_prework_service_prepared_targets,
            self.observation
                .engine_block_snapshot
                .last_prework_serviced_target_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_serviced_backlog_class,
            self.observation
                .engine_block_snapshot
                .prework_forecast_requested_mode,
            self.observation.engine_block_snapshot.prework_forecast_mode,
            self.observation
                .engine_block_snapshot
                .prework_forecast_policy_configured,
            self.observation
                .engine_block_snapshot
                .prework_forecast_profile,
            self.observation
                .engine_block_snapshot
                .prework_forecast_profile_source,
            self.observation
                .engine_block_snapshot
                .prework_forecast_profile_target_window_override,
            self.observation
                .engine_block_snapshot
                .prework_forecast_policy_target_window_blocks,
            self.observation.engine_block_snapshot.prework_cache_queue_capacity,
            self.observation.engine_block_snapshot.prework_cache_queue_depth,
            self.observation.engine_block_snapshot.prework_cache_peak_queue_depth,
            self.observation.engine_block_snapshot.prework_cache_window_target_count,
            self.observation
                .engine_block_snapshot
                .prework_cache_window_target_block_sequences,
            self.observation.engine_block_snapshot.prework_cache_freshness_state,
            self.observation
                .engine_block_snapshot
                .prework_cache_block_freshness_window,
            self.observation
                .engine_block_snapshot
                .prework_cache_remaining_valid_blocks,
            self.observation.engine_block_snapshot.prework_cache_admissions,
            self.observation.engine_block_snapshot.prework_cache_consumptions,
            self.observation
                .engine_block_snapshot
                .prework_cache_queued_admissions,
            self.observation
                .engine_block_snapshot
                .prework_cache_queued_consumptions,
            self.observation.engine_block_snapshot.prework_cache_hits,
            self.observation.engine_block_snapshot.prework_cache_misses,
            self.observation
                .engine_block_snapshot
                .prework_cache_invalidation_count,
            self.observation.engine_block_snapshot.last_prework_cache_hit,
            self.observation
                .engine_block_snapshot
                .last_prework_invalidation_reason,
            self.observation
                .engine_block_snapshot
                .prework_cache_valid_until_processing_epoch,
            self.observation
                .engine_block_snapshot
                .prework_cache_valid_until_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_source_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_source_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_admission_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_admission_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_admitted_from_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_consumption_processing_epoch,
            self.observation
                .engine_block_snapshot
                .last_prework_consumption_block_sequence,
            self.observation
                .engine_block_snapshot
                .last_prework_consumed_from_block_sequence,
            self.observation.engine_block_snapshot.planned_nodes,
            self.observation.engine_block_snapshot.stage_count,
            self.observation.engine_block_snapshot.dynamic_kernel_stage_count,
            self.observation.engine_block_snapshot.dynamic_stage_state_model,
            self.observation.engine_block_snapshot.total_latency_samples,
            self.observation.engine_block_snapshot.max_node_latency_samples,
            self.observation.engine_block_snapshot.total_tail_samples,
            self.observation.engine_block_snapshot.max_node_tail_samples,
            self.observation.engine_block_snapshot.output_tail_samples,
            self.observation.engine_block_snapshot.max_bus_tail_samples,
            self.observation.engine_block_snapshot.processed_blocks,
            self.observation.engine_block_snapshot.last_processing_epoch,
            self.observation.engine_block_snapshot.last_block_sequence,
            self.observation.engine_block_snapshot.last_frame_count,
            self.observation.engine_block_snapshot.last_channel_count,
            self.observation.engine_block_snapshot.last_input_peak,
            self.observation
                .engine_block_snapshot
                .last_prework_output_peak,
            self.observation
                .engine_block_snapshot
                .last_realtime_input_peak,
            self.observation.engine_block_snapshot.last_output_peak,
            self.observation.engine_block_snapshot.last_output_rms,
            self.observation.engine_block_snapshot.last_first_output_sample,
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.projection_epoch),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.parameter_epoch),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.anticipative_enabled),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_playing),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.transport_tempo_bpm),
            self.observation
                .engine_block_snapshot
                .last_execution_context
                .as_ref()
                .map(|context| context.timeline_position_samples),
            engine_transport,
            scheduler_topology,
            self.observation
                .transport_concurrency_snapshot
                .steady_session_limit,
            self.observation
                .transport_concurrency_snapshot
                .recovery_session_limit,
            self.observation
                .transport_concurrency_snapshot
                .current_attached_sessions,
            self.observation.transport_concurrency_snapshot.peak_attached_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_recovery_overlap_sessions,
            self.observation
                .transport_concurrency_snapshot
                .peak_recovery_overlap_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_lingering_sessions,
            self.observation
                .transport_concurrency_snapshot
                .peak_lingering_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_detach_requested_sessions,
            self.observation
                .transport_concurrency_snapshot
                .current_detach_faulted_sessions,
            self.observation.transport_concurrency_snapshot.active_sessions,
            self.observation
                .transport_concurrency_snapshot
                .pending_cleanup_waves,
            self.observation
                .transport_concurrency_snapshot
                .last_admitted_sandbox_id,
            self.observation
                .transport_concurrency_snapshot
                .last_rejected_sandbox_id,
            self.observation
                .transport_concurrency_snapshot
                .last_rejection_reason,
            self.observation.transport_fault_summary.boundary_mode,
            self.observation.transport_fault_summary.host_broker_events,
            self.observation.transport_fault_summary.sandbox_operation_events,
            self.observation.transport_fault_summary.runtime_dispatch_events,
            self.observation.transport_fault_summary.prepare_events,
            self.observation.transport_fault_summary.dispatch_events,
            self.observation.transport_fault_summary.teardown_events,
            self.observation.transport_fault_summary.control_events,
            self.observation.transport_fault_summary.first_processing_epoch,
            self.observation.transport_fault_summary.last_processing_epoch,
            self.observation.transport_fault_summary.first_block_sequence,
            self.observation.transport_fault_summary.last_block_sequence,
            self.observation.transport_session_summary.boundary_mode,
            self.observation.transport_session_summary.current_state,
            self.observation.transport_session_summary.currently_attached,
            self.observation.transport_session_summary.heartbeat_freshness,
            self.observation.transport_session_summary.dispatch_state,
            self.observation.transport_session_summary.current_attached_session_count,
            self.observation.transport_session_summary.max_concurrent_attached_sessions,
            self.observation.transport_session_summary.attach_events,
            self.observation.transport_session_summary.detach_requested_events,
            self.observation.transport_session_summary.detached_events,
            self.observation.transport_session_summary.detach_fault_events,
            self.observation.transport_session_summary.heartbeat_requested_events,
            self.observation.transport_session_summary.heartbeat_responded_events,
            self.observation.transport_session_summary.heartbeat_missed_events,
            self.observation.transport_session_summary.dispatch_requested_events,
            self.observation.transport_session_summary.dispatch_completed_events,
            self.observation.transport_session_summary.dispatch_timed_out_events,
            self.observation.transport_session_summary.first_processing_epoch,
            self.observation.transport_session_summary.last_processing_epoch,
            self.observation.transport_session_summary.first_block_sequence,
            self.observation.transport_session_summary.last_block_sequence,
            self.observation.transport_session_summary.active_sandbox_id,
            self.observation.transport_session_summary.active_lease_id,
            self.observation.transport_session_summary.active_region_id,
            self.observation.transport_session_summary.active_block_sequence,
            self.observation.transport_session_summary.active_sessions,
            self.observation.transport_session_summary.last_sandbox_id,
            self.observation.transport_session_summary.last_lease_id,
            self.observation.transport_session_summary.last_region_id,
            self.event_count(),
            self.supervision_update_count(),
            self.plugin_fault_count(),
            self.recovery_event_count(),
            self.lifecycle_event_count(),
            self.transport_event_count(),
            self.heartbeat_event_count(),
            self.block_dispatch_event_count(),
            self.lease_rollover_event_count(),
            self.invalidation_event_count(),
            self.completion_slot_event_count(),
            self.transport_fault_event_count(),
            self.broker_failure_event_count(),
            self.sandbox_operation_failure_event_count(),
            self.last_watchdog_trigger()
                .map(|trigger| format!("{trigger:?}"))
                .unwrap_or_else(|| "none".into()),
            self.observation
                .observation
                .plugin_faults
                .last()
                .map(|fault| format!("{}:{:?}", fault.sandbox_id, fault.kind))
                .unwrap_or_else(|| "none".into()),
            self.observation.observation.last_recovery_event(),
            self.observation.observation.last_lifecycle_event(),
            self.observation.observation.last_transport_event(),
            self.observation.observation.last_heartbeat_event(),
            self.observation.observation.last_block_dispatch_event(),
            self.observation.observation.last_lease_rollover_event(),
            self.observation.observation.last_invalidation_event(),
            self.observation.observation.last_completion_slot_event(),
            self.observation.observation.last_transport_fault_event(),
            self.observation.observation.last_broker_failure_event(),
            self.observation.observation.last_sandbox_operation_failure_event(),
            self.observation.observation.recovery_events,
            self.observation.observation.lifecycle_events,
            self.observation.observation.transport_events,
            self.observation.observation.heartbeat_events,
            self.observation.observation.block_dispatch_events,
            self.observation.observation.lease_rollover_events,
            self.observation.observation.invalidation_events,
            self.observation.observation.completion_slot_events,
            self.observation.observation.transport_fault_events,
            self.observation.observation.broker_failure_events,
            self.observation.observation.sandbox_operation_failure_events,
        ); */
        let multiline = self.observation.render_compact().replace(' ', "\n");
        format!(
            "{multiline}{tempo_map}{warp}{clip_processing}{stretch_engine}{marker_analysis}{transform_artifact}{media_pipeline}{media_service}{media_library}{recording_capture}{offline_render_session}{plugin_discovery}{plugin_lifecycle}{lv2_extension}{plugin_pin_matrix}{plugin_chain}{device_supervision_summary}{external_io_summary}{linux_backend_session_summary}{pipewire_alsa_parity_summary}{jack_coordination_summary}{external_midi_summary}{control_surface_summary}{advanced_hardware_summary}{execution_topology_summary}{metering_summary}{deferred_service}"
        )
    }

    pub fn render_json(&self) -> String {
        render_runtime_supervisor_report_json(self)
    }
}

impl RuntimeObservationReport {
    pub fn render_json(&self) -> String {
        RuntimeSupervisorReport {
            observation: self.clone(),
            events: Vec::new(),
        }
        .render_json()
    }
}

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
