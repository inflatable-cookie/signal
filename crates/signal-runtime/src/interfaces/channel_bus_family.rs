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

/// Target kind for a secondary (sidechain/aux) input connection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputTargetKind {
    /// Secondary audio feeds into a graph node's input bus.
    #[default]
    NodeInput,
    /// Secondary audio feeds directly into a plugin's input.
    PluginInput,
    /// Secondary audio feeds into a render stage input.
    RenderInput,
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

/// Resolved routing summary for a secondary input connection: source and target kinds, IDs, bus IDs, and attachment/fallback policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSecondaryInputRouteSummary {
    /// Kind of the source providing the secondary input.
    pub source_kind: RuntimeSecondaryInputSourceKind,
    /// Identifier of the source.
    pub source_id: String,
    /// Output bus ID on the source, if applicable.
    pub source_bus_id: Option<String>,
    /// Kind of the target receiving the secondary input.
    pub target_kind: RuntimeSecondaryInputTargetKind,
    /// Identifier of the target.
    pub target_id: String,
    /// Input bus ID on the target.
    pub target_bus_id: String,
    /// Attachment policy for this connection.
    pub attachment_policy: RuntimeSecondaryInputAttachmentPolicy,
    /// Fallback outcome if the connection cannot be satisfied.
    pub fallback_outcome: RuntimeSecondaryInputFallbackOutcome,
    /// Human-readable routing summary.
    pub summary: String,
}

impl RuntimeSecondaryInputRouteSummary {
    /// Builds a resolved route summary from a contract projection and the resolved target kind and ID.
    pub fn from_contract_for_target(
        contract: &RuntimeSecondaryInputContractProjection,
        target_kind: RuntimeSecondaryInputTargetKind,
        target_id: impl Into<String>,
    ) -> Self {
        let target_id = target_id.into();
        let summary = format!(
            "source={:?}:{}/{} target={:?}:{}/{} policy={:?} fallback={:?}",
            contract.source_kind,
            contract.source_id,
            contract.source_bus_id.as_deref().unwrap_or("none"),
            target_kind,
            target_id,
            contract.target_bus_id,
            contract.attachment_policy,
            contract.fallback_outcome,
        );
        Self {
            source_kind: contract.source_kind,
            source_id: contract.source_id.clone(),
            source_bus_id: contract.source_bus_id.clone(),
            target_kind,
            target_id,
            target_bus_id: contract.target_bus_id.clone(),
            attachment_policy: contract.attachment_policy,
            fallback_outcome: contract.fallback_outcome,
            summary,
        }
    }
}

/// Resolved role of a bus within the signal flow graph (main program, submix, aux, analysis, hardware).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusRole {
    /// Primary program audio bus.
    #[default]
    ProgramMain,
    /// Submix bus aggregating multiple sources.
    Submix,
    /// Auxiliary send bus.
    AuxSend,
    /// Auxiliary return bus.
    AuxReturn,
    /// Analysis tap bus (non-destructive metering).
    AnalysisTap,
    /// Hardware input ingress bus.
    HardwareIngress,
    /// Hardware output egress bus.
    HardwareEgress,
}

/// Kind of auxiliary routing path: send/return, submix, or analysis tap.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeAuxiliaryPathKind {
    /// Send/return effects loop.
    #[default]
    SendReturn,
    /// Submix grouping path.
    Submix,
    /// Analysis tap (metering, spectrum, etc.).
    Analysis,
}

/// Whether a bus connection is required, optional, or disabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusConnectionAttachmentClass {
    /// Connection must be present; failure triggers the fallback outcome.
    #[default]
    Required,
    /// Connection is attempted but failure is non-fatal.
    Optional,
    /// Connection is explicitly disabled.
    Disabled,
}

/// Fallback outcome when a bus connection topology cannot be satisfied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusConnectionFallbackOutcome {
    /// No fallback; topology failure is not expected.
    #[default]
    NoFallback,
    /// Bypass the auxiliary path and continue.
    BypassAuxiliaryPath,
    /// Silence the path that depends on this connection.
    MuteDependentPath,
    /// Engage safe-mode degradation.
    SafeModeDegradation,
    /// Unrecoverable topology failure.
    TerminalTopologyFailure,
}

/// Summary of a single bus connection edge: source/target node and bus IDs, roles, auxiliary path identity, and attachment/fallback policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeBusConnectionSummary {
    /// Unique identifier for this connection edge.
    pub connection_id: String,
    /// Source node of the connection.
    pub source_node_id: String,
    /// Output bus ID on the source node.
    pub source_bus_id: String,
    /// Role of the source bus.
    pub source_bus_role: RuntimeBusRole,
    /// Target node of the connection.
    pub target_node_id: String,
    /// Input bus ID on the target node.
    pub target_bus_id: String,
    /// Role of the target bus.
    pub target_bus_role: RuntimeBusRole,
    /// Kind of auxiliary path this connection is part of, if any.
    pub auxiliary_path_kind: Option<RuntimeAuxiliaryPathKind>,
    /// Auxiliary path identifier, if applicable.
    pub auxiliary_path_id: Option<String>,
    /// Attachment class for this connection.
    pub attachment_class: RuntimeBusConnectionAttachmentClass,
    /// Fallback outcome if this connection cannot be satisfied.
    pub fallback_outcome: RuntimeBusConnectionFallbackOutcome,
    /// Human-readable summary of this connection.
    pub summary: String,
}

/// Summary of an auxiliary routing path: kind, bus role, source/target node IDs, and the bus and connection IDs it spans.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAuxiliaryPathSummary {
    /// Unique identifier for this auxiliary path.
    pub auxiliary_path_id: String,
    /// Kind of auxiliary path (send/return, submix, analysis).
    pub path_kind: RuntimeAuxiliaryPathKind,
    /// Role of the buses on this path.
    pub bus_role: RuntimeBusRole,
    /// Bus intent of the material signal on this path.
    pub material_bus_intent: RuntimeBusIntent,
    /// Source node IDs feeding into this path.
    pub source_node_ids: Vec<String>,
    /// Target node IDs receiving signal from this path.
    pub target_node_ids: Vec<String>,
    /// Bus IDs involved in this path.
    pub bus_ids: Vec<String>,
    /// Connection edge IDs that make up this path.
    pub connection_ids: Vec<String>,
    /// Attachment class for this auxiliary path.
    pub attachment_class: RuntimeBusConnectionAttachmentClass,
    /// Fallback outcome if this path cannot be satisfied.
    pub fallback_outcome: RuntimeBusConnectionFallbackOutcome,
    /// Human-readable summary of this auxiliary path.
    pub summary: String,
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
    /// Human-readable layout summary.
    pub summary: String,
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
        let summary = match canonical_layout {
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
            summary,
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
    /// Human-readable I/O summary.
    pub summary: String,
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
