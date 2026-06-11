use super::*;

/// Outcome of LV2 extension negotiation (worker, URID, patch, extensions)
/// completed during sandbox preparation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2PreparedNegotiationRecord {
    /// Negotiated worker thread posture for this LV2 instance.
    pub worker_posture: RuntimeLv2WorkerPosture,
    /// Negotiated URID map/unmap posture.
    pub urid_negotiation_posture: RuntimeLv2UridNegotiationPosture,
    /// Negotiated LV2 patch exchange posture.
    pub patch_exchange_posture: RuntimeLv2PatchExchangePosture,
    /// Overall extension negotiation state.
    pub extension_negotiation_state: RuntimeLv2ExtensionNegotiationState,
}

/// Lifecycle state of a plugin sandbox from the runtime's perspective.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginLifecycleState {
    /// Sandbox is starting up but not yet ready.
    Booting,
    /// Sandbox is running and ready to process audio.
    Ready,
    /// Sandbox is running but in a degraded state.
    Degraded,
    /// Sandbox has faulted and is not processing audio.
    Faulted,
    /// Sandbox is undergoing a restart.
    Restarting,
    /// Sandbox has been quarantined after repeated faults.
    Quarantined,
    /// Sandbox is not running.
    #[default]
    Stopped,
}

/// Full lifecycle snapshot for a single plugin sandbox.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginSandboxSnapshot {
    /// Unique identifier for this sandbox instance.
    pub sandbox_id: String,
    /// Group key used for shared-sandbox placement.
    pub sandbox_group_key: String,
    /// Plugin type identifier, if known.
    pub plugin_type_id: Option<String>,
    /// Plugin format (VST3, LV2, etc.), if known.
    pub plugin_format: Option<PluginFormat>,
    /// Plugin instance identifier within the sandbox, if assigned.
    pub instance_id: Option<String>,
    /// Isolation outcome chosen by the placement policy.
    pub placement_outcome: RuntimePluginIsolationOutcome,
    /// ID of the placement rule that determined the isolation outcome.
    pub placement_rule_id: Option<String>,
    /// Number of other sandboxes sharing the same sandbox boundary.
    pub shared_boundary_member_count: usize,
    /// Interruption class this sandbox's faults contribute to.
    pub continuity_class: RuntimeInterruptionClass,
    /// Whether this sandbox can be rebound without a full restart.
    pub rebindable: bool,
    /// Current lifecycle state of this sandbox.
    pub state: RuntimePluginLifecycleState,
    /// Fine-grained lifecycle stage, if available.
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    /// Transport stage, if the sandbox is currently transporting audio.
    pub transport_stage: Option<PluginSandboxTransportStage>,
    /// Whether this sandbox is currently active (processing audio).
    pub active: bool,
    /// Whether this sandbox has an active transport session.
    pub active_transport: bool,
    /// Number of times this sandbox has been restarted.
    pub restart_count: u32,
    /// Number of recovery cycles this sandbox has undergone.
    pub recovery_count: u32,
    /// Total number of faults recorded for this sandbox.
    pub fault_count: u32,
    /// Kind of the most recent fault, if any.
    pub last_fault_kind: Option<PluginFaultKind>,
    /// Detail string from the most recent fault, if any.
    pub last_fault_detail: Option<String>,
    /// Intent behind the most recent restart, if any.
    pub last_restart_intent: Option<RecoveryRestartIntent>,
    /// Reason the sandbox was last stopped, if applicable.
    pub last_stop_reason: Option<StopReason>,
    /// Processing epoch at which this sandbox last processed audio.
    pub last_processing_epoch: Option<u64>,
    /// Human-readable readiness state string, if available.
    pub readiness_state: Option<String>,
    /// List of reasons the sandbox is currently degraded.
    pub degraded_reasons: Vec<String>,
    /// Lease ID of the active transport session, if any.
    pub active_lease_id: Option<String>,
    /// Region ID of the active transport session, if any.
    pub active_region_id: Option<String>,
    /// LV2 extension negotiation record from sandbox preparation, if applicable.
    pub lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
}

/// Aggregate lifecycle snapshot for all plugin sandboxes managed by the runtime.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginLifecycleSnapshot {
    /// Total number of sandboxes.
    pub sandbox_count: usize,
    /// Number of currently active sandboxes.
    pub active_sandbox_count: u32,
    /// Number of sandboxes in shared isolation.
    pub shared_sandbox_count: usize,
    /// Number of sandboxes in full isolation.
    pub isolated_sandbox_count: usize,
    /// Number of sandboxes in the `Ready` state.
    pub ready_sandbox_count: usize,
    /// Number of sandboxes in the `Booting` state.
    pub booting_sandbox_count: usize,
    /// Number of sandboxes in the `Degraded` state.
    pub degraded_sandbox_count: usize,
    /// Number of sandboxes in the `Faulted` state.
    pub faulted_sandbox_count: usize,
    /// Number of sandboxes in the `Restarting` state.
    pub restarting_sandbox_count: usize,
    /// Number of sandboxes in the `Quarantined` state.
    pub quarantined_sandbox_count: usize,
    /// Number of sandboxes in the `Stopped` state.
    pub stopped_sandbox_count: usize,
    /// Number of sandboxes that are rebindable.
    pub rebindable_sandbox_count: usize,
    /// Number of sandboxes in a terminal (non-recoverable) state.
    pub terminal_sandbox_count: usize,
    /// Per-format parity coverage records for all sandboxes.
    pub parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    /// Snapshot for each individual sandbox.
    pub sandboxes: Vec<RuntimePluginSandboxSnapshot>,
}
