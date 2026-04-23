use super::super::*;

/// Class of spatial adapter: balance scalar, per-channel gain, layout transform, or immersive renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialAdapterClass {
    #[default]
    /// Simple stereo balance scalar adapter.
    Balance,
    /// Per-channel gain adapter.
    PerChannelGain,
    /// Channel layout transform adapter.
    LayoutTransform,
    /// Full immersive renderer adapter.
    Renderer,
}

/// Active spatial execution mode: bypassed, balance groups, per-channel attenuation, layout transform, or full renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialExecutionMode {
    #[default]
    /// Spatial processing is bypassed.
    Bypassed,
    /// Processing using balance groups.
    BalanceGroups,
    /// Processing using per-channel attenuation.
    PerChannelAttenuation,
    /// Transforming audio to a target layout.
    TransformToTargetLayout,
    /// Rendering to a spatial environment.
    RenderToEnvironment,
}

/// Target output environment for spatial rendering: source layout, canonical speaker layout, device layout, or custom.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialTargetEnvironment {
    #[default]
    /// Output matches the source channel layout.
    SourceLayout,
    /// Output targets a canonical (standard) speaker layout.
    CanonicalLayout,
    /// Output targets the physical device layout.
    DeviceLayout,
    /// Output targets a custom environment definition.
    CustomEnvironment,
}

/// Control parameter family driving spatial processing: balance scalar, per-channel vector, or adapter parameter set.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialControlFamily {
    #[default]
    /// A single scalar balance value drives processing.
    BalanceScalar,
    /// A per-channel gain vector drives processing.
    PerChannelVector,
    /// An adapter-specific parameter set drives processing.
    AdapterParameterSet,
}

/// Policy governing whether spatial processing is activated: disabled, enabled-if-supported, or required.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialActivationPolicy {
    /// Spatial processing is always disabled.
    Disabled,
    #[default]
    /// Spatial processing is enabled when the adapter supports it.
    EnabledIfSupported,
    /// Spatial processing must be active; failure is an error.
    Required,
}

/// Fallback outcome when spatial processing cannot operate at the requested level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialFallbackOutcome {
    /// Spatial processing is bypassed entirely.
    BypassSpatialProcessing,
    /// Processing collapsed to a simple balance operation.
    CollapseToBalance,
    /// Processing collapsed to per-channel gain only.
    CollapseToPerChannelGain,
    /// Processing degraded to safe-mode behaviour.
    SafeModeDegradation,
    /// Spatial processing has permanently failed.
    TerminalSpatialFailure,
}

/// Speaker bed class: stereo, canonical surround, or custom discrete layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialBedClass {
    #[default]
    /// Standard stereo (L/R) bed.
    StereoBed,
    /// Canonical surround bed (e.g. 5.1, 7.1).
    CanonicalSurroundBed,
    /// Custom discrete channel bed.
    CustomDiscreteBed,
}

/// Role of an audio object within an immersive spatial mix.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialObjectRole {
    /// The primary foreground audio object.
    PrimaryObject,
    /// An auxiliary (support) audio object.
    AuxiliaryObject,
    /// A spatial effects object.
    EffectObject,
    /// An analysis-only object not routed to the output.
    AnalysisObject,
}

/// Mix policy for combining bed and object layers in an immersive render.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialMixPolicy {
    #[default]
    /// Render the bed channel only; no objects.
    BedOnly,
    /// Render the bed with audio objects mixed in.
    BedWithObjects,
    /// Prefer objects, fall back to bed if objects are unavailable.
    ObjectPreferredWithBedFallback,
    /// Downmix to the canonical bed layout.
    DownmixToCanonicalBed,
    /// Collapse to a baseline (non-immersive) spatial output.
    CollapseToBaselineSpatial,
}

/// Scope of the active spatial render pass: bed only, bed+objects, fold-down, or object metadata only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSpatialRenderScope {
    #[default]
    /// Render the bed channel only.
    BedRender,
    /// Render the bed and audio objects together.
    BedAndObjectRender,
    /// Fold the bed down to a lower channel count.
    BedFoldDownRender,
    /// Extract object metadata only; no audio rendering.
    ObjectMetadataOnly,
}

/// Fallback outcome when expanded (bed+object) spatial rendering cannot be fully satisfied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeSpatialExpandedFallbackOutcome {
    /// Objects are folded into the bed channel.
    CollapseObjectsIntoBed,
    /// Output collapsed to the canonical bed layout.
    CollapseToCanonicalBed,
    /// Output collapsed to a baseline spatial output.
    CollapseToBaselineSpatial,
    /// Expanded spatial rendering is bypassed.
    BypassExpandedSpatial,
    /// Expanded spatial rendering has permanently failed.
    TerminalExpandedSpatialFailure,
}

/// Rendering posture for immersive audio objects: not requested, metadata-only, room-policy-aware, collapsed to bed, or unavailable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveObjectRenderingPosture {
    #[default]
    /// Object rendering has not been requested.
    NotRequested,
    /// Only object metadata is extracted; no audio rendering.
    MetadataOnly,
    /// Objects are rendered with room policy applied.
    RoomPolicyAware,
    /// Objects have been collapsed into the bed channel.
    CollapsedToBed,
    /// Object rendering is unavailable.
    Unavailable,
}

