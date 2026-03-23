use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformServiceClass {
    #[default]
    Unavailable,
    StretchAligned,
    ArtifactBacked,
    Fallback,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformReadiness {
    #[default]
    Empty,
    Pending,
    Ready,
    Degraded,
    Invalidated,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformDegradedState {
    #[default]
    None,
    PendingInputs,
    InvalidatedInputs,
    FallbackOnly,
    UnsupportedScope,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformFallbackKind {
    #[default]
    None,
    MediaOnly,
    OfflineOnly,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePreviewTransformClipSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub service_class: RuntimePreviewTransformServiceClass,
    pub readiness: RuntimePreviewTransformReadiness,
    pub degraded_state: RuntimePreviewTransformDegradedState,
    pub fallback_kind: RuntimePreviewTransformFallbackKind,
    pub artifact_reuse_state: RuntimeTransformArtifactReuseState,
    pub audition_active: bool,
    pub scrub_supported: bool,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewOutputRoutingPosture {
    #[default]
    NoPreviewOutputRouting,
    ProgramAlignedPreviewRouting,
    MonitorAlignedPreviewRouting,
    GuardedPreviewOutputRouting,
    UnavailablePreviewOutputRouting,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeAuditionSinkClass {
    #[default]
    NoAuditionSink,
    ProgramOutputSink,
    MonitoringSink,
    GuardedPreviewSink,
    UnavailableAuditionSink,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeAuditionSinkAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    DeviceAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeLowLatencyDevicePolicyClass {
    #[default]
    NoLowLatencyDevicePolicy,
    ProgramAlignedLowLatencyPolicy,
    MonitorAlignedLowLatencyPolicy,
    GuardedLowLatencyDevicePolicy,
    UnavailableLowLatencyDevicePolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeLowLatencyDevicePolicyOutcome {
    #[default]
    ObserveOnlyPreview,
    PreserveProgramRoute,
    PreserveMonitoringRoute,
    CollapseToGuardedSink,
    TerminalPreviewRouteFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePreviewDevicePolicySummary {
    pub routing_posture: RuntimePreviewOutputRoutingPosture,
    pub audition_sink_class: RuntimeAuditionSinkClass,
    pub audition_sink_authority: RuntimeAuditionSinkAuthority,
    pub low_latency_device_policy_class: RuntimeLowLatencyDevicePolicyClass,
    pub low_latency_device_policy_outcome: RuntimeLowLatencyDevicePolicyOutcome,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewBrowserQueuePosture {
    #[default]
    NoRuntimePreviewQueue,
    SingleActivePreviewQueue,
    GuardedPreviewQueue,
    UnavailablePreviewQueue,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewBrowserQueueClass {
    #[default]
    NoPreviewQueue,
    SingleAssetAuditionQueue,
    PreviewAssetSelectionQueue,
    GuardedPreviewRequestQueue,
    UnavailablePreviewQueue,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewBrowserQueueOutcome {
    #[default]
    IdlePreviewQueue,
    PreserveActivePreviewRequest,
    CollapseToSingleActivePreview,
    TerminalPreviewQueueFailure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAuditionOrchestrationPosture {
    #[default]
    NoAuditionOrchestration,
    DirectRuntimeAuditionOrchestration,
    GuardedRuntimeAuditionOrchestration,
    UnavailableAuditionOrchestration,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAuditionOrchestrationAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    WorkflowForwarded,
    GuardedRuntimeOverride,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAuditionContinuityOutcome {
    #[default]
    IdleAuditionContinuity,
    PreserveActiveAudition,
    ResumePreviewAudition,
    CollapseToGuardedAudition,
    TerminalAuditionFailure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformSchedulingPosture {
    #[default]
    NoTransformScheduling,
    DirectRuntimeTransformScheduling,
    GuardedRuntimeTransformScheduling,
    UnavailableTransformScheduling,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformSchedulingAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    PreviewDemandDerived,
    GuardedRuntimeOverride,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformSchedulingOutcome {
    #[default]
    IdleTransformScheduling,
    PreserveReadyTransformSchedule,
    PreferArtifactBackedPreview,
    CollapseToFallbackTransforms,
    TerminalTransformSchedulingFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePreviewWorkflowSummary {
    pub queue_posture: RuntimePreviewBrowserQueuePosture,
    pub queue_class: RuntimePreviewBrowserQueueClass,
    pub queue_outcome: RuntimePreviewBrowserQueueOutcome,
    pub audition_posture: RuntimeMediaAuditionOrchestrationPosture,
    pub audition_authority: RuntimeMediaAuditionOrchestrationAuthority,
    pub audition_continuity_outcome: RuntimeMediaAuditionContinuityOutcome,
    pub transform_scheduling_posture: RuntimePreviewTransformSchedulingPosture,
    pub transform_scheduling_authority: RuntimePreviewTransformSchedulingAuthority,
    pub transform_scheduling_outcome: RuntimePreviewTransformSchedulingOutcome,
    pub queued_preview_request_count: usize,
    pub previewable_asset_count: usize,
    pub active_audition_clip_count: usize,
    pub pending_transform_clip_count: usize,
    pub ready_transform_clip_count: usize,
    pub fallback_transform_clip_count: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePreviewTransformServiceSnapshot {
    pub clip_count: usize,
    pub active_audition_clip_count: usize,
    pub scrub_supported_clip_count: usize,
    pub ready_clip_count: usize,
    pub pending_clip_count: usize,
    pub degraded_clip_count: usize,
    pub invalidated_clip_count: usize,
    pub unsupported_clip_count: usize,
    pub stretch_aligned_clip_count: usize,
    pub artifact_backed_clip_count: usize,
    pub fallback_clip_count: usize,
    pub preview_device_policy: RuntimePreviewDevicePolicySummary,
    pub preview_workflow: RuntimePreviewWorkflowSummary,
    pub clips: Vec<RuntimePreviewTransformClipSnapshot>,
    pub summary: String,
}
