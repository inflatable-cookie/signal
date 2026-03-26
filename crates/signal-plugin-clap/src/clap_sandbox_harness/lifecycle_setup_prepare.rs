use signal_ipc::{
    CorrelationId, PluginIoLayoutPayload, PluginMessageEnvelope, PluginMessageName,
    PluginMessagePayload, SharedMemoryLayoutPayload, SharedMemoryTransportPayload,
};
use signal_plugin::{PluginLifecycleState, SandboxStateMachine, SharedMemoryLease};

use super::failure::failure_event;
use super::state::ClapSandboxLifecycleHarness;

impl ClapSandboxLifecycleHarness {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn handle_prepare_instance(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
        shared_memory_lease_id: String,
        shared_memory_transport: SharedMemoryTransportPayload,
        sample_rate_hz: u32,
        max_block_frames: u32,
        io_layout: PluginIoLayoutPayload,
        shared_memory: SharedMemoryLayoutPayload,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "prepareInstance", Some(correlation.clone()))?;
        self.require_instance(&instance_id, "prepareInstance", Some(correlation.clone()))?;
        let instance = self.active_instance.as_ref().expect("validated instance");
        if !instance.lifecycle_contract.supports_prepare {
            return Err(failure_event(
                &sandbox_id,
                Some(instance_id.clone()),
                "prepareInstance",
                "unsupported",
                "loaded CLAP instance does not support prepare",
                Some(processing_epoch),
                Some(shared_memory_lease_id.clone()),
                Some(correlation),
            ));
        }
        if max_block_frames > instance.processing_contract.max_block_frames {
            return Err(failure_event(
                &sandbox_id,
                Some(instance_id.clone()),
                "prepareInstance",
                "resourceUnavailable",
                "requested block size exceeds discovered CLAP processing contract",
                Some(processing_epoch),
                Some(shared_memory_lease_id.clone()),
                Some(correlation),
            ));
        }
        let attached = self
            .broker
            .attach_region(&shared_memory_transport)
            .map_err(|error| {
                failure_event(
                    &sandbox_id,
                    Some(instance_id.clone()),
                    "prepareInstance",
                    "resourceUnavailable",
                    format!("failed to attach shared-memory region: {error}"),
                    Some(processing_epoch),
                    Some(shared_memory_lease_id.clone()),
                    Some(correlation.clone()),
                )
            })?;
        if attached.total_bytes() != shared_memory.total_bytes() {
            return Err(failure_event(
                &sandbox_id,
                Some(instance_id),
                "prepareInstance",
                "protocolViolation",
                "shared-memory transport size does not match negotiated layout",
                Some(processing_epoch),
                Some(shared_memory_lease_id),
                Some(correlation),
            ));
        }
        self.prepared_epoch = Some(processing_epoch);
        self.prepared_sample_rate_hz = Some(sample_rate_hz);
        self.prepared_max_block_frames = Some(max_block_frames);
        self.active = false;
        self.last_fault = None;
        self.active_io_layout = Some(crate::io_layout_from_payload(io_layout));
        self.block_machine = SandboxStateMachine::new();
        self.active_lease = Some(
            SharedMemoryLease::new(
                shared_memory_lease_id.clone(),
                processing_epoch,
                crate::shared_memory_layout(shared_memory),
            )
            .with_transport(shared_memory_transport.clone()),
        );
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxPrepareInstance,
            correlation,
            PluginMessagePayload::PrepareInstanceResponse {
                instance_id: instance_id.clone(),
                processing_epoch,
                shared_memory_lease_id,
                shared_memory_transport,
                shared_memory_bytes: shared_memory.total_bytes(),
                instance_state: self
                    .instance_state_payload(&instance_id, PluginLifecycleState::Prepared)
                    .expect("instance state after prepare"),
            },
        ))
    }

    pub(super) fn handle_activate_instance(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        instance_id: String,
        processing_epoch: u64,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "activateInstance", Some(correlation.clone()))?;
        self.require_instance(&instance_id, "activateInstance", Some(correlation.clone()))?;
        if !self
            .active_instance
            .as_ref()
            .expect("validated instance")
            .lifecycle_contract
            .supports_activate
        {
            return Err(failure_event(
                &sandbox_id,
                Some(instance_id),
                "activateInstance",
                "unsupported",
                "loaded CLAP instance does not support activate",
                Some(processing_epoch),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                Some(correlation),
            ));
        }
        if self.prepared_epoch != Some(processing_epoch) {
            if let Some(lease) = &mut self.active_lease {
                lease.invalidate_epoch(processing_epoch);
            }
            return Err(failure_event(
                &sandbox_id,
                Some(instance_id),
                "activateInstance",
                "protocolViolation",
                "activate requested with epoch that is not prepared",
                Some(processing_epoch),
                self.active_lease
                    .as_ref()
                    .map(|lease| lease.lease_id.clone()),
                Some(correlation),
            ));
        }
        self.active = true;
        self.last_fault = None;
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxActivateInstance,
            correlation,
            PluginMessagePayload::ActivateInstanceResponse {
                instance_id: instance_id.clone(),
                processing_epoch,
                instance_state: self
                    .instance_state_payload(&instance_id, PluginLifecycleState::Active)
                    .expect("instance state after activate"),
            },
        ))
    }

    pub(super) fn handle_heartbeat(
        &mut self,
        correlation: CorrelationId,
        sandbox_id: String,
        instance_id: Option<String>,
        processing_epoch: Option<u64>,
    ) -> Result<PluginMessageEnvelope, PluginMessageEnvelope> {
        self.require_sandbox(&sandbox_id, "heartbeat", Some(correlation.clone()))?;
        if let Some(instance_id) = instance_id.as_deref() {
            self.require_instance(instance_id, "heartbeat", Some(correlation.clone()))?;
        }
        self.heartbeat_count = self.heartbeat_count.saturating_add(1);
        Ok(PluginMessageEnvelope::response(
            PluginMessageName::SandboxHeartbeat,
            correlation,
            PluginMessagePayload::HeartbeatResponse {
                sandbox_id,
                instance_id: self
                    .active_instance
                    .as_ref()
                    .map(|instance| instance.instance_id.0.clone()),
                processing_epoch: processing_epoch.or(self.prepared_epoch),
                active: self.active,
                instance_state: self.active_instance.as_ref().and_then(|instance| {
                    self.instance_state_payload(
                        instance.instance_id.0.as_str(),
                        self.current_lifecycle_state()
                            .unwrap_or(PluginLifecycleState::Discovered),
                    )
                }),
            },
        ))
    }
}
