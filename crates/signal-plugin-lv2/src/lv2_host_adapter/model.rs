use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2HostPlatform {
    Linux,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2ScanRootKind {
    UserBundleRoot,
    SystemBundleRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ScanRoot {
    pub root: String,
    pub platform: Lv2HostPlatform,
    pub kind: Lv2ScanRootKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lv2DiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub plugin_uri: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub required_features: Vec<String>,
    pub supported_extensions: Vec<String>,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lv2InstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub plugin_uri: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub required_features: Vec<String>,
    pub default_io_layout: PluginIoLayout,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ProcessSessionPlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub plugin_uri: String,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub bundle_root: String,
    pub manifest_path: String,
    pub summary: String,
}
