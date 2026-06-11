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
#[path = "interfaces_offline_contract_family.rs"]
mod interfaces_offline_contract_family;
mod media_clip_family;
mod media_surface_family;
mod observation_receipt_family;
mod offline_render_family;
mod plugin_chain_render_family;
mod plugin_discovery_family;
mod plugin_recall_family;
mod plugin_runtime_surface_family;
mod preview_transform_family;
mod receipt_surface_family;
mod scheduler_surface_family;
pub use engine_lifecycle_family::*;
pub use event_recorder_family::*;
pub use event_transport_family::*;
pub use external_io_family::*;
pub use fault_interruption_family::*;
pub use host_observation_family::*;
pub use interfaces_host_io_family::*;
pub use media_clip_family::*;
pub use offline_render_family::*;
pub use plugin_discovery_family::*;
pub use plugin_recall_family::*;
pub use preview_transform_family::*;
pub use receipt_surface_family::*;

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
    GraphChannelAdaptationMode, GraphExecutionContext,
    GraphExecutionLane, GraphNodeExecutionClass, GraphNodePlanningGroup, GraphNodeResetPolicy,
    GraphNodeSilencePolicy, GraphNodeTopologyRole, GraphStageSpec,
};
use signal_hardware::{BackendHealth, BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{
    CompletionState, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginStateContract,
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
