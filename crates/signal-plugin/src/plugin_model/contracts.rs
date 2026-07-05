mod builders;

/// The plugin API format a plugin was built against.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFormat {
    /// CLAP (CLever Audio Plugin).
    Clap,
    /// VST3.
    Vst3,
    /// Audio Unit (macOS/iOS).
    Au,
    /// LV2.
    Lv2,
    /// Signal-native plugin format.
    Native,
}

/// How aggressively the sandbox isolates a plugin process from the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SandboxPolicy {
    /// Full OS-level isolation; maximum protection.
    Strict,
    /// Partial isolation; balances compatibility with protection.
    Moderate,
    /// No sandbox; plugin runs in the host process.
    Disabled,
}

/// User-chosen isolation tier for a hosted plugin instance (g11.012).
///
/// Isolation is configuration, not architecture: a global host preference
/// with per-plugin overrides selects how much process isolation a plugin
/// gets, trading crash protection against per-block transport overhead.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginIsolationTier {
    /// Plugin code runs in the host process; `process()` is a direct FFI
    /// call on the audio thread. No crash isolation — a plugin crash takes
    /// the host down. Lowest overhead.
    InProcess,
    /// One shared sandbox process hosts many plugin instances. Crash
    /// isolation from the host, shared blast radius between plugins.
    /// Modeled now, unimplemented in v1: instantiation returns a typed
    /// rejection.
    SharedSandbox,
    /// One sandbox process per plugin instance (v1 default): full crash
    /// isolation, one shm round-trip of overhead per block.
    DedicatedSandbox,
}

/// Format-specific string identifier for a plugin type (e.g. a CLAP plugin ID).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginTypeId(pub String);

/// Broad functional category of a plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginFeature {
    /// Processes audio (e.g. compressor, reverb).
    AudioEffect,
    /// Generates audio from note input (e.g. synthesizer, sampler).
    Instrument,
    /// Analyses audio without modifying it (e.g. spectrum analyser).
    Analyzer,
    /// Provides host-facing utility functions (e.g. tuner, meter).
    Utility,
    /// Transforms note/MIDI events without producing audio (e.g. arpeggiator).
    NoteEffect,
}

/// Whether an audio bus carries signal into or out of the plugin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginAudioBusDirection {
    /// Audio flows from the host into the plugin.
    Input,
    /// Audio flows from the plugin to the host.
    Output,
}

/// Describes a single audio bus exposed by a plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginAudioBusDescriptor {
    /// Unique identifier for this bus within the plugin.
    pub bus_id: String,
    /// Human-readable name (e.g. "Main Input", "Sidechain").
    pub name: String,
    /// Signal flow direction.
    pub direction: PluginAudioBusDirection,
    /// Number of channels on this bus.
    pub channels: u16,
    /// `true` if this is the plugin's primary bus for its direction.
    pub is_main: bool,
    /// `true` if the bus is currently active and should be connected.
    pub active: bool,
}

/// Semantic type of a parameter's value range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginParameterDomain {
    /// Unitless value mapped linearly to \[0.0, 1.0\].
    GenericNormalized,
    /// Decibel scale (e.g. gain or level).
    Decibels,
    /// Frequency in hertz.
    Hertz,
    /// Duration in seconds.
    Seconds,
    /// Boolean bypass control (0 = active, 1 = bypassed).
    Bypass,
}

/// Capability flags for a plugin parameter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginParameterFlags {
    /// Host may record and play back automation for this parameter.
    pub automatable: bool,
    /// Host may apply real-time modulation offsets to this parameter.
    pub modulatable: bool,
    /// Plugin supports begin/end gesture events for this parameter.
    pub supports_gesture: bool,
    /// Parameter takes discrete integer steps rather than continuous values.
    pub stepped: bool,
    /// Parameter should not be shown in generic host UI.
    pub hidden: bool,
    /// Host must not write values to this parameter.
    pub read_only: bool,
}

impl PluginParameterFlags {
    /// Returns flags suitable for a standard automatable, modulatable, gesture-supporting parameter.
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

    /// Returns flags suitable for a bypass parameter (automatable, stepped, no modulation).
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

/// Full description of a single plugin parameter, including its range and capabilities.
#[derive(Clone, Debug, PartialEq)]
pub struct PluginParameterDescriptor {
    /// Stable numeric identifier used in all event and automation messages.
    pub parameter_id: u32,
    /// Human-readable parameter name.
    pub name: String,
    /// Optional unit label (e.g. "dB", "Hz"); `None` for unitless parameters.
    pub unit: Option<String>,
    /// Semantic type of the value range.
    pub domain: PluginParameterDomain,
    /// Default value in the \[0.0, 1.0\] normalized range.
    pub default_normalized: f32,
    /// Minimum value in the plugin's native (plain) range.
    pub min_plain: f32,
    /// Maximum value in the plugin's native (plain) range.
    pub max_plain: f32,
    /// Number of discrete steps across \[`min_plain`, `max_plain`\]
    /// (g12.013): `Some(1)` for a toggle, `Some(n)` for an n-step integer
    /// or enumeration range, `None` for continuous parameters. When set,
    /// the parameter takes `step_count + 1` distinct values.
    pub step_count: Option<u32>,
    /// Capability flags for this parameter.
    pub flags: PluginParameterFlags,
}

impl PluginParameterDescriptor {
    /// Whether the host may record and play back automation for this
    /// parameter (the descriptor-vocabulary view of `flags.automatable`).
    pub fn is_automatable(&self) -> bool {
        self.flags.automatable
    }

