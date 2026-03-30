use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialAdapterClass {
    #[default]
    Balance,
    PerChannelGain,
    LayoutTransform,
    Renderer,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialExecutionMode {
    #[default]
    Bypassed,
    BalanceGroups,
    PerChannelAttenuation,
    TransformToTargetLayout,
    RenderToEnvironment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialTargetEnvironment {
    #[default]
    SourceLayout,
    CanonicalLayout,
    DeviceLayout,
    CustomEnvironment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialControlFamily {
    #[default]
    BalanceScalar,
    PerChannelVector,
    AdapterParameterSet,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialActivationPolicy {
    Disabled,
    #[default]
    EnabledIfSupported,
    Required,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialFallbackOutcome {
    BypassSpatialProcessing,
    CollapseToBalance,
    CollapseToPerChannelGain,
    SafeModeDegradation,
    TerminalSpatialFailure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialBedClass {
    #[default]
    StereoBed,
    CanonicalSurroundBed,
    CustomDiscreteBed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialObjectRole {
    PrimaryObject,
    AuxiliaryObject,
    EffectObject,
    AnalysisObject,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialMixPolicy {
    #[default]
    BedOnly,
    BedWithObjects,
    ObjectPreferredWithBedFallback,
    DownmixToCanonicalBed,
    CollapseToBaselineSpatial,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialRenderScope {
    #[default]
    BedRender,
    BedAndObjectRender,
    BedFoldDownRender,
    ObjectMetadataOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialExpandedFallbackOutcome {
    CollapseObjectsIntoBed,
    CollapseToCanonicalBed,
    CollapseToBaselineSpatial,
    BypassExpandedSpatial,
    TerminalExpandedSpatialFailure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveObjectRenderingPosture {
    #[default]
    NotRequested,
    MetadataOnly,
    RoomPolicyAware,
    CollapsedToBed,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRoomPolicyClass {
    #[default]
    NoRoomPolicy,
    ReferenceRoom,
    MonitoringRoom,
    DeploymentRoom,
    FallbackRoom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRoomPolicyAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveRoomOutcome {
    #[default]
    BypassRoomPolicy,
    RenderObjectsAgainstRoomPolicy,
    PreserveObjectMetadataOnly,
    CollapseObjectsIntoBed,
    TerminalImmersiveFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeImmersiveRoomPolicySummary {
    pub object_rendering_posture: RuntimeImmersiveObjectRenderingPosture,
    pub room_policy_class: RuntimeRoomPolicyClass,
    pub room_policy_authority: RuntimeRoomPolicyAuthority,
    pub room_outcome: RuntimeImmersiveRoomOutcome,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeploymentClass {
    #[default]
    SourceLayoutDeployment,
    ReferenceSpeakerDeployment,
    MonitoringSpeakerDeployment,
    PortableFoldDownDeployment,
    FallbackDeployment,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeFoldDownPolicy {
    #[default]
    PreserveDeclaredDeployment,
    FoldDownToReferenceBed,
    FoldDownToStereoMonitoring,
    FoldDownToPortablePreview,
    BypassDeploymentPolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringSceneClass {
    #[default]
    NoMonitoringScene,
    ReferenceScene,
    FoldDownScene,
    ConfidenceScene,
    FallbackScene,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringSceneAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringOutcome {
    MonitorDeclaredDeployment,
    MonitorFoldedDownScene,
    MonitorPortablePreview,
    #[default]
    BypassMonitoringScene,
    TerminalMonitoringFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDeploymentMonitoringSummary {
    pub deployment_class: RuntimeDeploymentClass,
    pub fold_down_policy: RuntimeFoldDownPolicy,
    pub monitoring_scene_class: RuntimeMonitoringSceneClass,
    pub monitoring_scene_authority: RuntimeMonitoringSceneAuthority,
    pub monitoring_outcome: RuntimeMonitoringOutcome,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRendererCapabilityNegotiationPosture {
    #[default]
    NotRequested,
    DeclaredCompatible,
    NegotiatedCompatible,
    FallbackNegotiation,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRendererCapabilityAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportClass {
    #[default]
    NoImmersiveExport,
    BedOnlyExport,
    ObjectAwareExport,
    MonitoringPreviewExport,
    FallbackExport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    RendererAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportOutcome {
    PreserveDeclaredExport,
    CollapseToBedExport,
    PreserveMetadataOnly,
    #[default]
    BypassImmersiveExport,
    TerminalExportFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeRendererImmersiveExportSummary {
    pub renderer_capability_posture: RuntimeRendererCapabilityNegotiationPosture,
    pub capability_authority: RuntimeRendererCapabilityAuthority,
    pub immersive_export_class: RuntimeImmersiveExportClass,
    pub export_authority: RuntimeImmersiveExportAuthority,
    pub export_outcome: RuntimeImmersiveExportOutcome,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpatialExecutionSummary {
    pub node_id: String,
    pub adapter_class: RuntimeSpatialAdapterClass,
    pub execution_mode: RuntimeSpatialExecutionMode,
    pub target_environment: RuntimeSpatialTargetEnvironment,
    pub control_family: RuntimeSpatialControlFamily,
    pub activation_policy: RuntimeSpatialActivationPolicy,
    pub fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    pub bed_class: RuntimeSpatialBedClass,
    pub object_role: Option<RuntimeSpatialObjectRole>,
    pub object_count: usize,
    pub mix_policy: RuntimeSpatialMixPolicy,
    pub render_scope: RuntimeSpatialRenderScope,
    pub expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
    pub immersive_room_policy: Option<RuntimeImmersiveRoomPolicySummary>,
    pub deployment_monitoring: Option<RuntimeDeploymentMonitoringSummary>,
    pub renderer_export: Option<RuntimeRendererImmersiveExportSummary>,
    pub balance: Option<String>,
    pub input_layout: RuntimeMultichannelLayoutSummary,
    pub output_layout: RuntimeMultichannelLayoutSummary,
    pub summary: String,
}

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

struct RuntimeSpatialRoomPolicyInput {
    adapter_class: RuntimeSpatialAdapterClass,
    execution_mode: RuntimeSpatialExecutionMode,
    target_environment: RuntimeSpatialTargetEnvironment,
    fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    bed_class: RuntimeSpatialBedClass,
    object_role: Option<RuntimeSpatialObjectRole>,
    object_count: usize,
    render_scope: RuntimeSpatialRenderScope,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
}

fn runtime_immersive_room_policy_summary_for_spatial(
    spatial: RuntimeSpatialRoomPolicyInput,
) -> Option<RuntimeImmersiveRoomPolicySummary> {
    let immersive_candidate = spatial.bed_class != RuntimeSpatialBedClass::StereoBed
        || spatial.object_count > 0
        || spatial.object_role.is_some()
        || spatial.adapter_class == RuntimeSpatialAdapterClass::Renderer
        || spatial.execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
        || spatial.target_environment != RuntimeSpatialTargetEnvironment::SourceLayout
        || matches!(
            spatial.render_scope,
            RuntimeSpatialRenderScope::BedAndObjectRender
                | RuntimeSpatialRenderScope::BedFoldDownRender
                | RuntimeSpatialRenderScope::ObjectMetadataOnly
        );
    if !immersive_candidate {
        return None;
    }

    let room_policy_class = if spatial.execution_mode == RuntimeSpatialExecutionMode::Bypassed
        || spatial.fallback_outcome.is_some()
        || spatial.expanded_fallback_outcome.is_some()
    {
        RuntimeRoomPolicyClass::FallbackRoom
    } else {
        match spatial.target_environment {
            RuntimeSpatialTargetEnvironment::SourceLayout
            | RuntimeSpatialTargetEnvironment::CanonicalLayout => {
                RuntimeRoomPolicyClass::ReferenceRoom
            }
            RuntimeSpatialTargetEnvironment::DeviceLayout => RuntimeRoomPolicyClass::MonitoringRoom,
            RuntimeSpatialTargetEnvironment::CustomEnvironment => {
                RuntimeRoomPolicyClass::DeploymentRoom
            }
        }
    };

    let room_policy_authority = if room_policy_class == RuntimeRoomPolicyClass::FallbackRoom {
        RuntimeRoomPolicyAuthority::RuntimeDefault
    } else if spatial.execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
        || spatial.adapter_class == RuntimeSpatialAdapterClass::Renderer
    {
        RuntimeRoomPolicyAuthority::RendererAdvisory
    } else {
        RuntimeRoomPolicyAuthority::RuntimeDeclared
    };

    let object_rendering_posture = if spatial.object_count == 0 && spatial.object_role.is_none() {
        RuntimeImmersiveObjectRenderingPosture::NotRequested
    } else if spatial.render_scope == RuntimeSpatialRenderScope::ObjectMetadataOnly {
        RuntimeImmersiveObjectRenderingPosture::MetadataOnly
    } else if spatial.execution_mode == RuntimeSpatialExecutionMode::Bypassed
        || spatial.fallback_outcome.is_some()
        || matches!(
            spatial.expanded_fallback_outcome,
            Some(
                RuntimeSpatialExpandedFallbackOutcome::CollapseObjectsIntoBed
                    | RuntimeSpatialExpandedFallbackOutcome::CollapseToCanonicalBed
                    | RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial
                    | RuntimeSpatialExpandedFallbackOutcome::BypassExpandedSpatial
            )
        )
    {
        RuntimeImmersiveObjectRenderingPosture::CollapsedToBed
    } else if room_policy_class == RuntimeRoomPolicyClass::FallbackRoom {
        RuntimeImmersiveObjectRenderingPosture::Unavailable
    } else {
        RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
    };

    let room_outcome = if matches!(
        spatial.expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::TerminalExpandedSpatialFailure)
    ) || spatial.fallback_outcome
        == Some(RuntimeSpatialFallbackOutcome::TerminalSpatialFailure)
    {
        RuntimeImmersiveRoomOutcome::TerminalImmersiveFailure
    } else {
        match object_rendering_posture {
            RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware => {
                RuntimeImmersiveRoomOutcome::RenderObjectsAgainstRoomPolicy
            }
            RuntimeImmersiveObjectRenderingPosture::MetadataOnly => {
                RuntimeImmersiveRoomOutcome::PreserveObjectMetadataOnly
            }
            RuntimeImmersiveObjectRenderingPosture::CollapsedToBed => {
                RuntimeImmersiveRoomOutcome::CollapseObjectsIntoBed
            }
            RuntimeImmersiveObjectRenderingPosture::NotRequested
            | RuntimeImmersiveObjectRenderingPosture::Unavailable => {
                RuntimeImmersiveRoomOutcome::BypassRoomPolicy
            }
        }
    };

    Some(RuntimeImmersiveRoomPolicySummary {
        object_rendering_posture,
        room_policy_class,
        room_policy_authority,
        room_outcome,
        summary: format!(
            "objects={:?} room_class={:?} authority={:?} outcome={:?}",
            object_rendering_posture, room_policy_class, room_policy_authority, room_outcome,
        ),
    })
}

fn runtime_deployment_monitoring_summary_for_spatial(
    target_environment: RuntimeSpatialTargetEnvironment,
    bed_class: RuntimeSpatialBedClass,
    fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
    immersive_room_policy: Option<&RuntimeImmersiveRoomPolicySummary>,
) -> Option<RuntimeDeploymentMonitoringSummary> {
    let immersive_room_policy = immersive_room_policy?;
    let fallback_active = immersive_room_policy.room_policy_class
        == RuntimeRoomPolicyClass::FallbackRoom
        || fallback_outcome.is_some()
        || expanded_fallback_outcome.is_some();

    let deployment_class = if fallback_active {
        RuntimeDeploymentClass::FallbackDeployment
    } else {
        match target_environment {
            RuntimeSpatialTargetEnvironment::SourceLayout => {
                RuntimeDeploymentClass::SourceLayoutDeployment
            }
            RuntimeSpatialTargetEnvironment::CanonicalLayout => {
                RuntimeDeploymentClass::ReferenceSpeakerDeployment
            }
            RuntimeSpatialTargetEnvironment::DeviceLayout => {
                RuntimeDeploymentClass::MonitoringSpeakerDeployment
            }
            RuntimeSpatialTargetEnvironment::CustomEnvironment => {
                RuntimeDeploymentClass::PortableFoldDownDeployment
            }
        }
    };

    let fold_down_policy = if fallback_active {
        match bed_class {
            RuntimeSpatialBedClass::StereoBed => RuntimeFoldDownPolicy::FoldDownToStereoMonitoring,
            RuntimeSpatialBedClass::CanonicalSurroundBed
            | RuntimeSpatialBedClass::CustomDiscreteBed => {
                RuntimeFoldDownPolicy::FoldDownToReferenceBed
            }
        }
    } else {
        match deployment_class {
            RuntimeDeploymentClass::PortableFoldDownDeployment => {
                RuntimeFoldDownPolicy::FoldDownToPortablePreview
            }
            RuntimeDeploymentClass::SourceLayoutDeployment
            | RuntimeDeploymentClass::ReferenceSpeakerDeployment
            | RuntimeDeploymentClass::MonitoringSpeakerDeployment => {
                RuntimeFoldDownPolicy::PreserveDeclaredDeployment
            }
            RuntimeDeploymentClass::FallbackDeployment => {
                RuntimeFoldDownPolicy::BypassDeploymentPolicy
            }
        }
    };

    let monitoring_scene_class = if fallback_active {
        RuntimeMonitoringSceneClass::FallbackScene
    } else {
        match fold_down_policy {
            RuntimeFoldDownPolicy::PreserveDeclaredDeployment => {
                if deployment_class == RuntimeDeploymentClass::MonitoringSpeakerDeployment {
                    RuntimeMonitoringSceneClass::ConfidenceScene
                } else {
                    RuntimeMonitoringSceneClass::ReferenceScene
                }
            }
            RuntimeFoldDownPolicy::FoldDownToReferenceBed
            | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring
            | RuntimeFoldDownPolicy::FoldDownToPortablePreview => {
                RuntimeMonitoringSceneClass::FoldDownScene
            }
            RuntimeFoldDownPolicy::BypassDeploymentPolicy => {
                RuntimeMonitoringSceneClass::FallbackScene
            }
        }
    };

    let monitoring_scene_authority = if fallback_active {
        RuntimeMonitoringSceneAuthority::RuntimeDefault
    } else if deployment_class == RuntimeDeploymentClass::MonitoringSpeakerDeployment {
        RuntimeMonitoringSceneAuthority::HostForwarded
    } else {
        RuntimeMonitoringSceneAuthority::RuntimeDeclared
    };

    let monitoring_outcome = if matches!(
        expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::TerminalExpandedSpatialFailure)
    ) || fallback_outcome
        == Some(RuntimeSpatialFallbackOutcome::TerminalSpatialFailure)
    {
        RuntimeMonitoringOutcome::TerminalMonitoringFailure
    } else if fallback_active {
        RuntimeMonitoringOutcome::BypassMonitoringScene
    } else {
        match fold_down_policy {
            RuntimeFoldDownPolicy::PreserveDeclaredDeployment => {
                RuntimeMonitoringOutcome::MonitorDeclaredDeployment
            }
            RuntimeFoldDownPolicy::FoldDownToReferenceBed
            | RuntimeFoldDownPolicy::FoldDownToStereoMonitoring => {
                RuntimeMonitoringOutcome::MonitorFoldedDownScene
            }
            RuntimeFoldDownPolicy::FoldDownToPortablePreview => {
                RuntimeMonitoringOutcome::MonitorPortablePreview
            }
            RuntimeFoldDownPolicy::BypassDeploymentPolicy => {
                RuntimeMonitoringOutcome::BypassMonitoringScene
            }
        }
    };

    Some(RuntimeDeploymentMonitoringSummary {
        deployment_class,
        fold_down_policy,
        monitoring_scene_class,
        monitoring_scene_authority,
        monitoring_outcome,
        summary: format!(
            "deployment={:?} fold_down={:?} scene={:?} authority={:?} outcome={:?}",
            deployment_class,
            fold_down_policy,
            monitoring_scene_class,
            monitoring_scene_authority,
            monitoring_outcome,
        ),
    })
}

fn runtime_renderer_immersive_export_summary_for_spatial(
    adapter_class: RuntimeSpatialAdapterClass,
    execution_mode: RuntimeSpatialExecutionMode,
    _target_environment: RuntimeSpatialTargetEnvironment,
    fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
    immersive_room_policy: Option<&RuntimeImmersiveRoomPolicySummary>,
    deployment_monitoring: Option<&RuntimeDeploymentMonitoringSummary>,
) -> Option<RuntimeRendererImmersiveExportSummary> {
    let immersive_room_policy = immersive_room_policy?;
    let fallback_active = immersive_room_policy.room_policy_class
        == RuntimeRoomPolicyClass::FallbackRoom
        || deployment_monitoring.is_some_and(|monitoring| {
            monitoring.monitoring_scene_class == RuntimeMonitoringSceneClass::FallbackScene
        })
        || fallback_outcome.is_some()
        || expanded_fallback_outcome.is_some();

    let renderer_capability_posture = if fallback_active {
        RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
    } else if adapter_class == RuntimeSpatialAdapterClass::Renderer
        || execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
    {
        RuntimeRendererCapabilityNegotiationPosture::NegotiatedCompatible
    } else {
        RuntimeRendererCapabilityNegotiationPosture::DeclaredCompatible
    };

    let capability_authority = if fallback_active {
        RuntimeRendererCapabilityAuthority::RuntimeDefault
    } else if adapter_class == RuntimeSpatialAdapterClass::Renderer
        || execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
    {
        RuntimeRendererCapabilityAuthority::RendererAdvisory
    } else if deployment_monitoring.is_some_and(|monitoring| {
        monitoring.deployment_class == RuntimeDeploymentClass::MonitoringSpeakerDeployment
    }) {
        RuntimeRendererCapabilityAuthority::HostForwarded
    } else {
        RuntimeRendererCapabilityAuthority::RuntimeDeclared
    };

    let immersive_export_class = if fallback_active {
        RuntimeImmersiveExportClass::FallbackExport
    } else if immersive_room_policy.object_rendering_posture
        == RuntimeImmersiveObjectRenderingPosture::RoomPolicyAware
    {
        RuntimeImmersiveExportClass::ObjectAwareExport
    } else if deployment_monitoring.is_some_and(|monitoring| {
        monitoring.fold_down_policy != RuntimeFoldDownPolicy::PreserveDeclaredDeployment
    }) {
        RuntimeImmersiveExportClass::MonitoringPreviewExport
    } else {
        RuntimeImmersiveExportClass::BedOnlyExport
    };

    let export_authority = if fallback_active {
        RuntimeImmersiveExportAuthority::RuntimeDefault
    } else if adapter_class == RuntimeSpatialAdapterClass::Renderer
        || execution_mode == RuntimeSpatialExecutionMode::RenderToEnvironment
    {
        RuntimeImmersiveExportAuthority::RendererAdvisory
    } else if deployment_monitoring.is_some_and(|monitoring| {
        monitoring.monitoring_scene_authority == RuntimeMonitoringSceneAuthority::HostForwarded
    }) {
        RuntimeImmersiveExportAuthority::HostForwarded
    } else {
        RuntimeImmersiveExportAuthority::RuntimeDeclared
    };

    let export_outcome = if matches!(
        expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::TerminalExpandedSpatialFailure)
    ) || fallback_outcome
        == Some(RuntimeSpatialFallbackOutcome::TerminalSpatialFailure)
    {
        RuntimeImmersiveExportOutcome::TerminalExportFailure
    } else if fallback_active {
        match immersive_room_policy.room_outcome {
            RuntimeImmersiveRoomOutcome::PreserveObjectMetadataOnly => {
                RuntimeImmersiveExportOutcome::PreserveMetadataOnly
            }
            RuntimeImmersiveRoomOutcome::CollapseObjectsIntoBed => {
                RuntimeImmersiveExportOutcome::CollapseToBedExport
            }
            RuntimeImmersiveRoomOutcome::BypassRoomPolicy
            | RuntimeImmersiveRoomOutcome::RenderObjectsAgainstRoomPolicy => {
                RuntimeImmersiveExportOutcome::BypassImmersiveExport
            }
            RuntimeImmersiveRoomOutcome::TerminalImmersiveFailure => {
                RuntimeImmersiveExportOutcome::TerminalExportFailure
            }
        }
    } else {
        match immersive_export_class {
            RuntimeImmersiveExportClass::BedOnlyExport
            | RuntimeImmersiveExportClass::ObjectAwareExport
            | RuntimeImmersiveExportClass::MonitoringPreviewExport => {
                RuntimeImmersiveExportOutcome::PreserveDeclaredExport
            }
            RuntimeImmersiveExportClass::FallbackExport
            | RuntimeImmersiveExportClass::NoImmersiveExport => {
                RuntimeImmersiveExportOutcome::BypassImmersiveExport
            }
        }
    };

    Some(RuntimeRendererImmersiveExportSummary {
        renderer_capability_posture,
        capability_authority,
        immersive_export_class,
        export_authority,
        export_outcome,
        summary: format!(
            "renderer={:?} capability_authority={:?} export={:?} export_authority={:?} outcome={:?}",
            renderer_capability_posture,
            capability_authority,
            immersive_export_class,
            export_authority,
            export_outcome,
        ),
    })
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
                summary: format!(
                    "node={} adapter={:?} mode={:?} target={:?} controls={:?} policy={:?} fallback={:?}/{:?} bed={:?} objects={:?}/{} mix={:?} render={:?} immersive={:?} monitoring={:?} export={:?} balance={} input={} output={}",
                    node_id,
                    RuntimeSpatialAdapterClass::Balance,
                    execution_mode,
                    target_environment,
                    RuntimeSpatialControlFamily::BalanceScalar,
                    RuntimeSpatialActivationPolicy::EnabledIfSupported,
                    fallback_outcome,
                    expanded_fallback_outcome,
                    bed_class,
                    None::<RuntimeSpatialObjectRole>,
                    object_count,
                    mix_policy,
                    render_scope,
                    immersive_room_policy.as_ref().map(|summary| &summary.summary),
                    deployment_monitoring.as_ref().map(|summary| &summary.summary),
                    renderer_export.as_ref().map(|summary| &summary.summary),
                    balance,
                    input_layout.summary,
                    output_layout.summary,
                ),
            })
        }
        _ => None,
    })
}

impl RuntimeMultichannelIoSummary {
    pub fn new(
        input_layout: RuntimeMultichannelLayoutSummary,
        output_layout: RuntimeMultichannelLayoutSummary,
        input_bus_intent: RuntimeBusIntent,
        output_bus_intent: RuntimeBusIntent,
    ) -> Self {
        let summary = format!(
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
            summary,
        }
    }

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

    pub fn for_plugin_io(layout: PluginIoLayout) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_count(layout.audio_inputs),
            RuntimeMultichannelLayoutSummary::from_channel_count(layout.audio_outputs),
            RuntimeBusIntent::MainProgram,
            RuntimeBusIntent::MainProgram,
        )
    }

    pub fn for_hardware(input_channels: u16, output_channels: u16) -> Self {
        Self::new(
            RuntimeMultichannelLayoutSummary::from_channel_count(input_channels),
            RuntimeMultichannelLayoutSummary::from_channel_count(output_channels),
            RuntimeBusIntent::HardwareInput,
            RuntimeBusIntent::HardwareOutput,
        )
    }
}
