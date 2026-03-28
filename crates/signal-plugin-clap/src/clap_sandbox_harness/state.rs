use std::io;

use signal_ipc::CorrelationId;
use signal_plugin::{
    PluginFault, PluginIoLayout, SandboxStateMachine, SharedMemoryLease,
};

use crate::{ClapDiscoveredPluginType, ClapInstanceControlSurface, ClapPluginHostAdapter};

use super::failure::{failure_event, ClapSandboxFailureInput};
use super::{ClapHarnessError, ClapHarnessResult};

mod projection;

#[derive(Debug)]
pub struct ClapSandboxLifecycleHarness {
    pub(super) adapter: ClapPluginHostAdapter,
    pub(super) broker: signal_ipc::SharedMemoryBroker,
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
            broker: signal_ipc::SharedMemoryBroker::default(),
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
    pub(super) fn failure_input(
        &self,
        stage: &str,
        error_kind: &str,
        detail: impl Into<String>,
        correlation: Option<CorrelationId>,
    ) -> ClapSandboxFailureInput {
        ClapSandboxFailureInput {
            sandbox_id: self
                .sandbox_id
                .clone()
                .unwrap_or_else(|| "unknown".to_string()),
            instance_id: self.current_instance_id(),
            stage: stage.to_string(),
            error_kind: error_kind.to_string(),
            detail: detail.into(),
            processing_epoch: self.prepared_epoch,
            shared_memory_lease_id: self
                .active_lease
                .as_ref()
                .map(|lease| lease.lease_id.clone()),
            correlation_id: correlation,
            instance_state: None,
        }
    }

    pub(super) fn failure_input_for_instance(
        &self,
        instance_id: Option<String>,
        lease_id: Option<String>,
        stage: &str,
        error_kind: &str,
        detail: impl Into<String>,
        correlation: Option<CorrelationId>,
    ) -> ClapSandboxFailureInput {
        ClapSandboxFailureInput {
            instance_id,
            shared_memory_lease_id: lease_id,
            ..self.failure_input(stage, error_kind, detail, correlation)
        }
    }

    pub(super) fn failure_error(
        &self,
        stage: &str,
        error_kind: &str,
        detail: impl Into<String>,
        correlation: Option<CorrelationId>,
    ) -> ClapHarnessError {
        Box::new(failure_event(self.failure_input(
            stage,
            error_kind,
            detail,
            correlation,
        )))
    }

    pub(super) fn failure_error_for_instance(
        &self,
        instance_id: Option<String>,
        lease_id: Option<String>,
        stage: &str,
        error_kind: &str,
        detail: impl Into<String>,
        correlation: Option<CorrelationId>,
    ) -> ClapHarnessError {
        Box::new(failure_event(self.failure_input_for_instance(
            instance_id,
            lease_id,
            stage,
            error_kind,
            detail,
            correlation,
        )))
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
    ) -> ClapHarnessResult<()> {
        if self.sandbox_id.as_deref() == Some(sandbox_id) {
            return Ok(());
        }
        Err(Box::new(failure_event(ClapSandboxFailureInput {
            sandbox_id: sandbox_id.to_string(),
            instance_id: self.current_instance_id(),
            stage: stage.to_string(),
            error_kind: "invalidState".to_string(),
            detail: "sandbox id does not match established handshake".to_string(),
            processing_epoch: self.prepared_epoch,
            shared_memory_lease_id: self
                .active_lease
                .as_ref()
                .map(|lease| lease.lease_id.clone()),
            correlation_id: correlation,
            instance_state: None,
        })))
    }

    pub(super) fn require_instance(
        &self,
        instance_id: &str,
        stage: &str,
        correlation: Option<signal_ipc::CorrelationId>,
    ) -> ClapHarnessResult<()> {
        if self
            .active_instance
            .as_ref()
            .map(|instance| instance.instance_id.0.as_str())
            == Some(instance_id)
        {
            return Ok(());
        }
        Err(self.failure_error_for_instance(
            Some(instance_id.to_string()),
            self.active_lease
                .as_ref()
                .map(|lease| lease.lease_id.clone()),
            stage,
            "invalidState",
            "instance id does not match created CLAP instance",
            correlation,
        ))
    }
}
