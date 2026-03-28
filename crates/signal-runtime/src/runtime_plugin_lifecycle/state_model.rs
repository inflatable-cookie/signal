use super::*;
use crate::runtime::runtime_plugin_recording::runtime_plugin_parity_coverage;
#[path = "state_model/sandbox_state.rs"]
mod sandbox_state;
use super::placement::runtime_plugin_sandbox_snapshot;
use sandbox_state::RuntimePluginLifecyclePolicy;
pub(super) use sandbox_state::RuntimePluginSandboxStateModel;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct RuntimePluginLifecycleStateModel {
    policy: RuntimePluginLifecyclePolicy,
    sandboxes: BTreeMap<String, RuntimePluginSandboxStateModel>,
    active_sandbox_count: u32,
}

impl RuntimePluginLifecycleStateModel {
    fn sandbox_mut(&mut self, sandbox_id: &str) -> &mut RuntimePluginSandboxStateModel {
        self.sandboxes
            .entry(sandbox_id.to_string())
            .or_insert_with(|| RuntimePluginSandboxStateModel::new(sandbox_id.to_string()))
    }

    pub(crate) fn record_spec(&mut self, spec: &PluginSandboxSpec) {
        let sandbox = self.sandbox_mut(spec.sandbox_id.as_str());
        sandbox.plugin_format = Some(spec.plugin_format);
        sandbox.plugin_type_id = spec.plugin_type_id.clone();
    }

    pub(crate) fn record_preset_descriptor(
        &mut self,
        sandbox_id: &str,
        descriptor: RuntimePluginPresetDescriptor,
    ) {
        self.sandbox_mut(sandbox_id).preset_descriptor = Some(descriptor);
    }

    pub(crate) fn record_ara_context(
        &mut self,
        sandbox_id: &str,
        context: RuntimePluginAraContextSnapshot,
    ) {
        self.sandbox_mut(sandbox_id).ara_context = Some(context);
    }

    pub(crate) fn snapshot(
        &self,
        policy: &RuntimePluginPlacementPolicy,
        boundary_stage_counts: &HashMap<String, usize>,
        discovered_types: &[RuntimePluginDiscoveredTypeRecord],
        platform_coverage: &[RuntimePluginFormatPlatformCoverageRecord],
    ) -> RuntimePluginLifecycleSnapshot {
        let sandboxes = self
            .sandboxes
            .values()
            .map(|sandbox| {
                runtime_plugin_sandbox_snapshot(
                    sandbox,
                    policy,
                    boundary_stage_counts
                        .get(sandbox.sandbox_id.as_str())
                        .copied()
                        .unwrap_or(1),
                )
            })
            .collect::<Vec<_>>();
        let mut snapshot = RuntimePluginLifecycleSnapshot {
            sandbox_count: sandboxes.len(),
            active_sandbox_count: self.active_sandbox_count,
            shared_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| {
                    sandbox.placement_outcome == RuntimePluginIsolationOutcome::SharedSandbox
                })
                .count(),
            isolated_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| {
                    sandbox.placement_outcome == RuntimePluginIsolationOutcome::IsolatedSandbox
                })
                .count(),
            ready_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Ready)
                .count(),
            booting_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Booting)
                .count(),
            degraded_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Degraded)
                .count(),
            faulted_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Faulted)
                .count(),
            restarting_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Restarting)
                .count(),
            quarantined_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Quarantined)
                .count(),
            stopped_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.state == RuntimePluginLifecycleState::Stopped)
                .count(),
            rebindable_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.rebindable)
                .count(),
            terminal_sandbox_count: sandboxes
                .iter()
                .filter(|sandbox| sandbox.continuity_class == RuntimeInterruptionClass::Terminal)
                .count(),
            parity_coverage: runtime_plugin_parity_coverage(
                discovered_types,
                &sandboxes,
                policy,
                platform_coverage,
            ),
            sandboxes,
            summary: String::new(),
        };
        snapshot.summary = format!(
            "sandboxes={} active={} shared={} isolated={} ready={} booting={} degraded={} faulted={} restarting={} quarantined={} stopped={} rebindable={} terminal={}",
            snapshot.sandbox_count,
            snapshot.active_sandbox_count,
            snapshot.shared_sandbox_count,
            snapshot.isolated_sandbox_count,
            snapshot.ready_sandbox_count,
            snapshot.booting_sandbox_count,
            snapshot.degraded_sandbox_count,
            snapshot.faulted_sandbox_count,
            snapshot.restarting_sandbox_count,
            snapshot.quarantined_sandbox_count,
            snapshot.stopped_sandbox_count,
            snapshot.rebindable_sandbox_count,
            snapshot.terminal_sandbox_count,
        );
        snapshot
    }

    pub(crate) fn set_active_sandbox_count(&mut self, count: u32) {
        self.active_sandbox_count = count;
    }

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
        sandbox.ara_context = None;
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
        sandbox.ara_context = None;
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
                sandbox.ara_context = None;
            }
            PluginSandboxLifecycleStage::SandboxRestarted => {
                sandbox.restart_count = sandbox.restart_count.saturating_add(1);
                sandbox.active = false;
                sandbox.active_transport = false;
                sandbox.active_lease_id = None;
                sandbox.active_region_id = None;
                sandbox.ara_context = None;
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
                sandbox.ara_context = None;
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
                sandbox.ara_context = None;
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
                sandbox.ara_context = None;
                sandbox.last_fault_detail = detail;
                if sandbox.state != RuntimePluginLifecycleState::Quarantined {
                    sandbox.state = RuntimePluginLifecycleState::Degraded;
                }
            }
        }
    }
}
