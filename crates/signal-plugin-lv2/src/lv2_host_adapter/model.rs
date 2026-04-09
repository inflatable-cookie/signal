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
    pub prepare_fault: Option<Lv2PreparationFaultMode>,
    pub descriptor: PluginDescriptor,
    pub default_io_layout: PluginIoLayout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2PreparationFaultMode {
    WorkerUnavailable,
    UridUnavailable,
    PatchUnavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2DiscoveryDiagnosticKind {
    MalformedManifest,
    UnsupportedRequiredFeature,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2DiscoveryDiagnostic {
    pub root: String,
    pub bundle_root: String,
    pub manifest_path: Option<String>,
    pub plugin_type_id: Option<String>,
    pub kind: Lv2DiscoveryDiagnosticKind,
    pub detail: String,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lv2DiscoveryBatch {
    pub discovered: Vec<Lv2DiscoveredPluginType>,
    pub diagnostics: Vec<Lv2DiscoveryDiagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lv2InstanceControlSurface {
    pub plugin_type_id: PluginTypeId,
    pub instance_id: PluginInstanceId,
    pub plugin_uri: String,
    pub bundle_root: String,
    pub manifest_path: String,
    pub required_features: Vec<String>,
    pub prepare_fault: Option<Lv2PreparationFaultMode>,
    pub default_io_layout: PluginIoLayout,
    pub descriptor: PluginDescriptor,
    pub lifecycle_contract: PluginLifecycleContract,
    pub processing_contract: PluginProcessingContract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2WorkerNegotiationPosture {
    WorkerAbsent,
    WorkerAvailable,
    WorkerRequiredAvailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2UridNegotiationPosture {
    NotRequired,
    Negotiated,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2PatchExchangePosture {
    Absent,
    Supported,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lv2ExtensionNegotiationState {
    NotRequired,
    Negotiated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2ExtensionPreparationRecord {
    pub worker_posture: Lv2WorkerNegotiationPosture,
    pub urid_negotiation_posture: Lv2UridNegotiationPosture,
    pub patch_exchange_posture: Lv2PatchExchangePosture,
    pub extension_negotiation_state: Lv2ExtensionNegotiationState,
    pub summary: String,
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
    pub extension_preparation: Lv2ExtensionPreparationRecord,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2TeardownRecord {
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lv2BlockProcessingRecord {
    pub block_sequence: u64,
    pub block_frames: u32,
    pub audio_output_channels: u16,
    pub midi_event_count: u16,
    pub patch_message_count: u16,
    pub completion_status: String,
    pub summary: String,
}
