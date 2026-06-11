//! Typed runtime-host interfaces for embedded Signal assemblies.
mod channel_bus_family;
mod engine_lifecycle_family;
mod event_recorder_family;
mod execution_topology_family;
mod external_io_family;
mod fault_interruption_family;
mod fault_supervision_family;
mod graph_projection_family;
mod host_observation_family;
#[path = "interfaces_host_io_family.rs"]
mod interfaces_host_io_family;
mod lifecycle_core_family;
mod media_clip_family;
mod observation_diagnostics_family;
mod observation_report_family;
mod plugin_discovery_family;
mod plugin_pin_matrix_family;
mod plugin_recall_family;
mod runtime_api_traits_family;
mod runtime_control_family;
mod sandbox_records_family;
mod supervisor_report_family;
mod transport_fault_family;

pub use channel_bus_family::*;
pub use engine_lifecycle_family::*;
pub use event_recorder_family::*;
pub use execution_topology_family::*;
pub use external_io_family::*;
pub use fault_interruption_family::*;
pub use fault_supervision_family::*;
pub use graph_projection_family::*;
pub use host_observation_family::*;
pub use interfaces_host_io_family::*;
pub use lifecycle_core_family::*;
pub use media_clip_family::*;
pub use observation_diagnostics_family::*;
pub use observation_report_family::*;
pub use plugin_discovery_family::*;
pub use plugin_pin_matrix_family::*;
pub use plugin_recall_family::*;
pub use runtime_api_traits_family::*;
pub use runtime_control_family::*;
pub use sandbox_records_family::*;
pub use supervisor_report_family::*;
pub use transport_fault_family::*;

use std::sync::{Arc, Mutex};

use signal_graph::{
    GraphChannelAdaptationMode, GraphExecutionLane, GraphNodeExecutionClass,
    GraphNodePlanningGroup, GraphNodeResetPolicy, GraphNodeSilencePolicy, GraphNodeTopologyRole,
    GraphStageSpec,
};
use signal_hardware::{BackendHealth, BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{
    CompletionState, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
    PluginProcessingContract, PluginStateContract,
};
use signal_primitives::{ChannelLayout, SampleRate};

fn runtime_lane_for_group(group: GraphNodePlanningGroup) -> GraphExecutionLane {
    match group {
        GraphNodePlanningGroup::InlineRealtime | GraphNodePlanningGroup::StatefulRealtime => {
            GraphExecutionLane::Realtime
        }
        GraphNodePlanningGroup::AnticipativeEligible => GraphExecutionLane::Anticipative,
    }
}
