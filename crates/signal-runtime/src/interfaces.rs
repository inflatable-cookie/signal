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
        let fault_status = RuntimeFaultStatusSnapshot::capture(RuntimeFaultStatusCaptureInput {
            readiness: readiness.clone(),
            control_snapshot: &control_snapshot,
            diagnostics_snapshot: &diagnostics_snapshot,
            supervision_snapshot: &supervision_snapshot,
            engine_block_snapshot: &engine_block_snapshot,
            transport_concurrency_snapshot: &transport_concurrency_snapshot,
            plugin_lifecycle_snapshot: &plugin_lifecycle_snapshot,
            device_loss_active: false,
            device_loss_count: 0,
        });
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
        let tempo_map = if self.observation.tempo_map_snapshot.segment_count > 0 {
            format_runtime_tempo_map_snapshot_multiline(&self.observation.tempo_map_snapshot)
        } else {
            String::new()
        };
        let warp = if self.observation.warp_pipeline_snapshot.clip_count > 0 {
            format_runtime_warp_pipeline_snapshot_multiline(
                &self.observation.warp_pipeline_snapshot,
            )
        } else {
            String::new()
        };
        let clip_processing = if self
            .observation
            .clip_processing_pipeline_snapshot
            .clip_count
            > 0
        {
            format_runtime_clip_processing_pipeline_snapshot_multiline(
                &self.observation.clip_processing_pipeline_snapshot,
            )
        } else {
            String::new()
        };
        let stretch_engine = if self.observation.stretch_engine_snapshot.clip_count > 0 {
            format_runtime_stretch_engine_snapshot_multiline(
                &self.observation.stretch_engine_snapshot,
            )
        } else {
            String::new()
        };
        let marker_analysis = if self.observation.marker_analysis_snapshot.clip_count > 0 {
            format_runtime_marker_analysis_snapshot_multiline(
                &self.observation.marker_analysis_snapshot,
            )
        } else {
            String::new()
        };
        let transform_artifact = if self.observation.transform_artifact_snapshot.clip_count > 0 {
            format_runtime_transform_artifact_snapshot_multiline(
                &self.observation.transform_artifact_snapshot,
            )
        } else {
            String::new()
        };
        let media_pipeline = if self.observation.media_pipeline_snapshot.asset_count > 0 {
            format_runtime_media_pipeline_snapshot_multiline(
                &self.observation.media_pipeline_snapshot,
            )
        } else {
            String::new()
        };
        let media_service = if self.observation.media_service_snapshot.indexed_asset_count > 0
            || self.observation.media_service_snapshot.invalidation_active
            || matches!(
                self.observation.media_service_snapshot.preview_state,
                RuntimeMediaPreviewState::Previewing | RuntimeMediaPreviewState::Invalidated
            ) {
            format_runtime_media_service_snapshot_multiline(
                &self.observation.media_service_snapshot,
            )
        } else {
            String::new()
        };
        let media_library = if self.observation.media_library_snapshot.indexed_asset_count > 0 {
            format_runtime_media_library_service_snapshot_multiline(
                &self.observation.media_library_snapshot,
            )
        } else {
            String::new()
        };
        let plugin_discovery = if self.observation.plugin_discovery_snapshot.scan_count > 0 {
            format_runtime_plugin_discovery_snapshot_multiline(
                &self.observation.plugin_discovery_snapshot,
            )
        } else {
            String::new()
        };
        let plugin_lifecycle = if self.observation.plugin_lifecycle_snapshot.sandbox_count > 0 {
            format_runtime_plugin_lifecycle_snapshot_multiline(
                &self.observation.plugin_lifecycle_snapshot,
            )
        } else {
            String::new()
        };
        let lv2_extension = if self.observation.lv2_extension_snapshot.plugin_type_count > 0 {
            format_runtime_lv2_extension_snapshot_multiline(
                &self.observation.lv2_extension_snapshot,
            )
        } else {
            String::new()
        };
        let plugin_pin_matrix = if self
            .observation
            .plugin_pin_matrix_snapshot
            .plugin_type_count
            > 0
        {
            format_runtime_plugin_pin_matrix_snapshot_multiline(
                &self.observation.plugin_pin_matrix_snapshot,
            )
        } else {
            String::new()
        };
        let plugin_chain = if self.observation.plugin_chain_snapshot.chain_count > 0 {
            format_runtime_plugin_chain_snapshot_multiline(&self.observation.plugin_chain_snapshot)
        } else {
            String::new()
        };
        let _automation = if self.observation.automation_snapshot.parameter_id != 0
            || self.observation.automation_snapshot.lane_count > 0
            || self
                .observation
                .automation_snapshot
                .last_batch_epoch
                .is_some()
        {
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
        } else {
            String::new()
        };
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
