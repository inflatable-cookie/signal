use super::super::*;
use super::spatial_execution_policy::*;

fn runtime_spatial_target_environment_for_layout(
    layout: &RuntimeMultichannelLayoutSummary,
) -> RuntimeSpatialTargetEnvironment {
    if layout.uses_custom_fallback {
        RuntimeSpatialTargetEnvironment::CustomEnvironment
    } else {
        RuntimeSpatialTargetEnvironment::SourceLayout
    }
}

fn runtime_spatial_bed_class_for_layout(
    layout: &RuntimeMultichannelLayoutSummary,
) -> RuntimeSpatialBedClass {
    match layout.canonical_layout {
        Some(RuntimeCanonicalChannelLayout::Stereo) if layout.channel_count == 2 => {
            RuntimeSpatialBedClass::StereoBed
        }
        Some(
            RuntimeCanonicalChannelLayout::Lcr
            | RuntimeCanonicalChannelLayout::Quad
            | RuntimeCanonicalChannelLayout::Surround5_0
            | RuntimeCanonicalChannelLayout::Surround5_1
            | RuntimeCanonicalChannelLayout::Surround7_1,
        ) => RuntimeSpatialBedClass::CanonicalSurroundBed,
        _ => RuntimeSpatialBedClass::CustomDiscreteBed,
    }
}

fn runtime_spatial_mix_policy_for_layout(
    layout: &RuntimeMultichannelLayoutSummary,
) -> RuntimeSpatialMixPolicy {
    if layout.channel_count == 2 && !layout.uses_custom_fallback {
        RuntimeSpatialMixPolicy::BedOnly
    } else {
        RuntimeSpatialMixPolicy::CollapseToBaselineSpatial
    }
}

fn runtime_spatial_render_scope_for_summary(
    object_count: usize,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
) -> RuntimeSpatialRenderScope {
    if object_count > 0 {
        if matches!(
            expanded_fallback_outcome,
            Some(RuntimeSpatialExpandedFallbackOutcome::CollapseObjectsIntoBed)
        ) {
            RuntimeSpatialRenderScope::BedFoldDownRender
        } else {
            RuntimeSpatialRenderScope::BedAndObjectRender
        }
    } else {
        RuntimeSpatialRenderScope::BedRender
    }
}

pub(crate) fn runtime_spatial_execution_summary_for_stages(
    node_id: &str,
    stages: &[GraphStageSpec],
    input_layout: &RuntimeMultichannelLayoutSummary,
    output_layout: &RuntimeMultichannelLayoutSummary,
) -> Option<RuntimeSpatialExecutionSummary> {
    stages.iter().find_map(|stage| match stage {
        GraphStageSpec::StereoBalance { balance } => {
            let supports_direct_balance = output_layout.channel_count == 2;
            let execution_mode = if supports_direct_balance {
                RuntimeSpatialExecutionMode::BalanceGroups
            } else {
                RuntimeSpatialExecutionMode::Bypassed
            };
            let fallback_outcome = (!supports_direct_balance)
                .then_some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing);
            let bed_class = runtime_spatial_bed_class_for_layout(output_layout);
            let object_count = 0usize;
            let mix_policy = runtime_spatial_mix_policy_for_layout(output_layout);
            let expanded_fallback_outcome = (!supports_direct_balance)
                .then_some(RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial);
            let render_scope =
                runtime_spatial_render_scope_for_summary(object_count, expanded_fallback_outcome);
            let balance = format!("{balance:.3}");
            let target_environment = runtime_spatial_target_environment_for_layout(output_layout);
            let immersive_room_policy =
                runtime_immersive_room_policy_summary_for_spatial(RuntimeSpatialRoomPolicyInput {
                    adapter_class: RuntimeSpatialAdapterClass::Balance,
                    execution_mode,
                    target_environment,
                    fallback_outcome,
                    bed_class,
                    object_role: None,
                    object_count,
                    render_scope,
                    expanded_fallback_outcome,
                });
            let deployment_monitoring = runtime_deployment_monitoring_summary_for_spatial(
                target_environment,
                bed_class,
                fallback_outcome,
                expanded_fallback_outcome,
                immersive_room_policy.as_ref(),
            );
            let renderer_export = runtime_renderer_immersive_export_summary_for_spatial(
                RuntimeSpatialAdapterClass::Balance,
                execution_mode,
                target_environment,
                fallback_outcome,
                expanded_fallback_outcome,
                immersive_room_policy.as_ref(),
                deployment_monitoring.as_ref(),
            );
            Some(RuntimeSpatialExecutionSummary {
                node_id: node_id.into(),
                adapter_class: RuntimeSpatialAdapterClass::Balance,
                execution_mode,
                target_environment,
                control_family: RuntimeSpatialControlFamily::BalanceScalar,
                activation_policy: RuntimeSpatialActivationPolicy::EnabledIfSupported,
                fallback_outcome,
                bed_class,
                object_role: None,
                object_count,
                mix_policy,
                render_scope,
                expanded_fallback_outcome,
                immersive_room_policy: immersive_room_policy.clone(),
                deployment_monitoring: deployment_monitoring.clone(),
                renderer_export: renderer_export.clone(),
                balance: Some(balance.clone()),
                input_layout: input_layout.clone(),
                output_layout: output_layout.clone(),
            })
        }
        _ => None,
    })
}

impl RuntimeMultichannelIoSummary {
    /// Constructs an I/O summary from explicit layout summaries and bus intents.
    pub fn new(
        input_layout: RuntimeMultichannelLayoutSummary,
        output_layout: RuntimeMultichannelLayoutSummary,
        input_bus_intent: RuntimeBusIntent,
        output_bus_intent: RuntimeBusIntent,
    ) -> Self {
        let _summary = format!(
            "input={:?}/{:?} output={:?}/{:?}",
            input_bus_intent,
            input_layout.canonical_layout,
            output_bus_intent,
            output_layout.canonical_layout
        );
        Self {
            input_layout,
            output_layout,
            input_bus_intent,
            output_bus_intent,
        }
    }

    /// Constructs an I/O summary from raw `ChannelLayout` values and bus intents.
    pub fn for_channel_layouts(
        input_layout: ChannelLayout,
        output_layout: ChannelLayout,
        input_bus_intent: RuntimeBusIntent,
        output_bus_intent: RuntimeBusIntent,
    ) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_layout(input_layout),
            RuntimeMultichannelLayoutSummary::from_channel_layout(output_layout),
            input_bus_intent,
            output_bus_intent,
        )
    }

    /// Constructs an I/O summary for a plugin from its declared I/O layout, using `MainProgram` bus intent on both sides.
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
