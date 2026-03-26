use signal_ipc::{
    CorrelationId, PluginInstanceStatePayload, PluginMessageEnvelope, PluginMessageName,
    PluginMessagePayload,
};
use signal_plugin::{PluginLifecycleState, SandboxStateMachine};

use super::failure::{failure_event, lifecycle_state_string};
use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    pub(super) fn handle_deactivate_instance(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        instance_id: String,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "deactivateInstance", Some(correlation.clone()))?;
        self.require_instance(
            &instance_id,
            "deactivateInstance",
            Some(correlation.clone()),
        )?;
        self.active = false;
        self.last_fault = None;
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxDeactivateInstance,
            correlation,
            PluginMessagePayload::DeactivateInstanceResponse {
                instance_id: instance_id.clone(),
                instance_state: self
                    .instance_state_payload(&instance_id, PluginLifecycleState::Inactive)
                    .expect("instance state after deactivate"),
            },
        ))
    }

    pub(super) fn handle_reset_instance(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "resetInstance", Some(correlation.clone()))?;
        self.require_instance(&instance_id, "resetInstance", Some(correlation.clone()))?;
        if self.active
            && !self
                .active_instance
                .as_ref()
                .expect("validated instance")
                .lifecycle_contract
                .supports_reset_while_active
        {
            return Err(failure_event(
                &sandbox_id,
                Some(instance_id),
                "resetInstance",
                "invalidState",
                "loaded CLAP instance does not support reset while active",
                Some(processing_epoch),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                Some(correlation),
            ));
        }
        if let Some(lease) = &mut self.active_lease {
            if lease.processing_epoch != processing_epoch {
                lease.invalidate_epoch(lease.processing_epoch);
                lease.processing_epoch = processing_epoch;
            }
        }
        self.prepared_epoch = Some(processing_epoch);
        self.active = false;
        self.last_fault = None;
        self.block_machine = SandboxStateMachine::new();
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxResetInstance,
            correlation,
            PluginMessagePayload::ResetInstanceResponse {
                instance_id: instance_id.clone(),
                processing_epoch,
                instance_state: self
                    .instance_state_payload(&instance_id, PluginLifecycleState::Prepared)
                    .expect("instance state after reset"),
            },
        ))
    }

    pub(super) fn handle_destroy_instance(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        instance_id: String,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "destroyInstance", Some(correlation.clone()))?;
        self.require_instance(&instance_id, "destroyInstance", Some(correlation.clone()))?;
        self.active = false;
        self.active_instance = None;
        self.active_io_layout = None;
        self.active_lease = None;
        self.prepared_epoch = None;
        self.prepared_sample_rate_hz = None;
        self.prepared_max_block_frames = None;
        self.last_fault = None;
        self.block_machine = SandboxStateMachine::new();
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxDestroyInstance,
            correlation,
            PluginMessagePayload::DestroyInstanceResponse {
                instance_id: instance_id.clone(),
                instance_state: PluginInstanceStatePayload {
                    plugin_type_id: self
                        .loaded_plugin
                        .as_ref()
                        .map(|plugin| plugin.plugin_type_id.0.clone())
                        .unwrap_or_default(),
                    instance_id,
                    lifecycle_state: lifecycle_state_string(PluginLifecycleState::Released).into(),
                    readiness_state: "Stopped".into(),
                    degraded_reasons: Vec::new(),
                    active: false,
                    processing: None,
                    last_fault: None,
                },
            },
        ))
    }

    pub(super) fn handle_sandbox_failure_request(
        &mut self,
        correlation: CorrelationId,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        Err(failure_event(
            self.sandbox_id.as_deref().unwrap_or("unknown"),
            self.active_instance
                .as_ref()
                .map(|instance| instance.instance_id.0.clone()),
            "failure",
            "protocolViolation",
            "sandbox received sandbox.failure as a request",
            self.prepared_epoch,
            self.active_lease
                .as_ref()
                .map(|lease| lease.lease_id.clone()),
            Some(correlation),
        ))
    }
}
