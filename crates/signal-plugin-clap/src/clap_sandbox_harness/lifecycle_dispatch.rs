use signal_ipc::{PluginMessageEnvelope, PluginMessagePayload};

use super::failure::{failure_event, ClapSandboxFailureInput};
use super::state::ClapSandboxLifecycleHarness;
use crate::ClapHarnessResult;

impl ClapSandboxLifecycleHarness {
    pub fn handle(
        &mut self,
        request: PluginMessageEnvelope,
    ) -> ClapHarnessResult<PluginMessageEnvelope> {
        let correlation = request.message.correlation_id.clone().ok_or_else(|| {
            Box::new(failure_event(ClapSandboxFailureInput {
                sandbox_id: "unknown".to_string(),
                instance_id: None,
                stage: "handshake".to_string(),
                error_kind: "protocolViolation".to_string(),
                detail: "plugin lifecycle message is missing correlation_id".to_string(),
                processing_epoch: None,
                shared_memory_lease_id: None,
                correlation_id: None,
                instance_state: None,
            }))
        })?;

        match request.payload {
            PluginMessagePayload::SandboxHandshakeRequest { sandbox_id, format } => {
                self.handle_handshake(correlation, sandbox_id, format)
            }
            PluginMessagePayload::LoadPluginTypeRequest {
                sandbox_id,
                plugin_type_id,
                descriptor,
            } => self.handle_load_plugin_type(correlation, sandbox_id, plugin_type_id, descriptor),
            PluginMessagePayload::CreateInstanceRequest {
                sandbox_id,
                plugin_type_id,
                instance_id,
            } => self.handle_create_instance(correlation, sandbox_id, plugin_type_id, instance_id),
            PluginMessagePayload::PrepareInstanceRequest {
                sandbox_id,
                instance_id,
                processing_epoch,
                shared_memory_lease_id,
                shared_memory_transport,
                sample_rate_hz,
                max_block_frames,
                io_layout,
                shared_memory,
                ..
            } => self.handle_prepare_instance(
                correlation,
                sandbox_id,
                instance_id,
                processing_epoch,
                shared_memory_lease_id,
                shared_memory_transport,
                sample_rate_hz,
                max_block_frames,
                io_layout,
                shared_memory,
            ),
            PluginMessagePayload::ActivateInstanceRequest {
                sandbox_id,
                instance_id,
                processing_epoch,
            } => self.handle_activate_instance(
                correlation,
                sandbox_id,
                instance_id,
                processing_epoch,
            ),
            PluginMessagePayload::HeartbeatRequest {
                sandbox_id,
                instance_id,
                processing_epoch,
            } => self.handle_heartbeat(correlation, sandbox_id, instance_id, processing_epoch),
            PluginMessagePayload::DeactivateInstanceRequest {
                sandbox_id,
                instance_id,
            } => self.handle_deactivate_instance(correlation, sandbox_id, instance_id),
            PluginMessagePayload::ResetInstanceRequest {
                sandbox_id,
                instance_id,
                processing_epoch,
            } => self.handle_reset_instance(correlation, sandbox_id, instance_id, processing_epoch),
            PluginMessagePayload::DestroyInstanceRequest {
                sandbox_id,
                instance_id,
            } => self.handle_destroy_instance(correlation, sandbox_id, instance_id),
            PluginMessagePayload::SandboxFailure { .. } => {
                self.handle_sandbox_failure_request(correlation)
            }
            other => Err(self.failure_error_for_instance(
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                "unsupported",
                "protocolViolation",
                format!("unsupported CLAP lifecycle request: {other:?}"),
                Some(correlation),
            )),
        }
    }
}
