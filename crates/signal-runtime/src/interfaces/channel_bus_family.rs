use super::*;

/// Named canonical speaker layout: mono through 7.1 surround.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeCanonicalChannelLayout {
    /// Single centre channel.
    Mono,
    /// Two-channel left/right.
    Stereo,
    /// Left, centre, right.
    Lcr,
    /// Quadraphonic (front L/R + rear L/R).
    Quad,
    /// 5.0 surround (no LFE).
    Surround5_0,
    /// 5.1 surround.
    Surround5_1,
    /// 7.1 surround.
    Surround7_1,
}

/// Semantic role of a single audio channel within a bus (front/side/rear/LFE or discrete index).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeChannelRole {
    /// Single mono channel.
    Mono,
    /// Front-left speaker.
    FrontLeft,
    /// Front-right speaker.
    FrontRight,
    /// Front-centre speaker.
    FrontCenter,
    /// Low-frequency effects (subwoofer) channel.
    LowFrequencyEffects,
    /// Side-left speaker.
    SideLeft,
    /// Side-right speaker.
    SideRight,
    /// Rear-left speaker.
    RearLeft,
    /// Rear-right speaker.
    RearRight,
    /// Discrete channel by index (used for non-canonical layouts).
    Discrete(u16),
}

/// Intended signal flow role of a bus (main program path, aux send/return, sidechain, hardware, analysis).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusIntent {
    /// Primary program audio path.
    #[default]
    MainProgram,
    /// Auxiliary send (pre/post-fader tap to an effect return).
    AuxSend,
    /// Auxiliary return (wet signal back from an effect).
    AuxReturn,
    /// Sidechain input for dynamics or other key-based processing.
    Sidechain,
    /// Physical hardware input.
    HardwareInput,
    /// Physical hardware output.
    HardwareOutput,
    /// Analysis tap (metering or spectrum without affecting the program path).
    AnalysisTap,
}

/// Source kind for a secondary (sidechain/aux) input connection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputSourceKind {
    /// Audio originates from a graph node's output bus.
    #[default]
    NodeOutput,
    /// Audio originates from a bus group.
    BusGroup,
    /// Audio originates from a hardware input.
    HardwareInput,
    /// Audio originates from an analysis tap.
    AnalysisTap,
}

/// Whether a secondary input connection is required, optional, or disabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputAttachmentPolicy {
    /// Connection must be satisfied; failure triggers the fallback outcome.
    #[default]
    Required,
    /// Connection is attempted but failure is non-fatal.
    Optional,
    /// Connection is explicitly not used.
    Disabled,
}

/// Fallback outcome when a secondary input connection cannot be satisfied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputFallbackOutcome {
    /// Skip the secondary input and continue processing without it.
    #[default]
    BypassSecondaryInput,
    /// Silence the path that depended on the secondary input.
    MuteDependentPath,
    /// Engage safe-mode degradation for the affected path.
    SafeModeDegradation,
    /// Hard routing failure; the path cannot be rendered.
    TerminalRoutingFailure,
}

/// Projected contract for a secondary input connection: source/target IDs, bus IDs, and attachment policy.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSecondaryInputContractProjection {
    /// Kind of the source providing the secondary input.
    pub source_kind: RuntimeSecondaryInputSourceKind,
    /// Identifier of the source node, bus, or tap.
    pub source_id: String,
    /// Output bus ID on the source, if applicable.
    pub source_bus_id: Option<String>,
    /// Input bus ID on the target that receives the secondary input.
    pub target_bus_id: String,
    /// Attachment policy for this connection.
    pub attachment_policy: RuntimeSecondaryInputAttachmentPolicy,
    /// Fallback outcome if the connection cannot be satisfied.
    pub fallback_outcome: RuntimeSecondaryInputFallbackOutcome,
}

/// Resolved multichannel layout summary: canonical layout, per-channel roles, and whether a custom discrete fallback is used.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMultichannelLayoutSummary {
    /// Number of channels in this layout.
    pub channel_count: u16,
    /// Canonical layout identifier, if this count maps to a standard layout.
    pub canonical_layout: Option<RuntimeCanonicalChannelLayout>,
    /// Semantic role for each channel in order.
    pub channel_roles: Vec<RuntimeChannelRole>,
    /// Whether discrete (non-canonical) channel roles are used as a fallback.
    pub uses_custom_fallback: bool,
}

impl Default for RuntimeMultichannelLayoutSummary {
    fn default() -> Self {
        Self::from_channel_count(0)
    }
}

impl RuntimeMultichannelLayoutSummary {
    /// Builds a layout summary from a [`ChannelLayout`] value.
    pub fn from_channel_layout(layout: ChannelLayout) -> Self {
        Self::from_channel_count(layout.channels().0 as u16)
    }

