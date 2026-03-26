use signal_ipc::PluginMessageEnvelope;
use signal_plugin::{BlockDispatch, BlockPayload, BlockProcessResult};

use super::failure::failure_event;
use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub(super) fn read_completion(
        &self,
        dispatch: &BlockDispatch,
    ) -> Result<BlockProcessResult, PluginMessageEnvelope> {
        let lease = self.active_lease.as_ref().ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "processBlock",
                "invalidState",
                "sandbox instance has no active lease",
                self.prepared_epoch,
                None,
                None,
            )
        })?;
        let transport = lease.transport().cloned().ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "processBlock",
                "resourceUnavailable",
                "sandbox instance has no attached shared-memory transport",
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;
        let region = self.broker.attach_region(&transport).map_err(|error| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.current_instance_id(),
                "processBlock",
                "resourceUnavailable",
                format!("failed to attach shared-memory region: {error}"),
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;

        BlockProcessResult::read_from_shared_memory(dispatch.layout, region.as_slice()).map_err(
            |detail| {
                failure_event(
                    self.sandbox_id.as_deref().unwrap_or("unknown"),
                    self.active_instance
                        .as_ref()
                        .map(|instance| instance.instance_id.0.clone()),
                    "processBlock",
                    "protocolViolation",
                    detail,
                    self.prepared_epoch,
                    Some(lease.lease_id.clone()),
                    None,
                )
            },
        )
    }

    pub(super) fn read_input_payload(
        &self,
        dispatch: &BlockDispatch,
    ) -> Result<BlockPayload, PluginMessageEnvelope> {
        let lease = self.active_lease.as_ref().ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "processBlock",
                "invalidState",
                "sandbox instance has no active lease",
                self.prepared_epoch,
                None,
                None,
            )
        })?;
        let transport = lease.transport().cloned().ok_or_else(|| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "processBlock",
                "resourceUnavailable",
                "sandbox instance has no attached shared-memory transport",
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;
        let region = self.broker.attach_region(&transport).map_err(|error| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "processBlock",
                "resourceUnavailable",
                format!("failed to attach shared-memory region: {error}"),
                self.prepared_epoch,
                Some(lease.lease_id.clone()),
                None,
            )
        })?;

        dispatch
            .read_input_payload(region.as_slice())
            .map_err(|detail| {
                failure_event(
                    self.sandbox_id.as_deref().unwrap_or("unknown"),
                    self.active_instance
                        .as_ref()
                        .map(|instance| instance.instance_id.0.clone()),
                    "processBlock",
                    "protocolViolation",
                    detail,
                    self.prepared_epoch,
                    Some(lease.lease_id.clone()),
                    None,
                )
            })
    }
}
