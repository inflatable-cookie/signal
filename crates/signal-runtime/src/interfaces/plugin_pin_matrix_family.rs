use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPortClass {
    #[default]
    MainInput,
    MainOutput,
    SecondaryInput,
    AuxInput,
    AuxOutput,
    InstrumentOutput,
    AnalysisOutput,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginBusCapableFxClass {
    #[default]
    SinglePathFx,
    SidechainCapableFx,
    SendReturnCapableFx,
    ParallelCapableFx,
    MultiStemFx,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginTopologyAttachmentPolicy {
    #[default]
    Required,
    Optional,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginTopologyFallbackOutcome {
    #[default]
    CollapseToPrimaryPath,
    BypassUnavailablePortGroup,
    MuteDependentOutput,
    SafeModeDegradation,
    TerminalPluginTopologyFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginComplexIoSummary {
    pub has_complex_topology: bool,
    pub declared_port_classes: Vec<RuntimePluginPortClass>,
    pub port_group_count: usize,
    pub main_input_group_count: usize,
    pub main_output_group_count: usize,
    pub secondary_input_group_count: usize,
    pub aux_input_group_count: usize,
    pub aux_output_group_count: usize,
    pub instrument_output_group_count: usize,
    pub analysis_output_group_count: usize,
    pub multi_output_instrument: bool,
    pub bus_capable_fx_class: Option<RuntimePluginBusCapableFxClass>,
    pub attachment_policy: RuntimePluginTopologyAttachmentPolicy,
    pub fallback_outcome: RuntimePluginTopologyFallbackOutcome,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePluginPinGroupIdentity {
    PrimaryProgramPath,
    SecondaryProgramPath,
    AuxReturnPath,
    SidechainPath,
    AnalysisPath,
    InactiveDeclaredPath,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPinMatrixPosture {
    #[default]
    Simple,
    Declared,
    Negotiated,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDynamicBusNegotiationPosture {
    #[default]
    Static,
    Negotiated,
    Guarded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginNegotiationFallbackOutcome {
    CollapseToDeclaredBaseline,
    DeactivateOptionalPath,
    #[default]
    RoutePrimaryOnly,
    GuardedDegradation,
    TerminalNegotiationFailure,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPinMatrixRecord {
    pub plugin_type_id: String,
    pub plugin_id: String,
    pub pin_group_identities: Vec<RuntimePluginPinGroupIdentity>,
    pub pin_matrix_posture: RuntimePluginPinMatrixPosture,
    pub dynamic_bus_negotiation_posture: RuntimeDynamicBusNegotiationPosture,
    pub fallback_outcome: RuntimePluginNegotiationFallbackOutcome,
    pub strongest_lifecycle_state: Option<RuntimePluginLifecycleState>,
    pub stage_count: usize,
    pub active_stage_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginPinMatrixSnapshot {
    pub plugin_type_count: usize,
    pub negotiated_type_count: usize,
    pub guarded_type_count: usize,
    pub unavailable_type_count: usize,
    pub dynamic_negotiated_type_count: usize,
    pub dynamic_guarded_type_count: usize,
    pub records: Vec<RuntimePluginPinMatrixRecord>,
    pub summary: String,
}

fn div_ceil_u16(value: u16, divisor: u16) -> u16 {
    if value == 0 {
        0
    } else {
        1 + ((value - 1) / divisor)
    }
}

impl RuntimePluginComplexIoSummary {
    pub fn from_plugin_features_and_layout(
        features: &[PluginFeature],
        layout: PluginIoLayout,
    ) -> Self {
        let is_instrument = features.contains(&PluginFeature::Instrument);
        let is_analyzer = features.contains(&PluginFeature::Analyzer);
        let is_fx = features.iter().any(|feature| {
            matches!(
                feature,
                PluginFeature::AudioEffect
                    | PluginFeature::Utility
                    | PluginFeature::Analyzer
                    | PluginFeature::NoteEffect
            )
        }) && !is_instrument;

        let main_input_group_count = usize::from(layout.audio_inputs > 0);
        let main_output_group_count = usize::from(layout.audio_outputs > 0);
        let main_input_channels = if layout.audio_inputs > 0 {
            layout.audio_inputs.min(2)
        } else {
            0
        };
        let main_output_channels = if layout.audio_outputs > 0 {
            layout.audio_outputs.min(2)
        } else {
            0
        };
        let extra_input_groups = usize::from(div_ceil_u16(
            layout.audio_inputs.saturating_sub(main_input_channels),
            2,
        ));
        let extra_output_groups = usize::from(div_ceil_u16(
            layout.audio_outputs.saturating_sub(main_output_channels),
            2,
        ));

        let secondary_input_group_count = if is_fx && extra_input_groups > 0 {
            1
        } else {
            0
        };
        let aux_input_group_count = if is_fx {
            extra_input_groups.saturating_sub(secondary_input_group_count)
        } else {
            0
        };
        let instrument_output_group_count = if is_instrument {
            extra_output_groups
        } else {
            0
        };
        let analysis_output_group_count =
            if is_analyzer && !is_instrument && layout.audio_outputs == 0 {
                1
            } else {
                0
            };
        let aux_output_group_count = if is_instrument {
            0
        } else {
            extra_output_groups
        };
        let multi_output_instrument = is_instrument && instrument_output_group_count > 0;

        let bus_capable_fx_class = if !is_fx {
            None
        } else if secondary_input_group_count > 0 && aux_output_group_count > 0 {
            Some(RuntimePluginBusCapableFxClass::SendReturnCapableFx)
        } else if secondary_input_group_count > 0 {
            Some(RuntimePluginBusCapableFxClass::SidechainCapableFx)
        } else if aux_output_group_count > 1 {
            Some(RuntimePluginBusCapableFxClass::MultiStemFx)
        } else if aux_input_group_count > 0 || aux_output_group_count > 0 {
            Some(RuntimePluginBusCapableFxClass::ParallelCapableFx)
        } else {
            Some(RuntimePluginBusCapableFxClass::SinglePathFx)
        };

        let has_complex_topology = multi_output_instrument
            || secondary_input_group_count > 0
            || aux_input_group_count > 0
            || aux_output_group_count > 0
            || analysis_output_group_count > 0;

        let attachment_policy = if has_complex_topology {
            RuntimePluginTopologyAttachmentPolicy::Optional
        } else {
            RuntimePluginTopologyAttachmentPolicy::Required
        };
        let fallback_outcome = if multi_output_instrument {
            RuntimePluginTopologyFallbackOutcome::CollapseToPrimaryPath
        } else if secondary_input_group_count > 0 {
            RuntimePluginTopologyFallbackOutcome::SafeModeDegradation
        } else if has_complex_topology {
            RuntimePluginTopologyFallbackOutcome::BypassUnavailablePortGroup
        } else {
            RuntimePluginTopologyFallbackOutcome::TerminalPluginTopologyFailure
        };

        let mut declared_port_classes = Vec::new();
        if main_input_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::MainInput);
        }
        if main_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::MainOutput);
        }
        if secondary_input_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::SecondaryInput);
        }
        if aux_input_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::AuxInput);
        }
        if aux_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::AuxOutput);
        }
        if instrument_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::InstrumentOutput);
        }
        if analysis_output_group_count > 0 {
            declared_port_classes.push(RuntimePluginPortClass::AnalysisOutput);
        }

        let port_group_count = main_input_group_count
            + main_output_group_count
            + secondary_input_group_count
            + aux_input_group_count
            + aux_output_group_count
            + instrument_output_group_count
            + analysis_output_group_count;

        let summary = format!(
            "complex={} classes={:?} groups={} main_in={} main_out={} secondary_in={} aux_in={} aux_out={} instrument_out={} analysis_out={} multi_output_instrument={} fx_class={:?} attachment={:?} fallback={:?}",
            has_complex_topology,
            declared_port_classes,
            port_group_count,
            main_input_group_count,
            main_output_group_count,
            secondary_input_group_count,
            aux_input_group_count,
            aux_output_group_count,
            instrument_output_group_count,
            analysis_output_group_count,
            multi_output_instrument,
            bus_capable_fx_class,
            attachment_policy,
            fallback_outcome,
        );

        Self {
            has_complex_topology,
            declared_port_classes,
            port_group_count,
            main_input_group_count,
            main_output_group_count,
            secondary_input_group_count,
            aux_input_group_count,
            aux_output_group_count,
            instrument_output_group_count,
            analysis_output_group_count,
            multi_output_instrument,
            bus_capable_fx_class,
            attachment_policy,
            fallback_outcome,
            summary,
        }
    }
}

