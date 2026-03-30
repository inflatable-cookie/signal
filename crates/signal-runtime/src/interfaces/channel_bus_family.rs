use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeCanonicalChannelLayout {
    Mono,
    Stereo,
    Lcr,
    Quad,
    Surround5_0,
    Surround5_1,
    Surround7_1,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeChannelRole {
    Mono,
    FrontLeft,
    FrontRight,
    FrontCenter,
    LowFrequencyEffects,
    SideLeft,
    SideRight,
    RearLeft,
    RearRight,
    Discrete(u16),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusIntent {
    #[default]
    MainProgram,
    AuxSend,
    AuxReturn,
    Sidechain,
    HardwareInput,
    HardwareOutput,
    AnalysisTap,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputSourceKind {
    #[default]
    NodeOutput,
    BusGroup,
    HardwareInput,
    AnalysisTap,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputTargetKind {
    #[default]
    NodeInput,
    PluginInput,
    RenderInput,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputAttachmentPolicy {
    #[default]
    Required,
    Optional,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSecondaryInputFallbackOutcome {
    #[default]
    BypassSecondaryInput,
    MuteDependentPath,
    SafeModeDegradation,
    TerminalRoutingFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSecondaryInputContractProjection {
    pub source_kind: RuntimeSecondaryInputSourceKind,
    pub source_id: String,
    pub source_bus_id: Option<String>,
    pub target_bus_id: String,
    pub attachment_policy: RuntimeSecondaryInputAttachmentPolicy,
    pub fallback_outcome: RuntimeSecondaryInputFallbackOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSecondaryInputRouteSummary {
    pub source_kind: RuntimeSecondaryInputSourceKind,
    pub source_id: String,
    pub source_bus_id: Option<String>,
    pub target_kind: RuntimeSecondaryInputTargetKind,
    pub target_id: String,
    pub target_bus_id: String,
    pub attachment_policy: RuntimeSecondaryInputAttachmentPolicy,
    pub fallback_outcome: RuntimeSecondaryInputFallbackOutcome,
    pub summary: String,
}

impl RuntimeSecondaryInputRouteSummary {
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusRole {
    #[default]
    ProgramMain,
    Submix,
    AuxSend,
    AuxReturn,
    AnalysisTap,
    HardwareIngress,
    HardwareEgress,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeAuxiliaryPathKind {
    #[default]
    SendReturn,
    Submix,
    Analysis,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusConnectionAttachmentClass {
    #[default]
    Required,
    Optional,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeBusConnectionFallbackOutcome {
    #[default]
    NoFallback,
    BypassAuxiliaryPath,
    MuteDependentPath,
    SafeModeDegradation,
    TerminalTopologyFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeBusConnectionSummary {
    pub connection_id: String,
    pub source_node_id: String,
    pub source_bus_id: String,
    pub source_bus_role: RuntimeBusRole,
    pub target_node_id: String,
    pub target_bus_id: String,
    pub target_bus_role: RuntimeBusRole,
    pub auxiliary_path_kind: Option<RuntimeAuxiliaryPathKind>,
    pub auxiliary_path_id: Option<String>,
    pub attachment_class: RuntimeBusConnectionAttachmentClass,
    pub fallback_outcome: RuntimeBusConnectionFallbackOutcome,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeAuxiliaryPathSummary {
    pub auxiliary_path_id: String,
    pub path_kind: RuntimeAuxiliaryPathKind,
    pub bus_role: RuntimeBusRole,
    pub material_bus_intent: RuntimeBusIntent,
    pub source_node_ids: Vec<String>,
    pub target_node_ids: Vec<String>,
    pub bus_ids: Vec<String>,
    pub connection_ids: Vec<String>,
    pub attachment_class: RuntimeBusConnectionAttachmentClass,
    pub fallback_outcome: RuntimeBusConnectionFallbackOutcome,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMultichannelLayoutSummary {
    pub channel_count: u16,
    pub canonical_layout: Option<RuntimeCanonicalChannelLayout>,
    pub channel_roles: Vec<RuntimeChannelRole>,
    pub uses_custom_fallback: bool,
    pub summary: String,
}

impl Default for RuntimeMultichannelLayoutSummary {
    fn default() -> Self {
        Self::from_channel_count(0)
    }
}

impl RuntimeMultichannelLayoutSummary {
    pub fn from_channel_layout(layout: ChannelLayout) -> Self {
        Self::from_channel_count(layout.channels().0 as u16)
    }

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMultichannelIoSummary {
    pub input_layout: RuntimeMultichannelLayoutSummary,
    pub output_layout: RuntimeMultichannelLayoutSummary,
    pub input_bus_intent: RuntimeBusIntent,
    pub output_bus_intent: RuntimeBusIntent,
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
