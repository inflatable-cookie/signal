use super::*;

/// Full snapshot of a single plugin chain stage: sandbox binding, I/O layout,
/// recall state, latency, and compensation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginChainStageSnapshot {
    /// Graph node identifier for this stage.
    pub node_id: String,
    /// Index of this stage within the chain.
    pub stage_index: usize,
    /// Sandbox ID bound to this stage, if any.
    pub sandbox_id: Option<String>,
    /// Group key for shared-sandbox stages.
    pub sandbox_group_key: Option<String>,
    /// Track lane ID, if this stage is on a track lane chain.
    pub track_lane_id: Option<String>,
    /// Bus group ID, if this stage is on a bus chain.
    pub bus_group_id: Option<String>,
    /// Console group ID, if this stage is on a console chain.
    pub console_group_id: Option<String>,
    /// Send/return ID, if this stage is on a send/return chain.
    pub send_return_id: Option<String>,
    /// Sandbox isolation outcome for this stage.
    pub placement_outcome: RuntimePluginIsolationOutcome,
    /// Placement rule ID that governed the isolation outcome.
    pub placement_rule_id: Option<String>,
    /// Number of stages sharing the same sandbox boundary.
    pub shared_boundary_member_count: usize,
    /// Interruption continuity class for this stage.
    pub continuity_class: RuntimeInterruptionClass,
    /// Whether this stage can be rebound to a different sandbox.
    pub rebindable: bool,
    /// Multichannel I/O layout for this stage.
    pub io_layout: RuntimeMultichannelIoSummary,
    /// Complex I/O topology summary for this stage.
    pub complex_io_summary: RuntimePluginComplexIoSummary,
    /// Secondary (sidechain) input routing, if applicable.
    pub secondary_input: Option<RuntimeSecondaryInputRouteSummary>,
    /// Spatial execution configuration for this stage, if applicable.
    pub spatial_execution: Option<RuntimeSpatialExecutionSummary>,
    /// Lifecycle state of the bound sandbox.
    pub lifecycle_state: Option<RuntimePluginLifecycleState>,
    /// Lifecycle stage of the bound sandbox.
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    /// Transport stage of the bound sandbox.
    pub transport_stage: Option<PluginSandboxTransportStage>,
    /// Recall readiness state for this stage.
    pub recall_state: RuntimePluginRecallState,
    /// Full recall snapshot for this stage.
    pub recall: RuntimePluginRecallSnapshot,
    /// Latency compensation state for this stage.
    pub compensation_state: RuntimePluginCompensationState,
    /// Planned latency contribution of this stage in samples.
    pub planned_latency_samples: u32,
    /// Realized latency reported by the plugin, if available.
    pub realized_latency_samples: Option<u32>,
    /// Tail length reported by the plugin, if available.
    pub tail_samples: Option<u32>,
    /// Whether this stage is currently bypassed.
    pub bypassed: bool,
    /// Whether this stage has an active transport session.
    pub active_transport: bool,
    /// Reasons this stage is currently degraded.
    pub degraded_reasons: Vec<String>,
}

/// Summary of all stages in one plugin execution chain, including latency and
/// compensation counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginExecutionChainSummary {
    /// Stable chain identifier.
    pub chain_id: String,
    /// Track lane ID this chain belongs to, if applicable.
    pub track_lane_id: Option<String>,
    /// Bus group ID this chain belongs to, if applicable.
    pub bus_group_id: Option<String>,
    /// Console group ID this chain belongs to, if applicable.
    pub console_group_id: Option<String>,
    /// Send/return ID this chain belongs to, if applicable.
    pub send_return_id: Option<String>,
    /// Total number of stages in this chain.
    pub stage_count: usize,
    /// Number of stages running in shared-process sandboxes.
    pub shared_sandbox_stage_count: usize,
    /// Number of stages running in isolated-process sandboxes.
    pub isolated_sandbox_stage_count: usize,
    /// Number of stages running in-process.
    pub in_process_stage_count: usize,
    /// Number of stages in the pending-render compensation state.
    pub pending_render_stage_count: usize,
    /// Number of stages settling their latency compensation.
    pub settling_stage_count: usize,
    /// Number of stages with active latency compensation.
    pub compensated_stage_count: usize,
    /// Number of stages in a degraded state.
    pub degraded_stage_count: usize,
    /// Number of bypassed stages.
    pub bypassed_stage_count: usize,
    /// Number of stages with a missing sandbox binding.
    pub missing_binding_stage_count: usize,
    /// Number of stages eligible for rebinding.
    pub rebindable_stage_count: usize,
    /// Number of stages in a terminal (unrecoverable) state.
    pub terminal_stage_count: usize,
    /// Sum of planned latency contributions across all stages.
    pub total_planned_latency_samples: u32,
    /// Sum of realized latency contributions across all stages.
    pub total_realized_latency_samples: u32,
    /// Sum of tail length contributions across all stages.
    pub total_tail_samples: u32,
    /// Per-stage snapshots.
    pub stages: Vec<RuntimePluginChainStageSnapshot>,
}

/// Aggregate snapshot of all plugin execution chains and their combined stage counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginChainSnapshot {
    /// Total number of plugin chains.
    pub chain_count: usize,
    /// Total number of stages across all chains.
    pub stage_count: usize,
    /// Number of stages running in shared-process sandboxes.
    pub shared_sandbox_stage_count: usize,
    /// Number of stages running in isolated-process sandboxes.
    pub isolated_sandbox_stage_count: usize,
    /// Number of stages running in-process.
    pub in_process_stage_count: usize,
    /// Number of stages in the pending-render compensation state.
    pub pending_render_stage_count: usize,
    /// Number of stages settling their latency compensation.
    pub settling_stage_count: usize,
    /// Number of stages with active latency compensation.
    pub compensated_stage_count: usize,
    /// Number of stages in a degraded state.
    pub degraded_stage_count: usize,
    /// Number of bypassed stages.
    pub bypassed_stage_count: usize,
    /// Number of stages with a missing sandbox binding.
    pub missing_binding_stage_count: usize,
    /// Number of stages eligible for rebinding.
    pub rebindable_stage_count: usize,
    /// Number of stages in a terminal (unrecoverable) state.
    pub terminal_stage_count: usize,
    /// Sum of planned latency contributions across all stages.
    pub total_planned_latency_samples: u32,
    /// Sum of realized latency contributions across all stages.
    pub total_realized_latency_samples: u32,
    /// Sum of tail length contributions across all stages.
    pub total_tail_samples: u32,
    /// Per-chain execution summaries.
    pub chains: Vec<RuntimePluginExecutionChainSummary>,
}