pub(crate) fn runtime_bus_intents_for_topology_role(
    topology_role: GraphNodeTopologyRole,
) -> (RuntimeBusIntent, RuntimeBusIntent) {
    match topology_role {
        GraphNodeTopologyRole::Utility => {
            (RuntimeBusIntent::AnalysisTap, RuntimeBusIntent::AnalysisTap)
        }
        GraphNodeTopologyRole::TrackLane
        | GraphNodeTopologyRole::Bus
        | GraphNodeTopologyRole::ConsoleNode => {
            (RuntimeBusIntent::MainProgram, RuntimeBusIntent::MainProgram)
        }
        GraphNodeTopologyRole::Send => (RuntimeBusIntent::MainProgram, RuntimeBusIntent::AuxSend),
        GraphNodeTopologyRole::Return => {
            (RuntimeBusIntent::AuxReturn, RuntimeBusIntent::MainProgram)
        }
    }
}

pub(crate) fn runtime_bus_role_for_endpoint(
    topology_role: GraphNodeTopologyRole,
    bus_intent: RuntimeBusIntent,
) -> RuntimeBusRole {
    match bus_intent {
        RuntimeBusIntent::AuxSend => RuntimeBusRole::AuxSend,
        RuntimeBusIntent::AuxReturn => RuntimeBusRole::AuxReturn,
        RuntimeBusIntent::Sidechain => RuntimeBusRole::AuxSend,
        RuntimeBusIntent::AnalysisTap => RuntimeBusRole::AnalysisTap,
        RuntimeBusIntent::HardwareInput => RuntimeBusRole::HardwareIngress,
        RuntimeBusIntent::HardwareOutput => RuntimeBusRole::HardwareEgress,
        RuntimeBusIntent::MainProgram => match topology_role {
            GraphNodeTopologyRole::Bus => RuntimeBusRole::Submix,
            GraphNodeTopologyRole::Utility => RuntimeBusRole::AnalysisTap,
            GraphNodeTopologyRole::Send => RuntimeBusRole::AuxSend,
            GraphNodeTopologyRole::Return => RuntimeBusRole::AuxReturn,
            GraphNodeTopologyRole::TrackLane | GraphNodeTopologyRole::ConsoleNode => {
                RuntimeBusRole::ProgramMain
            }
        },
    }
}
