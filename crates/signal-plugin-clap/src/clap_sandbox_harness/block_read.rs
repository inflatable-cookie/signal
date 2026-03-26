use signal_ipc::PluginMessageEnvelope;
use signal_plugin::{BlockDispatch, PluginInstanceId};

use super::failure::failure_event;
use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub(super) fn read_pending_dispatch(
        &mut self,
        stage: &str,
    ) -> Result<BlockDispatch, PluginMessageEnvelope> {
        if !self.active {
            return Err(failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                stage,
                "invalidState",
                "block processing requested while sandbox instance is not active",
                self.prepared_epoch,
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                None,
            ));
        }

        let lease = self.active_lease.clone().ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.current_instance_id(),
                stage,
                "invalidState",
                "sandbox instance has no prepared lease",
                self.prepared_epoch,
                None,
                None,
            )
        })?;
        let transport = lease.transport().cloned().ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.current_instance_id(),
                stage,
                "resourceUnavailable",
                "sandbox instance has no attached shared-memory transport",
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;
        let instance_id = self.current_instance_id().ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                None,
                stage,
                "invalidState",
                "sandbox instance id is not set",
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;
        let io_layout = self.active_io_layout.ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                Some(instance_id.clone()),
                stage,
                "invalidState",
                "sandbox instance has no negotiated io layout",
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;

        let region = self.broker.attach_region(&transport).map_err(|error| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                Some(instance_id.clone()),
                stage,
                "resourceUnavailable",
                format!("failed to attach shared-memory region: {error}"),
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;
        let dispatch = BlockDispatch::read_from_shared_memory(
            PluginInstanceId(instance_id.clone()),
            io_layout,
            lease.layout,
            region.as_slice(),
        )
        .map_err(|detail| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                Some(instance_id),
                stage,
                "protocolViolation",
                detail,
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;

        if self.prepared_epoch != Some(dispatch.header.processing_epoch)
            || !lease.is_epoch_valid(dispatch.header.processing_epoch)
        {
            return Err(failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.current_instance_id(),
                stage,
                "protocolViolation",
                "shared-memory block dispatch epoch is stale or unprepared",
                Some(dispatch.header.processing_epoch),
                Some(lease.lease_id),
                None,
            ));
        }

        Ok(dispatch)
    }
}
