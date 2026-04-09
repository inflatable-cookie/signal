use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeLv2PreparedNegotiationRecord {
    pub worker_posture: RuntimeLv2WorkerPosture,
    pub urid_negotiation_posture: RuntimeLv2UridNegotiationPosture,
    pub patch_exchange_posture: RuntimeLv2PatchExchangePosture,
    pub extension_negotiation_state: RuntimeLv2ExtensionNegotiationState,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginLifecycleState {
    Booting,
    Ready,
    Degraded,
    Faulted,
    Restarting,
    Quarantined,
    #[default]
    Stopped,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginSandboxSnapshot {
    pub sandbox_id: String,
    pub sandbox_group_key: String,
    pub plugin_type_id: Option<String>,
    pub plugin_format: Option<PluginFormat>,
    pub instance_id: Option<String>,
    pub preset_descriptor: Option<RuntimePluginPresetDescriptor>,
    pub ara_context: Option<RuntimePluginAraContextSnapshot>,
    pub placement_outcome: RuntimePluginIsolationOutcome,
    pub placement_rule_id: Option<String>,
    pub shared_boundary_member_count: usize,
    pub continuity_class: RuntimeInterruptionClass,
    pub rebindable: bool,
    pub state: RuntimePluginLifecycleState,
    pub lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    pub transport_stage: Option<PluginSandboxTransportStage>,
    pub active: bool,
    pub active_transport: bool,
    pub restart_count: u32,
    pub recovery_count: u32,
    pub fault_count: u32,
    pub last_fault_kind: Option<PluginFaultKind>,
    pub last_fault_detail: Option<String>,
    pub last_restart_intent: Option<RecoveryRestartIntent>,
    pub last_stop_reason: Option<StopReason>,
    pub last_processing_epoch: Option<u64>,
    pub readiness_state: Option<String>,
    pub degraded_reasons: Vec<String>,
    pub active_lease_id: Option<String>,
    pub active_region_id: Option<String>,
    pub lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginLifecycleSnapshot {
    pub sandbox_count: usize,
    pub active_sandbox_count: u32,
    pub shared_sandbox_count: usize,
    pub isolated_sandbox_count: usize,
    pub ready_sandbox_count: usize,
    pub booting_sandbox_count: usize,
    pub degraded_sandbox_count: usize,
    pub faulted_sandbox_count: usize,
    pub restarting_sandbox_count: usize,
    pub quarantined_sandbox_count: usize,
    pub stopped_sandbox_count: usize,
    pub rebindable_sandbox_count: usize,
    pub terminal_sandbox_count: usize,
    pub parity_coverage: Vec<RuntimePluginFormatParityRecord>,
    pub sandboxes: Vec<RuntimePluginSandboxSnapshot>,
    pub summary: String,
}
