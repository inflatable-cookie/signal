use signal_plugin::{BlockDispatch, BlockPayload, BlockProcessResult};

use crate::ClapHarnessResult;

use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub(super) fn write_completion(
        &mut self,
        result: BlockProcessResult,
    ) -> ClapHarnessResult<BlockProcessResult> {
        let lease = self.active_lease.clone().ok_or_else(|| {
            self.failure_error(
                "processBlock",
                "invalidState",
                "sandbox instance has no active lease",
                None,
            )
        })?;
        let transport = lease.transport().cloned().ok_or_else(|| {
            self.failure_error_for_instance(
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                Some(lease.lease_id.clone()),
                "processBlock",
                "resourceUnavailable",
                "sandbox instance has no attached shared-memory transport",
                None,
            )
        })?;
        let mut region = self.broker.attach_region(&transport).map_err(|error| {
            self.failure_error_for_instance(
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                Some(lease.lease_id.clone()),
                "processBlock",
                "resourceUnavailable",
                format!("failed to attach shared-memory region: {error}"),
                None,
            )
        })?;
        result
            .write_to_shared_memory(lease.layout, region.as_mut_slice())
            .map_err(|detail| {
                self.failure_error_for_instance(
                    self.active_instance
                        .as_ref()
                        .map(|instance| instance.instance_id.0.clone()),
                    Some(lease.lease_id.clone()),
                    "processBlock",
                    "protocolViolation",
                    detail,
                    None,
                )
            })?;
        region.flush().map_err(|error| {
            self.failure_error_for_instance(
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                Some(lease.lease_id),
                "processBlock",
                "resourceUnavailable",
                format!("failed to flush shared-memory region: {error}"),
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
    ) -> ClapHarnessResult<BlockProcessResult> {
        let lease = self.active_lease.clone().ok_or_else(|| {
            self.failure_error(
                "processBlock",
                "invalidState",
                "sandbox instance has no active lease",
                None,
            )
        })?;
        let transport = lease.transport().cloned().ok_or_else(|| {
            self.failure_error_for_instance(
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                Some(lease.lease_id.clone()),
                "processBlock",
                "resourceUnavailable",
                "sandbox instance has no attached shared-memory transport",
                None,
            )
        })?;
        let mut region = self.broker.attach_region(&transport).map_err(|error| {
            self.failure_error_for_instance(
                self.current_instance_id(),
                Some(lease.lease_id.clone()),
                "processBlock",
                "resourceUnavailable",
                format!("failed to attach shared-memory region: {error}"),
                None,
            )
        })?;
        dispatch
            .write_output_payload(region.as_mut_slice(), payload)
            .map_err(|detail| {
                self.failure_error_for_instance(
                    self.active_instance
                        .as_ref()
                        .map(|instance| instance.instance_id.0.clone()),
                    Some(lease.lease_id.clone()),
                    "processBlock",
                    "protocolViolation",
                    detail,
                    None,
                )
            })?;
        result
            .write_to_shared_memory(dispatch.layout, region.as_mut_slice())
            .map_err(|detail| {
                self.failure_error_for_instance(
                    self.current_instance_id(),
                    Some(lease.lease_id.clone()),
                    "processBlock",
                    "protocolViolation",
                    detail,
                    None,
                )
            })?;
        region.flush().map_err(|error| {
            self.failure_error_for_instance(
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                Some(lease.lease_id),
                "processBlock",
                "resourceUnavailable",
                format!("failed to flush shared-memory region: {error}"),
                None,
            )
        })?;
        Ok(result)
    }
}
