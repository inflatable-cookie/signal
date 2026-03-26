use super::*;

impl Vst3HostAdapter {
    pub fn instantiate_plugin(
        &self,
        discovered: &Vst3DiscoveredPluginType,
        instance_id: &str,
    ) -> Vst3InstanceControlSurface {
        Vst3InstanceControlSurface {
            plugin_type_id: discovered.plugin_type_id.clone(),
            instance_id: PluginInstanceId(instance_id.to_string()),
            class_id: discovered.class_id.clone(),
            controller_class_id: discovered.controller_class_id.clone(),
            module_root: discovered.module_root.clone(),
            default_io_layout: discovered.default_io_layout,
            descriptor: discovered.descriptor.clone(),
            lifecycle_contract: discovered.descriptor.lifecycle_contract,
            processing_contract: discovered.descriptor.processing_contract,
        }
    }

    pub fn prepare_session(
        &self,
        instance: &Vst3InstanceControlSurface,
        sample_rate_hz: u32,
        max_block_frames: u32,
    ) -> Vst3ProcessSessionPlan {
        Vst3ProcessSessionPlan {
            plugin_type_id: instance.plugin_type_id.clone(),
            instance_id: instance.instance_id.clone(),
            class_id: instance.class_id.clone(),
            controller_class_id: instance.controller_class_id.clone(),
            sample_rate_hz,
            max_block_frames,
            io_layout: instance.default_io_layout,
            transport: SandboxTransport::SharedMemory,
            module_root: instance.module_root.clone(),
            summary: format!(
                "plugin_type={} class={} controller={} sample_rate={} max_block_frames={} module_root={}",
                instance.plugin_type_id.0,
                instance.class_id,
                instance.controller_class_id.as_deref().unwrap_or("none"),
                sample_rate_hz,
                max_block_frames,
                instance.module_root,
            ),
        }
    }
}
