use super::*;

/// Target platform for VST3 discovery and session planning.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3HostPlatform {
    /// macOS (`Contents/MacOS`).
    MacOs,
    /// Linux (`Contents/x86_64-linux` or `Contents/aarch64-linux`).
    Linux,
    /// Windows (`Contents/x86_64-win` or `Contents/arm64-win`).
    Windows,
}

/// Classification of a VST3 scan root (user or system bundle directory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Vst3ScanRootKind {
    /// Per-user VST3 bundle root.
    UserBundleRoot,
    /// System-wide VST3 bundle root.
    SystemBundleRoot,
}

/// A filesystem root to scan for VST3 module bundles.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ScanRoot {
    /// Filesystem path of the scan root.
    pub root: String,
    /// Platform this root applies to.
    pub platform: Vst3HostPlatform,
    /// Classification of this scan root.
    pub kind: Vst3ScanRootKind,
}

/// A VST3 plugin type discovered during a scan, including its class ID, optional controller class, and default I/O layout.
#[derive(Clone, Debug, PartialEq)]
pub struct Vst3DiscoveredPluginType {
    /// Unique identifier for this plugin type.
    pub plugin_type_id: PluginTypeId,
    /// VST3 component class GUID string.
    pub class_id: String,
    /// VST3 controller class GUID string, if present.
    pub controller_class_id: Option<String>,
    /// VST3 sub-category string (e.g. `Instrument`, `Fx`).
    pub category: String,
    /// Path to the `.vst3` bundle directory.
    pub module_root: String,
    /// Full plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Default I/O layout reported at scan time.
    pub default_io_layout: PluginIoLayout,
}

/// Live control surface for a VST3 plugin instance. Binds the discovered class and controller IDs to a running instance ID.
#[derive(Clone, Debug, PartialEq)]
pub struct Vst3InstanceControlSurface {
    /// Plugin type this instance was created from.
    pub plugin_type_id: PluginTypeId,
    /// Unique identifier for this running instance.
    pub instance_id: PluginInstanceId,
    /// VST3 component class GUID string.
    pub class_id: String,
    /// VST3 controller class GUID string, if present.
    pub controller_class_id: Option<String>,
    /// VST3 sub-category string.
    pub category: String,
    /// Display name of the component class from the factory.
    pub component_name: String,
    /// Display name of the controller class from the factory, if present.
    pub controller_name: Option<String>,
    /// Number of classes exported by the module factory.
    pub factory_export_count: usize,
    /// Path to the `.vst3` bundle directory.
    pub module_root: String,
    /// Default I/O layout for this instance.
    pub default_io_layout: PluginIoLayout,
    /// Plugin descriptor including capabilities and contracts.
    pub descriptor: PluginDescriptor,
    /// Lifecycle contract for this instance.
    pub lifecycle_contract: PluginLifecycleContract,
    /// Processing contract for this instance.
    pub processing_contract: PluginProcessingContract,
}

/// Processing session plan for a VST3 instance, capturing the agreed sample rate, block size, component names, and transport.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ProcessSessionPlan {
    /// Plugin type being prepared.
    pub plugin_type_id: PluginTypeId,
    /// Instance being prepared.
    pub instance_id: PluginInstanceId,
    /// VST3 component class GUID string.
    pub class_id: String,
    /// VST3 controller class GUID string, if present.
    pub controller_class_id: Option<String>,
    /// VST3 sub-category string.
    pub category: String,
    /// Display name of the component class.
    pub component_name: String,
    /// Display name of the controller class, if present.
    pub controller_name: Option<String>,
    /// Number of classes exported by the module factory.
    pub factory_export_count: usize,
    /// Agreed sample rate in Hz.
    pub sample_rate_hz: u32,
    /// Maximum block size in frames.
    pub max_block_frames: u32,
    /// I/O layout for this session.
    pub io_layout: PluginIoLayout,
    /// Sandbox transport type for this session.
    pub transport: SandboxTransport,
    /// Path to the `.vst3` bundle directory.
    pub module_root: String,
    /// Human-readable session summary.
    pub summary: String,
}

/// Record produced when a VST3 instance is activated, confirming the sample rate, block size, and state bytes applied.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3ActivationRecord {
    /// Instance that was activated.
    pub instance_id: PluginInstanceId,
    /// Display name of the component class.
    pub component_name: String,
    /// Display name of the controller class, if present.
    pub controller_name: Option<String>,
    /// Sample rate in use after activation.
    pub sample_rate_hz: u32,
    /// Maximum block size in frames after activation.
    pub max_block_frames: u32,
    /// Number of state bytes applied at activation, or 0 if no state was loaded.
    pub state_bytes: usize,
    /// Human-readable activation summary.
    pub summary: String,
}

/// A captured state snapshot for a VST3 plugin instance, including raw bytes and a content digest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3StateSnapshot {
    /// Instance this snapshot was captured from.
    pub instance_id: PluginInstanceId,
    /// Raw serialised state bytes.
    pub bytes: Vec<u8>,
    /// Content digest of the state bytes.
    pub digest: String,
    /// Human-readable snapshot summary.
    pub summary: String,
}

/// Record produced when a VST3 instance is torn down, reporting how many state bytes were flushed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3TeardownRecord {
    /// Instance that was torn down.
    pub instance_id: PluginInstanceId,
    /// Number of state bytes flushed during teardown.
    pub flushed_state_bytes: usize,
    /// Human-readable teardown summary.
    pub summary: String,
}

/// Record produced after executing an audio block through a VST3 instance, including parameter and event ordering diagnostics.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vst3BlockProcessingRecord {
    /// Instance that processed this block.
    pub instance_id: PluginInstanceId,
    /// Block sequence number.
    pub block_sequence: u64,
    /// Number of audio frames in this block.
    pub block_frames: u32,
    /// Number of audio input channels.
    pub audio_input_channels: u16,
    /// Number of audio output channels.
    pub audio_output_channels: u16,
    /// Number of MIDI input ports.
    pub midi_input_ports: u16,
    /// Number of MIDI output ports.
    pub midi_output_ports: u16,
    /// Number of parameter events applied in this block.
    pub parameter_event_count: u16,
    /// Number of MIDI events applied in this block.
    pub midi_event_count: u16,
    /// Completion status string.
    pub completion_status: &'static str,
    /// Digest of the state snapshot active when this block was processed, if any.
    pub state_digest: Option<String>,
    /// Signature of the parameter events applied in this block.
    pub parameter_signature: String,
    /// Describes the order in which parameter events were applied.
    pub parameter_application_order: String,
    /// Describes the order in which event packets were processed.
    pub event_packet_order: String,
    /// Describes the automation delta for this block.
    pub automation_delta: String,
    /// Describes the state transition that occurred in this block.
    pub state_transition: String,
    /// Digest of the state after this block was processed.
    pub next_state_digest: String,
    /// Human-readable block processing summary.
    pub summary: String,
}
