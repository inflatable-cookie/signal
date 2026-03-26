use super::*;

impl Lv2HostAdapter {
    pub fn instantiate_plugin(
        &self,
        discovered: &Lv2DiscoveredPluginType,
        instance_id: &str,
    ) -> Lv2InstanceControlSurface {
        Lv2InstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            plugin_uri: discovered.plugin_uri.clone(),
            bundle_root: discovered.bundle_root.clone(),
            manifest_path: discovered.manifest_path.clone(),
            required_features: discovered.required_features.clone(),
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }

    pub fn prepare_session(
        &self,
        instance: &Lv2InstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> Lv2ProcessSessionPlan {
        Lv2ProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            plugin_uri: instance.plugin_uri.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            bundle_root: instance.bundle_root.clone(),
            manifest_path: instance.manifest_path.clone(),
            summary: format!(
                "plugin_type={} uri={} sample_rate={} max_block_frames={} bundle_root={} manifest={} required_features={}",
                instance.plugin_type_id.0,
                instance.plugin_uri,
                sample_rate_hz,
                max_block_frames,
                instance.bundle_root,
                instance.manifest_path,
                instance.required_features.join(","),
            ),
        }
    }
}
