use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RuntimePluginLifecyclePolicy {
    pub(super) quarantine_after_faults: u32,
}

impl Default for RuntimePluginLifecyclePolicy {
    fn default() -> Self {
        Self {
            quarantine_after_faults: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct RuntimePluginSandboxStateModel {
    pub(crate) sandbox_id: String,
    pub(crate) plugin_type_id: Option<String>,
    pub(crate) plugin_format: Option<PluginFormat>,
    pub(super) instance_id: Option<String>,
    pub(super) state: RuntimePluginLifecycleState,
    pub(super) lifecycle_stage: Option<PluginSandboxLifecycleStage>,
    pub(super) transport_stage: Option<PluginSandboxTransportStage>,
    pub(super) active: bool,
    pub(super) active_transport: bool,
    pub(super) restart_count: u32,
    pub(super) recovery_count: u32,
    pub(super) fault_count: u32,
    pub(super) last_fault_kind: Option<PluginFaultKind>,
    pub(super) last_fault_detail: Option<String>,
    pub(super) last_restart_intent: Option<RecoveryRestartIntent>,
    pub(super) last_stop_reason: Option<StopReason>,
    pub(super) last_processing_epoch: Option<u64>,
    pub(super) readiness_state: Option<String>,
    pub(super) degraded_reasons: Vec<String>,
    pub(super) active_lease_id: Option<String>,
    pub(super) active_region_id: Option<String>,
    pub(super) lv2_prepared_negotiation: Option<RuntimeLv2PreparedNegotiationRecord>,
}

impl RuntimePluginSandboxStateModel {
    pub(super) fn new(sandbox_id: String) -> Self {
        Self {
            sandbox_id,
            plugin_type_id: None,
            plugin_format: None,
            instance_id: None,
            state: RuntimePluginLifecycleState::Stopped,
            lifecycle_stage: None,
            transport_stage: None,
            active: false,
            active_transport: false,
            restart_count: 0,
            recovery_count: 0,
            fault_count: 0,
            last_fault_kind: None,
            last_fault_detail: None,
            last_restart_intent: None,
            last_stop_reason: None,
            last_processing_epoch: None,
            readiness_state: None,
            degraded_reasons: Vec::new(),
            active_lease_id: None,
            active_region_id: None,
            lv2_prepared_negotiation: None,
        }
    }

    pub(crate) fn snapshot(&self) -> RuntimePluginSandboxSnapshot {
        RuntimePluginSandboxSnapshot {
            sandbox_id: self.sandbox_id.clone(),
            sandbox_group_key: self.sandbox_id.clone(),
            plugin_type_id: self.plugin_type_id.clone(),
            plugin_format: self.plugin_format,
            instance_id: self.instance_id.clone(),
            placement_outcome: RuntimePluginIsolationOutcome::IsolatedSandbox,
            placement_rule_id: None,
            shared_boundary_member_count: 1,
            continuity_class: RuntimeInterruptionClass::Steady,
            rebindable: false,
            state: self.state,
            lifecycle_stage: self.lifecycle_stage,
            transport_stage: self.transport_stage,
            active: self.active,
            active_transport: self.active_transport,
            restart_count: self.restart_count,
            recovery_count: self.recovery_count,
            fault_count: self.fault_count,
            last_fault_kind: self.last_fault_kind,
            last_fault_detail: self.last_fault_detail.clone(),
            last_restart_intent: self.last_restart_intent,
            last_stop_reason: self.last_stop_reason,
            last_processing_epoch: self.last_processing_epoch,
            readiness_state: self.readiness_state.clone(),
            degraded_reasons: self.degraded_reasons.clone(),
            active_lease_id: self.active_lease_id.clone(),
            active_region_id: self.active_region_id.clone(),
            lv2_prepared_negotiation: self.lv2_prepared_negotiation.clone(),
        }
    }
}
