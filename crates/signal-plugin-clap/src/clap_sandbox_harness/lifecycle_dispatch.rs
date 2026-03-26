use signal_ipc::{PluginMessageEnvelope, PluginMessagePayload};

use super::failure::failure_event;
use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub fn handle(
        &mut self,
        request: PluginMessageEnvelope,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        let correlation = request.message.correlation_id.clone().ok_or_else(|| {
            failure_event(
                "unknown",
                None,
                "handshake",
                "protocolViolation",
                "plugin lifecycle message is missing correlation_id",
                None,
                None,
                None,
            )
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
            other => Err(failure_event(
                self.sandbox_id.as_deref().unwrap_or("unknown"),
                self.active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                "unsupported",
                "protocolViolation",
                format!("unsupported CLAP lifecycle request: {other:?}"),
                self.prepared_epoch,
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                Some(correlation),
            )),
        }
    }
}
