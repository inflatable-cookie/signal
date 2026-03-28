use signal_ipc::{
    CorrelationId, PluginDescriptorPayload, PluginMessageEnvelope, PluginMessageName,
    PluginMessagePayload,
};
use signal_plugin::{PluginFormat, PluginLifecycleState};

use super::descriptor::descriptor_payload;
use super::failure::{failure_event, ClapSandboxFailureInput};
use super::state::ClapSandboxLifecycleHarness;
use crate::ClapHarnessResult;

impl ClapSandboxLifecycleHarness {
    pub(super) fn handle_handshake(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        format: String,
    ) -> ClapHarnessResult<PluginMessageEnvelope> {
        if format != "clap" || !self.adapter.supports_format(PluginFormat::Clap) {
            return Err(Box::new(failure_event(ClapSandboxFailureInput {
                sandbox_id,
                instance_id: None,
                stage: "handshake".to_string(),
                error_kind: "unsupported".to_string(),
                detail: "sandbox handshake requested unsupported format".to_string(),
                processing_epoch: None,
                shared_memory_lease_id: None,
                correlation_id: Some(correlation),
                instance_state: None,
            })));
        }
        self.sandbox_id = Some(sandbox_id.clone());
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxHandshake,
            correlation,
            PluginMessagePayload::SandboxHandshakeResponse {
                sandbox_id,
                protocol_version: 1,
                supports_state: true,
                supports_midi: true,
                max_block_frames: 2048,
            },
        ))
    }

    pub(super) fn handle_load_plugin_type(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        plugin_type_id: String,
        descriptor: PluginDescriptorPayload,
    ) -> ClapHarnessResult<PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "loadPluginType", Some(correlation.clone()))?;
        let discovered = self
            .adapter
            .discover_plugin_type(&plugin_type_id)
            .ok_or_else(|| {
                Box::new(failure_event(ClapSandboxFailureInput {
                    sandbox_id: sandbox_id.clone(),
                    instance_id: None,
                    stage: "loadPluginType".to_string(),
                    error_kind: "unsupported".to_string(),
                    detail: "requested CLAP plugin type is not available in the local catalog"
                        .to_string(),
                    processing_epoch: None,
                    shared_memory_lease_id: None,
                    correlation_id: Some(correlation.clone()),
                    instance_state: None,
                }))
            })?;
        if descriptor.plugin_id != discovered.descriptor.plugin_id
            || descriptor.vendor != discovered.descriptor.vendor
            || descriptor.name != discovered.descriptor.name
            || descriptor.format != "clap"
        {
            return Err(Box::new(failure_event(ClapSandboxFailureInput {
                sandbox_id,
                instance_id: None,
                stage: "loadPluginType".to_string(),
                error_kind: "protocolViolation".to_string(),
                detail: "loadPluginType descriptor hint does not match discovered CLAP descriptor"
                    .to_string(),
                processing_epoch: None,
                shared_memory_lease_id: None,
                correlation_id: Some(correlation),
                instance_state: None,
            })));
        }
        self.loaded_plugin = Some(discovered.clone());
        self.active_instance = None;
        self.last_fault = None;
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxLoadPluginType,
            correlation,
            PluginMessagePayload::LoadPluginTypeResponse {
                plugin_type_id,
                descriptor: descriptor_payload(&discovered.descriptor),
            },
        ))
    }

    pub(super) fn handle_create_instance(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        plugin_type_id: String,
        instance_id: String,
    ) -> ClapHarnessResult<PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "createInstance", Some(correlation.clone()))?;
        let loaded_plugin = self.loaded_plugin.as_ref().ok_or_else(|| {
            Box::new(failure_event(ClapSandboxFailureInput {
                sandbox_id: sandbox_id.clone(),
                instance_id: Some(instance_id.clone()),
                stage: "createInstance".to_string(),
                error_kind: "invalidState".to_string(),
                detail: "instance creation requested before plugin type load".to_string(),
                processing_epoch: None,
                shared_memory_lease_id: None,
                correlation_id: Some(correlation.clone()),
                instance_state: None,
            }))
        })?;
        if loaded_plugin.plugin_type_id.0 != plugin_type_id {
            return Err(Box::new(failure_event(ClapSandboxFailureInput {
                sandbox_id,
                instance_id: Some(instance_id.clone()),
                stage: "createInstance".to_string(),
                error_kind: "invalidState".to_string(),
                detail: "instance creation requested before plugin type load".to_string(),
                processing_epoch: None,
                shared_memory_lease_id: None,
                correlation_id: Some(correlation),
                instance_state: None,
            })));
        }
        self.active_instance = Some(
            self.adapter
                .instantiate_plugin(loaded_plugin, instance_id.as_str()),
        );
        self.last_fault = None;
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxCreateInstance,
            correlation,
            PluginMessagePayload::CreateInstanceResponse {
                instance_id: instance_id.clone(),
                instance_state: self
                    .instance_state_payload(&instance_id, PluginLifecycleState::InstanceCreated)
                    .expect("instance state after create"),
            },
        ))
    }
}
