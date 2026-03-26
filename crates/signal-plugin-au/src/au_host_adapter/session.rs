use super::*;

impl AuHostAdapter {
    pub fn instantiate_plugin(
        &self,
        discovered: &AuDiscoveredPluginType,
        instance_id: &str,
    ) -> AuInstanceControlSurface {
        AuInstanceControlSurface {
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
        }
    }

    pub fn prepare_session(
        &self,
        instance: &AuInstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> AuProcessSessionPlan {
        AuProcessSessionPlan {
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
        }
    }
}
