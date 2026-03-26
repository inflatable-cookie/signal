use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuHostPlatform {
    MacOs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AuScanRootKind {
    UserComponentRoot,
    SystemComponentRoot,
    BuiltInComponentRoot,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuScanRoot {
    pub root: String,
    pub platform: AuHostPlatform,
    pub kind: AuScanRootKind,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuDiscoveredPluginType {
    pub plugin_type_id: PluginTypeId,
    pub component_type: String,
    pub component_subtype: String,
    pub manufacturer_code: String,
    pub bundle_root: String,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuInstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub component_type: String,
    pub component_subtype: String,
    pub manufacturer_code: String,
    pub bundle_root: String,
    pub default_io_layout: PluginIoLayout,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuProcessSessionPlan {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub component_type: String,
    pub component_subtype: String,
    pub manufacturer_code: String,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub bundle_root: String,
    pub summary: String,
}