/// Room policy class applied during immersive rendering: none, reference room, monitoring room, deployment room, or fallback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRoomPolicyClass {
    #[default]
    /// No room policy is applied.
    NoRoomPolicy,
    /// Reference listening room policy.
    ReferenceRoom,
    /// Monitoring room policy.
    MonitoringRoom,
    /// Deployment-specific room policy.
    DeploymentRoom,
    /// Fallback room policy.
    FallbackRoom,
}

/// Who authorises the immersive room policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRoomPolicyAuthority {
    #[default]
    /// Authority comes from the runtime default.
    RuntimeDefault,
    /// Authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Authority was forwarded from the host.
    HostForwarded,
    /// Authority is advisory from the renderer.
    RendererAdvisory,
}

/// Resolved outcome of immersive room policy application.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveRoomOutcome {
    #[default]
    /// Room policy is bypassed.
    BypassRoomPolicy,
    /// Objects are rendered against the room policy.
    RenderObjectsAgainstRoomPolicy,
    /// Only object metadata is preserved; room policy not applied.
    PreserveObjectMetadataOnly,
    /// Objects are collapsed into the bed channel.
    CollapseObjectsIntoBed,
    /// Immersive rendering has permanently failed.
    TerminalImmersiveFailure,
}

/// Summary of the immersive room policy applied to object rendering: posture, class, authority, and outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeImmersiveRoomPolicySummary {
    /// Object rendering posture.
    pub object_rendering_posture: RuntimeImmersiveObjectRenderingPosture,
    /// Room policy class in effect.
    pub room_policy_class: RuntimeRoomPolicyClass,
    /// Authority that determined the active room policy.
    pub room_policy_authority: RuntimeRoomPolicyAuthority,
    /// Resolved immersive room policy outcome.
    pub room_outcome: RuntimeImmersiveRoomOutcome,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Speaker deployment class: source layout, reference speaker, monitoring speaker, portable fold-down, or fallback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeDeploymentClass {
    #[default]
    /// Deploy to the source channel layout.
    SourceLayoutDeployment,
    /// Deploy to a reference speaker configuration.
    ReferenceSpeakerDeployment,
    /// Deploy to a monitoring speaker configuration.
    MonitoringSpeakerDeployment,
    /// Deploy to a portable fold-down configuration.
    PortableFoldDownDeployment,
    /// Fallback deployment class.
    FallbackDeployment,
}

/// Fold-down policy applied to an immersive deployment when the target speaker count is lower than the source.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeFoldDownPolicy {
    #[default]
    /// Preserve the declared deployment without fold-down.
    PreserveDeclaredDeployment,
    /// Fold down to a reference bed layout.
    FoldDownToReferenceBed,
    /// Fold down to a stereo monitoring mix.
    FoldDownToStereoMonitoring,
    /// Fold down to a portable preview mix.
    FoldDownToPortablePreview,
    /// Bypass the deployment fold-down policy.
    BypassDeploymentPolicy,
}

/// Class of monitoring scene applied during playback: none, reference, fold-down, confidence, or fallback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringSceneClass {
    #[default]
    /// No monitoring scene is active.
    NoMonitoringScene,
    /// Reference monitoring scene.
    ReferenceScene,
    /// Fold-down monitoring scene.
    FoldDownScene,
    /// Confidence monitoring scene.
    ConfidenceScene,
    /// Fallback monitoring scene.
    FallbackScene,
}

/// Who authorises the monitoring scene selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringSceneAuthority {
    #[default]
    /// Authority comes from the runtime default.
    RuntimeDefault,
    /// Authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Authority was forwarded from the host.
    HostForwarded,
    /// Authority is advisory from the renderer.
    RendererAdvisory,
}

/// Resolved monitoring scene outcome: monitor deployment, fold-down, portable preview, bypassed, or terminal failure.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMonitoringOutcome {
    /// Monitor the declared deployment directly.
    MonitorDeclaredDeployment,
    /// Monitor a folded-down scene.
    MonitorFoldedDownScene,
    /// Monitor a portable preview mix.
    MonitorPortablePreview,
    #[default]
    /// Monitoring scene is bypassed.
    BypassMonitoringScene,
    /// Monitoring has permanently failed.
    TerminalMonitoringFailure,
}

/// Summary of deployment and monitoring configuration: deployment class, fold-down policy, scene class, authority, and outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeDeploymentMonitoringSummary {
    /// Speaker deployment class in effect.
    pub deployment_class: RuntimeDeploymentClass,
    /// Fold-down policy in effect.
    pub fold_down_policy: RuntimeFoldDownPolicy,
    /// Monitoring scene class in effect.
    pub monitoring_scene_class: RuntimeMonitoringSceneClass,
    /// Authority that determined the monitoring scene.
    pub monitoring_scene_authority: RuntimeMonitoringSceneAuthority,
    /// Resolved monitoring outcome.
    pub monitoring_outcome: RuntimeMonitoringOutcome,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Renderer capability negotiation posture: not requested, declared/negotiated compatible, fallback negotiation, or unavailable.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRendererCapabilityNegotiationPosture {
    #[default]
    /// Capability negotiation has not been requested.
    NotRequested,
    /// Renderer declared itself compatible.
    DeclaredCompatible,
    /// Compatibility was confirmed through negotiation.
    NegotiatedCompatible,
    /// Negotiation fell back to a reduced capability set.
    FallbackNegotiation,
    /// Renderer capability negotiation is unavailable.
    Unavailable,
}

