//! Typed runtime-host interfaces for embedded Signal assemblies.
mod clip_analysis_family;
mod engine_block_surface_family;
mod engine_lifecycle_family;
mod event_recorder_family;
mod event_transport_family;
mod execution_metering_surface_family;
mod external_io_family;
mod fault_interruption_family;
mod host_observation_family;
#[path = "interfaces_host_io_family.rs"]
mod interfaces_host_io_family;
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
pub use interfaces_host_io_family::*;
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
    format_runtime_device_supervision_snapshot_compact,
    format_runtime_device_supervision_snapshot_multiline,
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

mod lifecycle_core_family;
pub use lifecycle_core_family::*;
mod plugin_pin_matrix_family;
pub(crate) use plugin_pin_matrix_family::runtime_bus_intents_for_topology_role;
pub(crate) use plugin_pin_matrix_family::runtime_bus_role_for_endpoint;
pub use plugin_pin_matrix_family::*;

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

mod runtime_api_traits_family;
pub use runtime_api_traits_family::*;

#[cfg(test)]
#[path = "interfaces_tests.rs"]
mod tests;
