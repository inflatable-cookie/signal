use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3HostPlatform {
    MacOs,
    Linux,
    Windows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3ScanRootKind {
    UserBundleRoot,
    SystemBundleRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ScanRoot {
    pub root: String,
    pub platform: Vst3HostPlatform,
    pub kind: Vst3ScanRootKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vst3DiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub class_id: String,
    pub controller_class_id: Option<String>,
    pub category: String,
    pub module_root: String,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Vst3InstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub class_id: String,
    pub controller_class_id: Option<String>,
    pub module_root: String,
    pub default_io_layout: PluginIoLayout,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ProcessSessionPlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub class_id: String,
    pub controller_class_id: Option<String>,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub module_root: String,
    pub summary: String,
}
