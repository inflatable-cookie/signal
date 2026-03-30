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

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
            let auxiliary_path = runtime_auxiliary_path_for_connection(source, &target);
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
