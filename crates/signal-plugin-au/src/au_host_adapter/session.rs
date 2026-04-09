use super::*;

impl AuHostAdapter {
    pub fn instantiate_plugin(
        &self,
        discovered: &AuDiscoveredPluginType,
        instance_id: &str,
    ) -> Result<AuInstanceControlSurface, String> {
        if let Some(detail) = discovered.failure_contract.init_failure.as_ref() {
            return Err(format!(
                "au initialization failed for {}: {}",
                discovered.plugin_type_id.0, detail
            ));
        }

        Ok(AuInstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            component_type: discovered.component_type.clone(),
            component_subtype: discovered.component_subtype.clone(),
            manufacturer_code: discovered.manufacturer_code.clone(),
            bundle_root: discovered.bundle_root.clone(),
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
            failure_contract: discovered.failure_contract.clone(),
        })
    }

    pub fn prepare_session(
        &self,
        instance: &AuInstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> Result<AuProcessSessionPlan, String> {
        if let Some(detail) = instance.failure_contract.bus_layout_failure.as_ref() {
            return Err(format!(
                "au bus layout negotiation failed for {}: {}",
                instance.plugin_type_id.0, detail
            ));
        }

        Ok(AuProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            component_type: instance.component_type.clone(),
            component_subtype: instance.component_subtype.clone(),
            manufacturer_code: instance.manufacturer_code.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            bundle_root: instance.bundle_root.clone(),
            summary: format!(
                "plugin_type={} component_type={} component_subtype={} manufacturer={} sample_rate={} max_block_frames={} bundle_root={}",
                instance.plugin_type_id.0,
                instance.component_type,
                instance.component_subtype,
                instance.manufacturer_code,
                sample_rate_hz,
                max_block_frames,
                instance.bundle_root,
            ),
        })
    }

    pub fn store_state_snapshot(&self, instance: &AuInstanceControlSurface) -> AuStateSnapshot {
        let bytes = format!(
            "plugin_type={} instance={} component_type={} component_subtype={} manufacturer={} bundle_root={}",
            instance.plugin_type_id.0,
            instance.instance_id.0,
            instance.component_type,
            instance.component_subtype,
            instance.manufacturer_code,
            instance.bundle_root,
        )
        .into_bytes();
        let state_bytes = bytes.len();
        let digest = format!(
            "au-state:{}:{}:{}:{}",
            instance.plugin_type_id.0,
            instance.component_type,
            instance.component_subtype,
            state_bytes,
        );
        AuStateSnapshot {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            component_type: instance.component_type.clone(),
            component_subtype: instance.component_subtype.clone(),
            manufacturer_code: instance.manufacturer_code.clone(),
            bytes,
            digest: digest.clone(),
            summary: format!(
                "instance={} state_stored=1 state_bytes={} state_digest={} component={} subtype={} manufacturer={}",
                instance.instance_id.0,
                state_bytes,
                digest,
                instance.descriptor.name,
                instance.component_subtype,
                instance.manufacturer_code,
            ),
        }
    }

    pub fn activate_instance(
        &self,
        instance: &AuInstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
        state: Option<&AuStateSnapshot>,
    ) -> Result<AuActivationRecord, String> {
        if let Some(detail) = instance.failure_contract.render_context_failure.as_ref() {
            return Err(format!(
                "au render context activation failed for {}: {}",
                instance.plugin_type_id.0, detail
            ));
        }

        Ok(AuActivationRecord {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            component_type: instance.component_type.clone(),
            component_subtype: instance.component_subtype.clone(),
            manufacturer_code: instance.manufacturer_code.clone(),
            sample_rate_hz,
            max_block_frames,
            summary: format!(
                "activation=ready component={} component_type={} component_subtype={} manufacturer={} sample_rate={} max_block_frames={} state_digest={}",
                instance.descriptor.name,
                instance.component_type,
                instance.component_subtype,
                instance.manufacturer_code,
                sample_rate_hz,
                max_block_frames,
                state.map(|snapshot| snapshot.digest.as_str()).unwrap_or("none"),
            ),
        })
    }

    pub fn teardown_instance(
        &self,
        instance: &AuInstanceControlSurface,
        state: Option<&AuStateSnapshot>,
    ) -> AuTeardownRecord {
        let flushed_state_bytes = state.map(|snapshot| snapshot.bytes.len()).unwrap_or(0);
        AuTeardownRecord {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            component_type: instance.component_type.clone(),
            component_subtype: instance.component_subtype.clone(),
            manufacturer_code: instance.manufacturer_code.clone(),
            flushed_state_bytes,
            summary: format!(
                "flushed_state_bytes={} suspended=1 component={} component_type={} component_subtype={} manufacturer={}",
                flushed_state_bytes,
                instance.descriptor.name,
                instance.component_type,
                instance.component_subtype,
                instance.manufacturer_code,
            ),
        }
    }
}
