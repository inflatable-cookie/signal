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
    pub category: String,
    pub component_name: String,
    pub controller_name: Option<String>,
    pub factory_export_count: usize,
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
    pub category: String,
    pub component_name: String,
    pub controller_name: Option<String>,
    pub factory_export_count: usize,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
    pub transport: SandboxTransport,
    pub module_root: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ActivationRecord {
    pub instance_id: PluginInstanceId,
    pub component_name: String,
    pub controller_name: Option<String>,
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub state_bytes: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3StateSnapshot {
    pub instance_id: PluginInstanceId,
    pub bytes: Vec<u8>,
    pub digest: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3TeardownRecord {
    pub instance_id: PluginInstanceId,
    pub flushed_state_bytes: usize,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3BlockProcessingRecord {
    pub instance_id: PluginInstanceId,
    pub block_sequence: u64,
    pub block_frames: u32,
    pub audio_input_channels: u16,
    pub audio_output_channels: u16,
    pub midi_input_ports: u16,
    pub midi_output_ports: u16,
    pub parameter_event_count: u16,
    pub midi_event_count: u16,
    pub completion_status: &'static str,
    pub state_digest: Option<String>,
    pub parameter_signature: String,
    pub parameter_application_order: String,
    pub event_packet_order: String,
    pub automation_delta: String,
    pub state_transition: String,
    pub next_state_digest: String,
    pub summary: String,
}
