use signal_plugin::{BlockDispatch, PluginInstanceId};

use crate::ClapHarnessResult;

use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub(super) fn read_pending_dispatch(
        &mut self,
        stage: &str,
    ) -> ClapHarnessResult<BlockDispatch> {
        if !self.active {
            return Err(self.failure_error_for_instance(
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                stage,
                "invalidState",
                "block processing requested while sandbox instance is not active",
                None,
            ));
        }

        let lease = self.active_lease.clone().ok_or_else(|| {
            self.failure_error(
                stage,
                "invalidState",
                "sandbox instance has no prepared lease",
                None,
            )
        })?;
        let transport = lease.transport().cloned().ok_or_else(|| {
            self.failure_error_for_instance(
                self.current_instance_id(),
                Some(lease.lease_id.clone()),
                stage,
                "resourceUnavailable",
                "sandbox instance has no attached shared-memory transport",
                None,
            )
        })?;
        let instance_id = self.current_instance_id().ok_or_else(|| {
            self.failure_error_for_instance(
                None,
                Some(lease.lease_id.clone()),
                stage,
                "invalidState",
                "sandbox instance id is not set",
                None,
            )
        })?;
        let io_layout = self.active_io_layout.ok_or_else(|| {
            self.failure_error_for_instance(
                Some(instance_id.clone()),
                Some(lease.lease_id.clone()),
                stage,
                "invalidState",
                "sandbox instance has no negotiated io layout",
                None,
            )
        })?;

        let region = self.broker.attach_region(&transport).map_err(|error| {
            self.failure_error_for_instance(
                Some(instance_id.clone()),
                Some(lease.lease_id.clone()),
                stage,
                "resourceUnavailable",
                format!("failed to attach shared-memory region: {error}"),
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
            self.failure_error_for_instance(
                Some(instance_id),
                Some(lease.lease_id.clone()),
                stage,
                "protocolViolation",
                detail,
                None,
            )
        })?;

        if self.prepared_epoch != Some(dispatch.header.processing_epoch)
            || !lease.is_epoch_valid(dispatch.header.processing_epoch)
        {
            return Err(self.failure_error_for_instance(
                self.current_instance_id(),
                Some(lease.lease_id),
                stage,
                "protocolViolation",
                "shared-memory block dispatch epoch is stale or unprepared",
                None,
            ));
        }

        Ok(dispatch)
    }
}