/// Who authorises the renderer capability negotiation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeRendererCapabilityAuthority {
    #[default]
    /// Authority comes from the runtime default.
    RuntimeDefault,
    /// Authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Authority was forwarded from the host.
    HostForwarded,
    /// Authority is advisory from the renderer.
    RendererAdvisory,
}

/// Class of immersive export: none, bed-only, object-aware, monitoring preview, or fallback.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportClass {
    #[default]
    /// No immersive export is configured.
    NoImmersiveExport,
    /// Export includes bed channels only.
    BedOnlyExport,
    /// Export includes both bed and object audio.
    ObjectAwareExport,
    /// Export is a monitoring preview mix.
    MonitoringPreviewExport,
    /// Fallback export class.
    FallbackExport,
}

/// Who authorises the immersive export class.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportAuthority {
    #[default]
    /// Authority comes from the runtime default.
    RuntimeDefault,
    /// Authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Authority was forwarded from the host.
    HostForwarded,
    /// Authority is advisory from the renderer.
    RendererAdvisory,
}

/// Resolved outcome of the immersive export policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeImmersiveExportOutcome {
    /// The declared export configuration is preserved.
    PreserveDeclaredExport,
    /// Export collapsed to bed channels only.
    CollapseToBedExport,
    /// Only object metadata is preserved in the export.
    PreserveMetadataOnly,
    #[default]
    /// Immersive export is bypassed.
    BypassImmersiveExport,
    /// Immersive export has permanently failed.
    TerminalExportFailure,
}

/// Summary of immersive renderer export: capability posture/authority, export class/authority, and resolved export outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeRendererImmersiveExportSummary {
    /// Renderer capability negotiation posture.
    pub renderer_capability_posture: RuntimeRendererCapabilityNegotiationPosture,
    /// Authority that determined the capability negotiation outcome.
    pub capability_authority: RuntimeRendererCapabilityAuthority,
    /// Immersive export class in effect.
    pub immersive_export_class: RuntimeImmersiveExportClass,
    /// Authority that determined the export class.
    pub export_authority: RuntimeImmersiveExportAuthority,
    /// Resolved immersive export outcome.
    pub export_outcome: RuntimeImmersiveExportOutcome,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Full spatial execution summary for a graph node: adapter class, execution mode, target environment, control family, bed/object layout, mix policy, render scope, and optional immersive sub-summaries.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpatialExecutionSummary {
    /// Graph node identifier.
    pub node_id: String,
    /// Spatial adapter class in use for this node.
    pub adapter_class: RuntimeSpatialAdapterClass,
    /// Active spatial execution mode.
    pub execution_mode: RuntimeSpatialExecutionMode,
    /// Target output environment for spatial rendering.
    pub target_environment: RuntimeSpatialTargetEnvironment,
    /// Control parameter family driving spatial processing.
    pub control_family: RuntimeSpatialControlFamily,
    /// Activation policy for spatial processing.
    pub activation_policy: RuntimeSpatialActivationPolicy,
    /// Fallback outcome when spatial processing cannot be fully satisfied, if applicable.
    pub fallback_outcome: Option<RuntimeSpatialFallbackOutcome>,
    /// Speaker bed class for this node.
    pub bed_class: RuntimeSpatialBedClass,
    /// Audio object role, if this node carries an immersive object.
    pub object_role: Option<RuntimeSpatialObjectRole>,
    /// Number of audio objects associated with this node.
    pub object_count: usize,
    /// Mix policy for combining bed and object layers.
    pub mix_policy: RuntimeSpatialMixPolicy,
    /// Scope of the active render pass.
    pub render_scope: RuntimeSpatialRenderScope,
    /// Fallback outcome for expanded spatial rendering, if applicable.
    pub expanded_fallback_outcome: Option<RuntimeSpatialExpandedFallbackOutcome>,
    /// Immersive room policy summary, if applicable.
    pub immersive_room_policy: Option<RuntimeImmersiveRoomPolicySummary>,
    /// Deployment and monitoring summary, if applicable.
    pub deployment_monitoring: Option<RuntimeDeploymentMonitoringSummary>,
    /// Renderer immersive export summary, if applicable.
    pub renderer_export: Option<RuntimeRendererImmersiveExportSummary>,
    /// Balance descriptor string, if a balance adapter is active.
    pub balance: Option<String>,
    /// Multichannel input layout summary.
    pub input_layout: RuntimeMultichannelLayoutSummary,
    /// Multichannel output layout summary.
    pub output_layout: RuntimeMultichannelLayoutSummary,
    /// Human-readable one-line summary.
    pub summary: String,
}
