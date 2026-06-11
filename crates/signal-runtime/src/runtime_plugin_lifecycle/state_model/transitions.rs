use super::*;

impl RuntimePluginLifecycleStateModel {
    pub(crate) fn record_fault(
        &mut self,
        sandbox_id: &str,
        kind: PluginFaultKind,
        detail: String,
        processing_epoch: Option<u64>,
    ) {
        let threshold = self.policy.quarantine_after_faults;
        let sandbox = self.sandbox_mut(sandbox_id);
        sandbox.fault_count = sandbox.fault_count.saturating_add(1);
        sandbox.last_fault_kind = Some(kind);
        sandbox.last_fault_detail = Some(detail);
        sandbox.last_processing_epoch = processing_epoch;
        sandbox.active = false;
        sandbox.active_transport = false;
        sandbox.transport_stage = Some(PluginSandboxTransportStage::DetachFault);
        sandbox.active_lease_id = None;
        sandbox.active_region_id = None;
        sandbox.state = if sandbox.fault_count >= threshold {
            RuntimePluginLifecycleState::Quarantined
        } else {
            RuntimePluginLifecycleState::Faulted
        };
    }

    pub(crate) fn record_recovery_cycle(
        &mut self,
        sandbox_id: &str,
        intent: RecoveryRestartIntent,
        stop_reason: StopReason,
        processing_epoch: Option<u64>,
    ) {
        let sandbox = self.sandbox_mut(sandbox_id);
        sandbox.recovery_count = sandbox.recovery_count.saturating_add(1);
        sandbox.last_restart_intent = Some(intent);
        sandbox.last_stop_reason = Some(stop_reason);
        sandbox.last_processing_epoch = processing_epoch;
        sandbox.active = false;
        sandbox.active_transport = false;
        sandbox.active_lease_id = None;
        sandbox.active_region_id = None;
        if sandbox.state != RuntimePluginLifecycleState::Quarantined {
            sandbox.state = RuntimePluginLifecycleState::Restarting;
        }
    }

    pub(crate) fn record_lifecycle(
        &mut self,
        sandbox_id: &str,
        stage: PluginSandboxLifecycleStage,
        processing_epoch: Option<u64>,
    ) {
        let sandbox = self.sandbox_mut(sandbox_id);
        sandbox.lifecycle_stage = Some(stage);
        sandbox.last_processing_epoch = processing_epoch;
        match stage {
            PluginSandboxLifecycleStage::SandboxEnsured
            | PluginSandboxLifecycleStage::SandboxHandshaken
            | PluginSandboxLifecycleStage::PluginTypeLoaded
            | PluginSandboxLifecycleStage::InstanceCreated => {
                if sandbox.state != RuntimePluginLifecycleState::Quarantined {
                    sandbox.state = RuntimePluginLifecycleState::Booting;
                    sandbox.active = true;
                }
            }
            PluginSandboxLifecycleStage::InstancePrepared
            | PluginSandboxLifecycleStage::TransportAttached
            | PluginSandboxLifecycleStage::InstanceActivated => {
                if sandbox.state != RuntimePluginLifecycleState::Quarantined {
                    sandbox.state = if sandbox.degraded_reasons.is_empty() {
                        RuntimePluginLifecycleState::Ready
                    } else {
                        RuntimePluginLifecycleState::Degraded
                    };
                    sandbox.active = true;
                }
            }
            PluginSandboxLifecycleStage::InstanceDeactivated
            | PluginSandboxLifecycleStage::InstanceReset => {
                if sandbox.state != RuntimePluginLifecycleState::Quarantined {
                    sandbox.state = RuntimePluginLifecycleState::Degraded;
                }
                sandbox.active = false;
            }
            PluginSandboxLifecycleStage::InstanceDestroyed
            | PluginSandboxLifecycleStage::SandboxTeardown
            | PluginSandboxLifecycleStage::TransportTornDown => {
                sandbox.state = RuntimePluginLifecycleState::Stopped;
                sandbox.active = false;
                sandbox.active_transport = false;
                sandbox.active_lease_id = None;
                sandbox.active_region_id = None;
            }
            PluginSandboxLifecycleStage::SandboxRestarted => {
                sandbox.restart_count = sandbox.restart_count.saturating_add(1);
                sandbox.active = false;
                sandbox.active_transport = false;
                sandbox.active_lease_id = None;
                sandbox.active_region_id = None;
                if sandbox.state != RuntimePluginLifecycleState::Quarantined {
                    sandbox.state = RuntimePluginLifecycleState::Restarting;
                }
            }
        }
    }

