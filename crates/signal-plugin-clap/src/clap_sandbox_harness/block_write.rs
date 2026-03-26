use signal_ipc::PluginMessageEnvelope;
use signal_plugin::{BlockDispatch, BlockPayload, BlockProcessResult};

use super::failure::failure_event;
use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub(super) fn write_completion(
        &mut self,
        result: BlockProcessResult,
    ) -> Result<BlockProcessResult, PluginMessageEnvelope> {
        let lease = self.active_lease.clone().ok_or_else(|| {
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
        let mut region = self.broker.attach_region(&transport).map_err(|error| {
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
        result
            .write_to_shared_memory(lease.layout, region.as_mut_slice())
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
            })?;
        region.flush().map_err(|error| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "processBlock",
                "resourceUnavailable",
                format!("failed to flush shared-memory region: {error}"),
                self.prepared_epoch,
                Some(lease.lease_id),
                None,
            )
        })?;
        Ok(result)
    }

    pub(super) fn commit_processed_block(
        &mut self,
        dispatch: &BlockDispatch,
        payload: &BlockPayload,
        result: BlockProcessResult,
    ) -> Result<BlockProcessResult, PluginMessageEnvelope> {
        let lease = self.active_lease.clone().ok_or_else(|| {
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
        let mut region = self.broker.attach_region(&transport).map_err(|error| {
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
        dispatch
            .write_output_payload(region.as_mut_slice(), payload)
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
            })?;
        result
            .write_to_shared_memory(dispatch.layout, region.as_mut_slice())
            .map_err(|detail| {
                failure_event(
                    self.sandbox_id.as_deref().unwrap_or("unknown"),
                    self.current_instance_id(),
                    "processBlock",
                    "protocolViolation",
                    detail,
                    self.prepared_epoch,
                    Some(lease.lease_id.clone()),
                    None,
                )
            })?;
        region.flush().map_err(|error| {
            failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "processBlock",
                "resourceUnavailable",
                format!("failed to flush shared-memory region: {error}"),
                self.prepared_epoch,
                Some(lease.lease_id),
                None,
            )
        })?;
        Ok(result)
    }
}
