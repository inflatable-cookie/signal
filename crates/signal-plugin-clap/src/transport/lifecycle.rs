use std::io;

use signal_ipc::{
    PluginMessageEnvelope, PluginMessageName, PluginMessagePayload, SharedMemoryBroker,
};
use signal_plugin::SharedMemoryLease;

use crate::adapter::ClapPreparePlan;
use crate::clap_sandbox_harness;
use crate::event_translation::{io_layout_payload, shared_memory_layout_payload};
use crate::protocol::ClapBlockProtocol;

impl ClapBlockProtocol {
    /// Allocates a shared-memory region and returns a [`ClapPreparePlan`] for the given sandbox and epoch.
    pub fn prepare_plan(
        &self,
        broker: &SharedMemoryBroker,
        sandbox_id: &str,
        sample_rate_hz: u32,
        max_block_frames: u32,
        processing_epoch: u64,
    ) -> io::Result<ClapPreparePlan> {
        let layout = self.shared_memory_layout(max_block_frames);
        let region = broker.create_region(
            &format!(
                "{sandbox_id}:{}:epoch-{processing_epoch}",
                self.instance_id.0
            ),
            layout.total_bytes(),
        )?;
        let transport = region.metadata().clone();

        Ok(ClapPreparePlan {
            plugin_type_id: self.plugin_type_id.clone(),
            instance_id: self.instance_id.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: self.io_layout,
            lease: SharedMemoryLease::new(
                format!(
                    "{sandbox_id}:{}:epoch-{processing_epoch}",
                    self.instance_id.0
                ),
                processing_epoch,
                layout,
            )
            .with_transport(transport),
        })
    }

    /// Builds the full lifecycle message sequence (handshake → load → create → prepare → activate) for this plugin.
    pub fn lifecycle_sequence(
        &self,
        broker: &SharedMemoryBroker,
        sandbox_id: &str,
        sample_rate_hz: u32,
        max_block_frames: u32,
        processing_epoch: u64,
    ) -> io::Result<Vec<PluginMessageEnvelope>> {
        let descriptor = self.descriptor();
        let prepare = self.prepare_plan(
            broker,
            sandbox_id,
            sample_rate_hz,
            max_block_frames,
            processing_epoch,
        )?;
        let transport = prepare
            .lease
            .transport()
            .cloned()
            .expect("brokered transport");

        Ok(vec![
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxHandshake,
                format!("{sandbox_id}:handshake"),
                PluginMessagePayload::SandboxHandshakeRequest {
                    sandbox_id: sandbox_id.into(),
                    format: "clap".into(),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxLoadPluginType,
                format!("{sandbox_id}:load"),
                PluginMessagePayload::LoadPluginTypeRequest {
                    sandbox_id: sandbox_id.into(),
                    plugin_type_id: self.plugin_type_id.0.clone(),
                    descriptor: clap_sandbox_harness::descriptor_payload(&descriptor),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxCreateInstance,
                format!("{sandbox_id}:create"),
                PluginMessagePayload::CreateInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    plugin_type_id: self.plugin_type_id.0.clone(),
                    instance_id: self.instance_id.0.clone(),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxPrepareInstance,
                format!("{sandbox_id}:prepare"),
                PluginMessagePayload::PrepareInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                    processing_epoch,
                    shared_memory_lease_id: prepare.lease.lease_id.clone(),
                    shared_memory_transport: transport.clone(),
                    sample_rate_hz,
                    max_block_frames,
                    io_layout: io_layout_payload(prepare.io_layout),
                    shared_memory: shared_memory_layout_payload(prepare.lease.layout),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxActivateInstance,
                format!("{sandbox_id}:activate"),
                PluginMessagePayload::ActivateInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                    processing_epoch,
                },
            ),
        ])
    }

    /// Builds a heartbeat request envelope for the given sandbox and optional processing epoch.
    pub fn heartbeat_request(
        &self,
        sandbox_id: &str,
        processing_epoch: Option<u64>,
    ) -> PluginMessageEnvelope {
        PluginMessageEnvelope::command(
            PluginMessageName::SandboxHeartbeat,
            format!("{sandbox_id}:heartbeat"),
            PluginMessagePayload::HeartbeatRequest {
                sandbox_id: sandbox_id.into(),
                instance_id: Some(self.instance_id.0.clone()),
                processing_epoch,
            },
        )
    }

    /// Builds the teardown message sequence (deactivate → reset → destroy) for this plugin instance.
    pub fn teardown_sequence(
        &self,
        sandbox_id: &str,
        processing_epoch: u64,
    ) -> Vec<PluginMessageEnvelope> {
        vec![
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxDeactivateInstance,
                format!("{sandbox_id}:deactivate"),
                PluginMessagePayload::DeactivateInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxResetInstance,
                format!("{sandbox_id}:reset"),
                PluginMessagePayload::ResetInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                    processing_epoch,
                },
            ),
            PluginMessageEnvelope::command(
                PluginMessageName::SandboxDestroyInstance,
                format!("{sandbox_id}:destroy"),
                PluginMessagePayload::DestroyInstanceRequest {
                    sandbox_id: sandbox_id.into(),
                    instance_id: self.instance_id.0.clone(),
                },
            ),
        ]
    }
}