    /// Builds a layout summary for an arbitrary channel count, mapping known counts to canonical layouts.
    pub fn from_channel_count(channel_count: u16) -> Self {
        let (canonical_layout, channel_roles, uses_custom_fallback) = match channel_count {
            0 => (None, Vec::new(), false),
            1 => (
                Some(RuntimeCanonicalChannelLayout::Mono),
                vec![RuntimeChannelRole::Mono],
                false,
            ),
            2 => (
                Some(RuntimeCanonicalChannelLayout::Stereo),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                ],
                false,
            ),
            3 => (
                Some(RuntimeCanonicalChannelLayout::Lcr),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::FrontRight,
                ],
                false,
            ),
            4 => (
                Some(RuntimeCanonicalChannelLayout::Quad),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::RearLeft,
                    RuntimeChannelRole::RearRight,
                ],
                false,
            ),
            5 => (
                Some(RuntimeCanonicalChannelLayout::Surround5_0),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::SideLeft,
                    RuntimeChannelRole::SideRight,
                ],
                false,
            ),
            6 => (
                Some(RuntimeCanonicalChannelLayout::Surround5_1),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::LowFrequencyEffects,
                    RuntimeChannelRole::SideLeft,
                    RuntimeChannelRole::SideRight,
                ],
                false,
            ),
            8 => (
                Some(RuntimeCanonicalChannelLayout::Surround7_1),
                vec![
                    RuntimeChannelRole::FrontLeft,
                    RuntimeChannelRole::FrontRight,
                    RuntimeChannelRole::FrontCenter,
                    RuntimeChannelRole::LowFrequencyEffects,
                    RuntimeChannelRole::SideLeft,
                    RuntimeChannelRole::SideRight,
                    RuntimeChannelRole::RearLeft,
                    RuntimeChannelRole::RearRight,
                ],
                false,
            ),
            _ => (
                None,
                (0..channel_count)
                    .map(RuntimeChannelRole::Discrete)
                    .collect(),
                true,
            ),
        };
        let _summary = match canonical_layout {
            Some(layout) => format!(
                "channels={} canonical={layout:?} roles={:?}",
                channel_count, channel_roles
            ),
            None if channel_count == 0 => "channels=0 canonical=None roles=[]".into(),
            None => format!(
                "channels={} canonical=None roles={:?} fallback=Discrete",
                channel_count, channel_roles
            ),
        };
        Self {
            channel_count,
            canonical_layout,
            channel_roles,
            uses_custom_fallback,
        }
    }
}

/// Combined multichannel I/O summary: input and output layout plus bus intent for each direction.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMultichannelIoSummary {
    /// Multichannel layout summary for the input.
    pub input_layout: RuntimeMultichannelLayoutSummary,
    /// Multichannel layout summary for the output.
    pub output_layout: RuntimeMultichannelLayoutSummary,
    /// Bus intent for the input direction.
    pub input_bus_intent: RuntimeBusIntent,
    /// Bus intent for the output direction.
    pub output_bus_intent: RuntimeBusIntent,
}

impl Default for RuntimeMultichannelIoSummary {
    fn default() -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::default(),
            RuntimeMultichannelLayoutSummary::default(),
            RuntimeBusIntent::MainProgram,
            RuntimeBusIntent::MainProgram,
        )
    }
}

impl RuntimeMultichannelIoSummary {
    /// Constructs an I/O summary from explicit layout summaries and bus intents.
    pub fn new(
        input_layout: RuntimeMultichannelLayoutSummary,
        output_layout: RuntimeMultichannelLayoutSummary,
        input_bus_intent: RuntimeBusIntent,
        output_bus_intent: RuntimeBusIntent,
    ) -> Self {
        Self {
            input_layout,
            output_layout,
            input_bus_intent,
            output_bus_intent,
        }
    }

    /// Constructs an I/O summary for a plugin from its declared I/O layout.
    pub fn for_plugin_io(layout: PluginIoLayout) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_count(layout.audio_inputs),
            RuntimeMultichannelLayoutSummary::from_channel_count(layout.audio_outputs),
            RuntimeBusIntent::MainProgram,
            RuntimeBusIntent::MainProgram,
        )
    }

    /// Constructs an I/O summary for a hardware device with the given channel counts.
    pub fn for_hardware(input_channels: u16, output_channels: u16) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_count(input_channels),
            RuntimeMultichannelLayoutSummary::from_channel_count(output_channels),
            RuntimeBusIntent::HardwareInput,
            RuntimeBusIntent::HardwareOutput,
        )
    }
}