    pub(crate) fn record_instance_state(&mut self, state: PluginSandboxInstanceStateRecord) {
        let sandbox = self.sandbox_mut(state.sandbox_id.as_str());
        sandbox.plugin_type_id = Some(state.plugin_type_id.clone());
        sandbox.instance_id = Some(state.instance_id.clone());
        sandbox.last_processing_epoch = state.processing_epoch;
        sandbox.readiness_state = Some(state.readiness_state.clone());
        sandbox.degraded_reasons = state.degraded_reasons.clone();
        sandbox.active = state.active;
        if let Some(last_fault) = state.last_fault.as_ref() {
            sandbox.last_fault_detail = Some(last_fault.message.clone());
        }
        if sandbox.state != RuntimePluginLifecycleState::Quarantined
            && sandbox.state != RuntimePluginLifecycleState::Restarting
            && sandbox.state != RuntimePluginLifecycleState::Stopped
        {
            sandbox.state = if !sandbox.degraded_reasons.is_empty() {
                RuntimePluginLifecycleState::Degraded
            } else if state.last_fault.is_some() && !state.active {
                RuntimePluginLifecycleState::Faulted
            } else if state.active && state.readiness_state.eq_ignore_ascii_case("ready") {
                RuntimePluginLifecycleState::Ready
            } else if state.active {
                RuntimePluginLifecycleState::Booting
            } else {
                RuntimePluginLifecycleState::Degraded
            };
        }
    }

    pub(crate) fn record_transport(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        stage: PluginSandboxTransportStage,
        processing_epoch: Option<u64>,
        detail: Option<String>,
    ) {
        let sandbox = self.sandbox_mut(sandbox_id);
        sandbox.transport_stage = Some(stage);
        sandbox.last_processing_epoch = processing_epoch;
        match stage {
            PluginSandboxTransportStage::Attached => {
                sandbox.active_transport = true;
                sandbox.active = true;
                sandbox.active_lease_id = Some(lease_id.to_string());
                sandbox.active_region_id = Some(region_id.to_string());
                if sandbox.state != RuntimePluginLifecycleState::Quarantined
                    && sandbox.state != RuntimePluginLifecycleState::Faulted
                {
                    sandbox.state = if sandbox.degraded_reasons.is_empty() {
                        RuntimePluginLifecycleState::Ready
                    } else {
                        RuntimePluginLifecycleState::Degraded
                    };
                }
            }
            PluginSandboxTransportStage::DetachRequested => {
                sandbox.active_transport = false;
                sandbox.active = false;
                if sandbox.state != RuntimePluginLifecycleState::Quarantined
                    && sandbox.state != RuntimePluginLifecycleState::Faulted
                {
                    sandbox.state = RuntimePluginLifecycleState::Degraded;
                }
            }
            PluginSandboxTransportStage::Detached => {
                sandbox.active_transport = false;
                sandbox.active = false;
                sandbox.active_lease_id = None;
                sandbox.active_region_id = None;
                if sandbox.state != RuntimePluginLifecycleState::Quarantined
                    && sandbox.state != RuntimePluginLifecycleState::Faulted
                    && sandbox.state != RuntimePluginLifecycleState::Stopped
                {
                    sandbox.state = RuntimePluginLifecycleState::Degraded;
                }
            }
            PluginSandboxTransportStage::DetachFault => {
                sandbox.active_transport = false;
                sandbox.active = false;
                sandbox.active_lease_id = None;
                sandbox.active_region_id = None;
                sandbox.last_fault_detail = detail;
                if sandbox.state != RuntimePluginLifecycleState::Quarantined {
                    sandbox.state = RuntimePluginLifecycleState::Degraded;
                }
            }
        }
    }
}