    /// Whether this parameter is the plugin's bypass control (the
    /// descriptor-vocabulary view of `domain == Bypass`).
    pub fn is_bypass(&self) -> bool {
        self.domain == PluginParameterDomain::Bypass
    }
}

/// Declares which state operations a plugin supports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginStateContract {
    /// Plugin can save and restore its full state as a byte blob.
    pub supports_snapshot: bool,
    /// Plugin can reset itself to factory defaults.
    pub supports_reset: bool,
    /// Plugin exposes a bypass parameter.
    pub supports_bypass: bool,
    /// Plugin reports its processing latency in frames.
    pub exposes_latency: bool,
    /// Plugin reports its tail length in frames.
    pub exposes_tail: bool,
}

/// Declares the plugin's audio and event processing capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessingContract {
    /// Maximum number of frames the plugin can process in a single block.
    pub max_block_frames: u32,
    /// Plugin handles per-frame parameter changes within a block accurately.
    pub sample_accurate_automation: bool,
    /// Plugin accepts raw MIDI 1.0 messages.
    pub accepts_midi: bool,
    /// Plugin accepts structured note-on/off events.
    pub accepts_note_events: bool,
    /// Plugin accepts per-note expression events.
    pub supports_note_expression: bool,
    /// Plugin may produce MIDI output.
    pub produces_midi: bool,
    /// Plugin can detect and communicate silence to the host.
    pub silence_aware: bool,
}

/// Declares the plugin's lifecycle and threading requirements.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginLifecycleContract {
    /// State operations (snapshot, reset) must be called on the main thread.
    pub requires_main_thread_for_state: bool,
    /// Plugin implements a prepare phase before activation.
    pub supports_prepare: bool,
    /// Plugin implements explicit activate/deactivate transitions.
    pub supports_activate: bool,
    /// Plugin can be reset without deactivating first.
    pub supports_reset_while_active: bool,
}

/// Current lifecycle phase of a plugin instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginLifecycleState {
    /// Plugin binary has been found but not yet loaded.
    Discovered,
    /// Plugin type has been loaded and its descriptor is available.
    TypeLoaded,
    /// An instance has been created but not yet prepared for processing.
    InstanceCreated,
    /// Instance has been prepared with a sample rate and block size.
    Prepared,
    /// Instance is active and processing audio.
    Active,
    /// Instance has been deactivated; can be re-activated or released.
    Inactive,
    /// Instance has been destroyed.
    Released,
    /// Instance encountered a fault and cannot continue.
    Faulted,
}

/// The sample rate and block configuration used when activating a plugin instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginProcessConfiguration {
    /// Sample rate in Hz (e.g. 44100, 48000).
    pub sample_rate_hz: u32,
    /// Maximum number of frames per processing block.
    pub max_block_frames: u32,
    /// Audio and MIDI bus counts for this activation.
    pub io_layout: PluginIoLayout,
}

/// Complete static description of a plugin type as reported by its format adapter.
#[derive(Clone, Debug, PartialEq)]
pub struct PluginDescriptor {
    /// Unique string identifier for this plugin type within its format.
    pub plugin_id: String,
    /// Plugin vendor / manufacturer name.
    pub vendor: String,
    /// Plugin display name.
    pub name: String,
    /// Plugin format.
    pub format: PluginFormat,
    /// Version string (format-defined; `None` if not provided).
    pub version: Option<String>,
    /// Functional categories this plugin belongs to.
    pub features: Vec<PluginFeature>,
    /// All audio buses the plugin exposes.
    pub audio_buses: Vec<PluginAudioBusDescriptor>,
    /// All parameters the plugin exposes.
    pub parameters: Vec<PluginParameterDescriptor>,
    /// State capabilities.
    pub state_contract: PluginStateContract,
    /// Processing capabilities.
    pub processing_contract: PluginProcessingContract,
    /// Lifecycle and threading requirements.
    pub lifecycle_contract: PluginLifecycleContract,
}

/// Counts of audio and MIDI buses connected to a plugin instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PluginIoLayout {
    /// Number of audio input buses.
    pub audio_inputs: u16,
    /// Number of audio output buses.
    pub audio_outputs: u16,
    /// Number of MIDI input ports.
    pub midi_inputs: u16,
    /// Number of MIDI output ports.
    pub midi_outputs: u16,
}
