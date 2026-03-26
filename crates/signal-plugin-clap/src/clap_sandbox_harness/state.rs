use std::io;

use signal_ipc::{
    PluginInstanceStatePayload, PluginProcessConfigurationPayload, SharedMemoryBroker,
};
use signal_plugin::{
    PluginFault, PluginIoLayout, PluginLifecycleState, PluginReadiness, SandboxStateMachine,
    SharedMemoryLease,
};

use crate::{ClapDiscoveredPluginType, ClapInstanceControlSurface, ClapPluginHostAdapter};

use super::failure::{failure_event, lifecycle_state_string, plugin_fault_payload};

#[derive(Debug)]
pub struct ClapSandboxLifecycleHarness {
    pub(super) adapter: ClapPluginHostAdapter,
    pub(super) broker: SharedMemoryBroker,
    pub(super) sandbox_id: Option<String>,
    pub(super) loaded_plugin: Option<ClapDiscoveredPluginType>,
    pub(super) active_instance: Option<ClapInstanceControlSurface>,
    pub(super) active_io_layout: Option<PluginIoLayout>,
    pub(super) active_lease: Option<SharedMemoryLease>,
    pub(super) prepared_epoch: Option<u64>,
    pub(super) prepared_sample_rate_hz: Option<u32>,
    pub(super) prepared_max_block_frames: Option<u32>,
    pub(super) active: bool,
    pub(super) last_fault: Option<PluginFault>,
    pub(super) block_machine: SandboxStateMachine,
    pub(super) heartbeat_count: u64,
}

impl Default for ClapSandboxLifecycleHarness {
    fn default() -> Self {
        Self {
            adapter: ClapPluginHostAdapter::default(),
            broker: SharedMemoryBroker::default(),
            sandbox_id: None,
            loaded_plugin: None,
            active_instance: None,
            active_io_layout: None,
            active_lease: None,
            prepared_epoch: None,
            prepared_sample_rate_hz: None,
            prepared_max_block_frames: None,
            active: false,
            last_fault: None,
            block_machine: SandboxStateMachine::new(),
            heartbeat_count: 0,
        }
    }
}

impl ClapSandboxLifecycleHarness {
    pub(super) fn current_instance_id(&self) -> Option<String> {
        self.active_instance
            .as_ref()
            .map(|instance| instance.instance_id.0.clone())
    }

    pub(super) fn current_lifecycle_state(&self) -> Option<PluginLifecycleState> {
        if self.active {
            Some(PluginLifecycleState::Active)
        } else if self.prepared_epoch.is_some() {
            Some(PluginLifecycleState::Prepared)
        } else if self.active_instance.is_some() {
            Some(PluginLifecycleState::InstanceCreated)
        } else if self.loaded_plugin.is_some() {
            Some(PluginLifecycleState::TypeLoaded)
        } else if self.sandbox_id.is_some() {
            Some(PluginLifecycleState::Discovered)
        } else {
            None
        }
    }

    pub(super) fn current_readiness(&self) -> PluginReadiness {
        match self.last_fault.clone() {
            Some(fault) => PluginReadiness::from_fault(fault),
            None if self.active => PluginReadiness::Ready,
            None if self.sandbox_id.is_some() => PluginReadiness::Starting,
            None => PluginReadiness::Stopped,
        }
    }

    pub(super) fn process_configuration_payload(
        &self,
    ) -> Option<PluginProcessConfigurationPayload> {
        Some(PluginProcessConfigurationPayload {
            sample_rate_hz: self.prepared_sample_rate_hz?,
            max_block_frames: self.prepared_max_block_frames?,
            io_layout: crate::io_layout_payload(self.active_io_layout?),
        })
    }

    pub(super) fn instance_state_payload(
        &self,
        instance_id: &str,
        lifecycle_state: PluginLifecycleState,
    ) -> Option<PluginInstanceStatePayload> {
        let plugin_type_id = self.loaded_plugin.as_ref()?.plugin_type_id.0.clone();
        let readiness = self.current_readiness();
        let (readiness_state, degraded_reasons) = match readiness {
            PluginReadiness::Starting => ("Starting".to_string(), Vec::new()),
            PluginReadiness::Ready => ("Ready".to_string(), Vec::new()),
            PluginReadiness::Stopped => ("Stopped".to_string(), Vec::new()),
            PluginReadiness::Degraded { reasons } => (
                "Degraded".to_string(),
                reasons
                    .into_iter()
                    .map(|reason| reason.0.to_string())
                    .collect(),
            ),
            PluginReadiness::Failed { .. } => ("Failed".to_string(), Vec::new()),
        };

        Some(PluginInstanceStatePayload {
            plugin_type_id,
            instance_id: instance_id.to_string(),
            lifecycle_state: lifecycle_state_string(lifecycle_state).into(),
            readiness_state,
            degraded_reasons,
            active: self.active,
            processing: self.process_configuration_payload(),
            last_fault: self.last_fault.as_ref().map(plugin_fault_payload),
        })
    }

    pub fn lease(&self) -> Option<&SharedMemoryLease> {
        self.active_lease.as_ref()
    }

    pub fn heartbeat_count(&self) -> u64 {
        self.heartbeat_count
    }

    pub fn invalidate_active_epoch(&mut self, processing_epoch: u64) -> (bool, bool) {
        let lease_invalidated = self
            .active_lease
            .as_mut()
            .map(|lease| lease.invalidate_epoch(processing_epoch))
            .unwrap_or(false);
        let completion_invalidated =
            self.prepared_epoch.is_some() || self.active_lease.is_some() || self.active;
        if completion_invalidated {
            self.block_machine.invalidate_epoch(processing_epoch);
        }
        (completion_invalidated, lease_invalidated)
    }

    pub fn teardown_active_transport(&mut self) -> io::Result<()> {
        if let Some(transport) = self
            .active_lease
            .as_ref()
            .and_then(|lease| lease.transport().cloned())
        {
            self.broker.destroy_region(&transport)?;
        }
        self.active_lease = None;
        self.prepared_epoch = None;
        self.active = false;
        self.active_io_layout = None;
        self.block_machine = SandboxStateMachine::new();
        Ok(())
    }

    pub(super) fn require_sandbox(
        &self,
        sandbox_id: &str,
        stage: &str,
        correlation: Option<signal_ipc::CorrelationId>,
    ) -> Result<(), signal_ipc::PluginMessageEnvelope> {
        if self.sandbox_id.as_deref() == Some(sandbox_id) {
            return Ok(());
        }
        Err(failure_event(
            sandbox_id,
            self.current_instance_id(),
            stage,
            "invalidState",
            "sandbox id does not match established handshake",
            self.prepared_epoch,
            self.active_lease
                .as_ref()
                .map(|lease| lease.lease_id.clone()),
            correlation,
        ))
    }

    pub(super) fn require_instance(
        &self,
        instance_id: &str,
        stage: &str,
        correlation: Option<signal_ipc::CorrelationId>,
    ) -> Result<(), signal_ipc::PluginMessageEnvelope> {
        if self
            .active_instance
            .as_ref()
            .map(|instance| instance.instance_id.0.as_str())
            == Some(instance_id)
        {
            return Ok(());
        }
        Err(failure_event(
            self.sandbox_id.as_deref().unwrap_or("unknown"),
            Some(instance_id.to_string()),
            stage,
            "invalidState",
            "instance id does not match created CLAP instance",
            self.prepared_epoch,
            self.active_lease
                .as_ref()
                .map(|lease| lease.lease_id.clone()),
            correlation,
        ))
    }
}
