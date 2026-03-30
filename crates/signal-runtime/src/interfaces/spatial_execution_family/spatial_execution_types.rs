use super::super::*;

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
