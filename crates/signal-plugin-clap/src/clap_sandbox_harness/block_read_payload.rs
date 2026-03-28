use signal_plugin::{BlockDispatch, BlockPayload, BlockProcessResult};

use crate::ClapHarnessResult;

use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub(super) fn read_completion(
        &self,
        dispatch: &BlockDispatch,
    ) -> ClapHarnessResult<BlockProcessResult> {
        let lease = self.active_lease.as_ref().ok_or_else(|| {
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
        let region = self.broker.attach_region(&transport).map_err(|error| {
            self.failure_error_for_instance(
                self.current_instance_id(),
                Some(lease.lease_id.clone()),
                "processBlock",
                "resourceUnavailable",
                format!("failed to attach shared-memory region: {error}"),
                None,
            )
        })?;

        BlockProcessResult::read_from_shared_memory(dispatch.layout, region.as_slice()).map_err(
            |detail| {
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
            },
        )
    }

    pub(super) fn read_input_payload(
        &self,
        dispatch: &BlockDispatch,
    ) -> ClapHarnessResult<BlockPayload> {
        let lease = self.active_lease.as_ref().ok_or_else(|| {
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
        let region = self.broker.attach_region(&transport).map_err(|error| {
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

        dispatch
            .read_input_payload(region.as_slice())
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
            })
    }
}
