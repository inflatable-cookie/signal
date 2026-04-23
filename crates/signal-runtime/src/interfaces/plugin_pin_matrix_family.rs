use super::*;

/// Semantic class of a plugin audio port group (main, secondary, aux, instrument, analysis).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPortClass {
    #[default]
    /// Primary audio input port group.
    MainInput,
    /// Primary audio output port group.
    MainOutput,
    /// Secondary (sidechain) audio input port group.
    SecondaryInput,
    /// Auxiliary audio input port group.
    AuxInput,
    /// Auxiliary audio output port group.
    AuxOutput,
    /// Instrument audio output port group.
    InstrumentOutput,
    /// Analysis-only audio output port group.
    AnalysisOutput,
}

/// Bus-routing capability class for an FX plugin (sidechain, send/return, parallel, multi-stem).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginBusCapableFxClass {
    #[default]
    /// FX with a single audio path; no auxiliary routing.
    SinglePathFx,
    /// FX that accepts a sidechain input.
    SidechainCapableFx,
    /// FX that supports send/return bus routing.
    SendReturnCapableFx,
    /// FX that supports parallel bus routing.
    ParallelCapableFx,
    /// FX with multiple independent output stems.
    MultiStemFx,
}

/// Whether a complex plugin port group is required, optional, or disabled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginTopologyAttachmentPolicy {
    #[default]
    /// Port group attachment is required for operation.
    Required,
    /// Port group attachment is optional.
    Optional,
    /// Port group is disabled and will not be attached.
    Disabled,
}

/// Fallback outcome when a complex plugin port topology cannot be satisfied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginTopologyFallbackOutcome {
    #[default]
    /// Collapse to the primary audio path only.
    CollapseToPrimaryPath,
    /// Bypass the unavailable port group and continue.
    BypassUnavailablePortGroup,
    /// Mute any output that depends on the unavailable port group.
    MuteDependentOutput,
    /// Degrade to safe-mode behaviour.
    SafeModeDegradation,
    /// Topology has permanently failed; no recovery possible.
    TerminalPluginTopologyFailure,
}

/// Decomposed port topology summary for a plugin: declared port classes, group counts, FX bus class, and attachment/fallback policy.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginComplexIoSummary {
    /// Whether this plugin has any non-trivial port topology.
    pub has_complex_topology: bool,
    /// List of distinct port classes declared by this plugin.
    pub declared_port_classes: Vec<RuntimePluginPortClass>,
    /// Total number of declared port groups.
    pub port_group_count: usize,
    /// Number of main (stereo) input port groups.
    pub main_input_group_count: usize,
    /// Number of main (stereo) output port groups.
    pub main_output_group_count: usize,
    /// Number of secondary (sidechain) input port groups.
    pub secondary_input_group_count: usize,
    /// Number of auxiliary input port groups.
    pub aux_input_group_count: usize,
    /// Number of auxiliary output port groups.
    pub aux_output_group_count: usize,
    /// Number of instrument output port groups.
    pub instrument_output_group_count: usize,
    /// Number of analysis output port groups.
    pub analysis_output_group_count: usize,
    /// Whether this is an instrument with multiple output buses.
    pub multi_output_instrument: bool,
    /// FX bus routing capability class, if this plugin is an FX type.
    pub bus_capable_fx_class: Option<RuntimePluginBusCapableFxClass>,
    /// Port group attachment policy for optional groups.
    pub attachment_policy: RuntimePluginTopologyAttachmentPolicy,
    /// Fallback outcome when the topology cannot be fully satisfied.
    pub fallback_outcome: RuntimePluginTopologyFallbackOutcome,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Identity of a resolved plugin pin group within the runtime graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimePluginPinGroupIdentity {
    /// Primary program audio path.
    PrimaryProgramPath,
    /// Secondary program audio path.
    SecondaryProgramPath,
    /// Auxiliary return audio path.
    AuxReturnPath,
    /// Sidechain input path.
    SidechainPath,
    /// Analysis-only output path.
    AnalysisPath,
    /// A declared path that is currently inactive.
    InactiveDeclaredPath,
}

/// Pin matrix configuration posture: simple fixed layout, declared multi-bus, runtime-negotiated, or guarded/unavailable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginPinMatrixPosture {
    #[default]
    /// Simple fixed layout; no complex topology.
    Simple,
    /// Layout declared by the plugin manifest.
    Declared,
    /// Layout negotiated at runtime with the plugin.
    Negotiated,
    /// Layout is in a guarded fallback state.
    Guarded,
    /// Pin matrix configuration is unavailable.
    Unavailable,
}

/// Dynamic bus count negotiation posture: static, runtime-negotiated, guarded, or unavailable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDynamicBusNegotiationPosture {
    #[default]
    /// Bus count is static; no runtime negotiation occurs.
    Static,
    /// Bus count was negotiated at runtime.
    Negotiated,
    /// Negotiation is in a guarded fallback state.
    Guarded,
    /// Dynamic bus negotiation is unavailable.
    Unavailable,
}

/// Fallback outcome when a dynamic pin matrix negotiation cannot be satisfied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePluginNegotiationFallbackOutcome {
    /// Collapse to the plugin's declared baseline configuration.
    CollapseToDeclaredBaseline,
    /// Deactivate the optional path that could not be negotiated.
    DeactivateOptionalPath,
    #[default]
    /// Route only the primary audio path.
    RoutePrimaryOnly,
    /// Degrade to a guarded fallback state.
    GuardedDegradation,
    /// Negotiation has permanently failed.
    TerminalNegotiationFailure,
}

/// Pin matrix record for one plugin type: resolved pin group identities, posture, negotiation fallback, and stage counts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePluginPinMatrixRecord {
    /// Stable unique identifier for this plugin type.
    pub plugin_type_id: String,
    /// Shorter plugin identifier.
    pub plugin_id: String,
    /// Resolved set of pin group identities for this plugin type.
    pub pin_group_identities: Vec<RuntimePluginPinGroupIdentity>,
    /// Overall pin matrix configuration posture.
    pub pin_matrix_posture: RuntimePluginPinMatrixPosture,
    /// Dynamic bus count negotiation posture.
    pub dynamic_bus_negotiation_posture: RuntimeDynamicBusNegotiationPosture,
    /// Fallback outcome if dynamic negotiation cannot be satisfied.
    pub fallback_outcome: RuntimePluginNegotiationFallbackOutcome,
    /// Highest-severity lifecycle state across all sandboxes for this type.
    pub strongest_lifecycle_state: Option<RuntimePluginLifecycleState>,
    /// Total chain stages for this plugin type.
    pub stage_count: usize,
    /// Number of chain stages with an active transport session.
    pub active_stage_count: usize,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Aggregate pin matrix snapshot across all plugin types: counts by posture and the full record list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePluginPinMatrixSnapshot {
    /// Total number of plugin types with complex I/O topology.
    pub plugin_type_count: usize,
    /// Number of types with a fully negotiated pin matrix.
    pub negotiated_type_count: usize,
    /// Number of types with a guarded pin matrix posture.
    pub guarded_type_count: usize,
    /// Number of types where pin matrix is unavailable.
    pub unavailable_type_count: usize,
    /// Number of types with a negotiated dynamic bus count.
    pub dynamic_negotiated_type_count: usize,
    /// Number of types with a guarded dynamic bus negotiation.
    pub dynamic_guarded_type_count: usize,
    /// Per-type pin matrix records.
    pub records: Vec<RuntimePluginPinMatrixRecord>,
    /// Human-readable one-line summary.
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
    /// Derives the complex I/O topology summary from a plugin's declared features and default I/O layout.
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
