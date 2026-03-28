mod builders;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFormat {
    Clap,
    Vst3,
    Au,
    Lv2,
    Native,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxPolicy {
    Strict,
    Moderate,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginTypeId(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFeature {
    AudioEffect,
    Instrument,
    Analyzer,
    Utility,
    NoteEffect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginAudioBusDirection {
    Input,
    Output,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginAudioBusDescriptor {
    pub bus_id: String,
    pub name: String,
    pub direction: PluginAudioBusDirection,
    pub channels: u16,
    pub is_main: bool,
    pub active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginParameterDomain {
    GenericNormalized,
    Decibels,
    Hertz,
    Seconds,
    Bypass,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginParameterFlags {
    pub automatable: bool,
    pub modulatable: bool,
    pub supports_gesture: bool,
    pub stepped: bool,
    pub hidden: bool,
    pub read_only: bool,
}

impl PluginParameterFlags {
    pub fn automatable() -> Self {
        Self {
            automatable: true,
            modulatable: true,
            supports_gesture: true,
            stepped: false,
            hidden: false,
            read_only: false,
        }
    }

    pub fn bypass() -> Self {
        Self {
            automatable: true,
            modulatable: false,
            supports_gesture: false,
            stepped: true,
            hidden: false,
            read_only: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginParameterDescriptor {
    pub parameter_id: u32,
    pub name: String,
    pub unit: Option<String>,
    pub domain: PluginParameterDomain,
    pub default_normalized: f32,
    pub min_plain: f32,
    pub max_plain: f32,
    pub flags: PluginParameterFlags,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginStateContract {
    pub supports_snapshot: bool,
    pub supports_reset: bool,
    pub supports_bypass: bool,
    pub exposes_latency: bool,
    pub exposes_tail: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessingContract {
    pub max_block_frames: u32,
    pub sample_accurate_automation: bool,
    pub accepts_midi: bool,
    pub accepts_note_events: bool,
    pub supports_note_expression: bool,
    pub produces_midi: bool,
    pub silence_aware: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginLifecycleContract {
    pub requires_main_thread_for_state: bool,
    pub supports_prepare: bool,
    pub supports_activate: bool,
    pub supports_reset_while_active: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginLifecycleState {
    Discovered,
    TypeLoaded,
    InstanceCreated,
    Prepared,
    Active,
    Inactive,
    Released,
    Faulted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessConfiguration {
    pub sample_rate_hz: u32,
    pub max_block_frames: u32,
    pub io_layout: PluginIoLayout,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PluginDescriptor {
    pub plugin_id: String,
    pub vendor: String,
    pub name: String,
    pub format: PluginFormat,
    pub version: Option<String>,
    pub features: Vec<PluginFeature>,
    pub audio_buses: Vec<PluginAudioBusDescriptor>,
    pub parameters: Vec<PluginParameterDescriptor>,
    pub state_contract: PluginStateContract,
    pub processing_contract: PluginProcessingContract,
    pub lifecycle_contract: PluginLifecycleContract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginIoLayout {
    pub audio_inputs: u16,
    pub audio_outputs: u16,
    pub midi_inputs: u16,
    pub midi_outputs: u16,
}
