use super::*;

/// Class of preview transform service active for a clip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformServiceClass {
    #[default]
    /// No preview transform service is available.
    Unavailable,
    /// Preview is aligned to the stretch timeline.
    StretchAligned,
    /// Preview is backed by a persisted transform artifact.
    ArtifactBacked,
    /// Preview is using a fallback rendering path.
    Fallback,
}

/// Readiness of the preview transform for a clip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformReadiness {
    #[default]
    /// No transform data is present.
    Empty,
    /// Transform is being prepared.
    Pending,
    /// Transform is ready for playback.
    Ready,
    /// Transform is in a degraded state.
    Degraded,
    /// Transform has been invalidated and must be rebuilt.
    Invalidated,
    /// Preview transforms are not supported for this clip.
    Unsupported,
}

/// Reason a preview transform is degraded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformDegradedState {
    #[default]
    /// Transform is not degraded.
    None,
    /// Transform is waiting for its required inputs to become available.
    PendingInputs,
    /// Transform inputs have been invalidated.
    InvalidatedInputs,
    /// Only a fallback path is available.
    FallbackOnly,
    /// Transform scope is not supported.
    UnsupportedScope,
}

/// Fallback mode for a degraded preview transform.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformFallbackKind {
    #[default]
    /// No fallback is in use.
    None,
    /// Playback falls back to the raw media asset.
    MediaOnly,
    /// Playback falls back to an offline-rendered file.
    OfflineOnly,
}

/// Preview transform snapshot for one clip: service class, readiness,
/// degraded state, fallback, and audition status.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimePreviewTransformClipSnapshot {
    /// Clip identifier.
    pub clip_id: String,
    /// Media asset identifier, if a media asset is associated.
    pub media_asset_id: Option<String>,
    /// Active preview transform service class.
    pub service_class: RuntimePreviewTransformServiceClass,
    /// Current readiness of the preview transform.
    pub readiness: RuntimePreviewTransformReadiness,
    /// Degraded state reason, if applicable.
    pub degraded_state: RuntimePreviewTransformDegradedState,
    /// Fallback kind in use, if any.
    pub fallback_kind: RuntimePreviewTransformFallbackKind,
    /// Artifact reuse state for this clip's transform.
    pub artifact_reuse_state: RuntimeTransformArtifactReuseState,
    /// Whether an audition session is currently active for this clip.
    pub audition_active: bool,
    /// Whether scrub playback is supported for this clip.
    pub scrub_supported: bool,
}

/// Preview output routing posture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewOutputRoutingPosture {
    #[default]
    /// No preview output routing is configured.
    NoPreviewOutputRouting,
    /// Preview is routed to the program output.
    ProgramAlignedPreviewRouting,
    /// Preview is routed to the monitor output.
    MonitorAlignedPreviewRouting,
    /// Preview is using a guarded fallback routing path.
    GuardedPreviewOutputRouting,
    /// Preview output routing is unavailable.
    UnavailablePreviewOutputRouting,
}

/// Class of audio sink used for audition/preview output.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeAuditionSinkClass {
    #[default]
    /// No audition sink is configured.
    NoAuditionSink,
    /// Audition is routed to the program output.
    ProgramOutputSink,
    /// Audition is routed to a monitoring sink.
    MonitoringSink,
    /// Audition is routed to a guarded preview sink.
    GuardedPreviewSink,
    /// Audition sink is unavailable.
    UnavailableAuditionSink,
}

/// Who authorised the audition sink selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeAuditionSinkAuthority {
    #[default]
    /// Sink was selected by the runtime default policy.
    RuntimeDefault,
    /// Sink was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Sink selection was forwarded from the host.
    HostForwarded,
    /// Sink selection is advisory from the device.
    DeviceAdvisory,
}

/// Low-latency device policy class for preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeLowLatencyDevicePolicyClass {
    #[default]
    /// No low-latency device policy is active.
    NoLowLatencyDevicePolicy,
    /// Low-latency policy aligned with the program output path.
    ProgramAlignedLowLatencyPolicy,
    /// Low-latency policy aligned with the monitor output path.
    MonitorAlignedLowLatencyPolicy,
    /// Low-latency policy using a guarded fallback path.
    GuardedLowLatencyDevicePolicy,
    /// Low-latency device policy is unavailable.
    UnavailableLowLatencyDevicePolicy,
}

/// Resolved outcome of the low-latency device policy for preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeLowLatencyDevicePolicyOutcome {
    #[default]
    /// Preview is in observe-only mode; no routing changes applied.
    ObserveOnlyPreview,
    /// Existing program route is preserved.
    PreserveProgramRoute,
    /// Existing monitoring route is preserved.
    PreserveMonitoringRoute,
    /// Routing collapsed to a guarded sink.
    CollapseToGuardedSink,
    /// Preview route has permanently failed.
    TerminalPreviewRouteFailure,
}

/// Resolved preview device policy: routing posture, sink class/authority,
/// and low-latency policy outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePreviewDevicePolicySummary {
    /// Output routing posture for preview playback.
    pub routing_posture: RuntimePreviewOutputRoutingPosture,
    /// Class of audio sink selected for preview audition.
    pub audition_sink_class: RuntimeAuditionSinkClass,
    /// Authority that selected the audition sink.
    pub audition_sink_authority: RuntimeAuditionSinkAuthority,
    /// Low-latency device policy class in effect.
    pub low_latency_device_policy_class: RuntimeLowLatencyDevicePolicyClass,
    /// Resolved outcome of the low-latency device policy.
    pub low_latency_device_policy_outcome: RuntimeLowLatencyDevicePolicyOutcome,
}

/// Preview browser queue posture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewBrowserQueuePosture {
    #[default]
    /// No preview queue is active in the runtime.
    NoRuntimePreviewQueue,
    /// A single active preview request queue is running.
    SingleActivePreviewQueue,
    /// Preview queue is in a guarded fallback state.
    GuardedPreviewQueue,
    /// Preview queue is unavailable.
    UnavailablePreviewQueue,
}

/// Class of the preview browser request queue.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewBrowserQueueClass {
    #[default]
    /// No preview queue is configured.
    NoPreviewQueue,
    /// Queue handles a single asset audition at a time.
    SingleAssetAuditionQueue,
    /// Queue manages asset selection for preview.
    PreviewAssetSelectionQueue,
    /// Queue is operating in a guarded request mode.
    GuardedPreviewRequestQueue,
    /// Preview queue is unavailable.
    UnavailablePreviewQueue,
}

/// Resolved outcome of the preview browser queue policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewBrowserQueueOutcome {
    #[default]
    /// Queue is idle; no requests are pending.
    IdlePreviewQueue,
    /// The active preview request is preserved.
    PreserveActivePreviewRequest,
    /// Multiple requests collapsed to a single active preview.
    CollapseToSingleActivePreview,
    /// Preview queue has permanently failed.
    TerminalPreviewQueueFailure,
}

/// Posture of the media audition orchestration subsystem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAuditionOrchestrationPosture {
    #[default]
    /// No audition orchestration is active.
    NoAuditionOrchestration,
    /// Audition orchestration is handled directly by the runtime.
    DirectRuntimeAuditionOrchestration,
    /// Audition orchestration is in a guarded fallback state.
    GuardedRuntimeAuditionOrchestration,
    /// Audition orchestration is unavailable.
    UnavailableAuditionOrchestration,
}

/// Who authorises the media audition orchestration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAuditionOrchestrationAuthority {
    #[default]
    /// Orchestration authority comes from the runtime default.
    RuntimeDefault,
    /// Orchestration authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Orchestration authority was forwarded from the workflow layer.
    WorkflowForwarded,
    /// Orchestration authority comes from a guarded runtime override.
    GuardedRuntimeOverride,
}

/// Resolved continuity outcome for an active audition.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeMediaAuditionContinuityOutcome {
    #[default]
    /// No audition continuity action is required.
    IdleAuditionContinuity,
    /// The active audition is preserved.
    PreserveActiveAudition,
    /// A paused preview audition is resumed.
    ResumePreviewAudition,
    /// Audition collapsed to a guarded fallback.
    CollapseToGuardedAudition,
    /// Audition has permanently failed.
    TerminalAuditionFailure,
}

/// Posture of the preview transform scheduling subsystem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformSchedulingPosture {
    #[default]
    /// No transform scheduling is active.
    NoTransformScheduling,
    /// Transform scheduling is handled directly by the runtime.
    DirectRuntimeTransformScheduling,
    /// Transform scheduling is in a guarded fallback state.
    GuardedRuntimeTransformScheduling,
    /// Transform scheduling is unavailable.
    UnavailableTransformScheduling,
}

/// Who authorises the preview transform schedule.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformSchedulingAuthority {
    #[default]
    /// Scheduling authority comes from the runtime default.
    RuntimeDefault,
    /// Scheduling authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Scheduling authority is derived from preview demand.
    PreviewDemandDerived,
    /// Scheduling authority comes from a guarded runtime override.
    GuardedRuntimeOverride,
}

/// Resolved outcome of the preview transform scheduling policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimePreviewTransformSchedulingOutcome {
    #[default]
    /// Scheduling is idle; no transforms are pending.
    IdleTransformScheduling,
    /// The ready transform schedule is preserved.
    PreserveReadyTransformSchedule,
    /// Artifact-backed preview is preferred for scheduling.
    PreferArtifactBackedPreview,
    /// Scheduling collapsed to fallback transforms.
    CollapseToFallbackTransforms,
    /// Transform scheduling has permanently failed.
    TerminalTransformSchedulingFailure,
}

/// Summary of the full preview workflow: queue, audition orchestration,
/// and transform scheduling posture/authority/outcome.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePreviewWorkflowSummary {
    /// Browser queue posture.
    pub queue_posture: RuntimePreviewBrowserQueuePosture,
    /// Browser queue class.
    pub queue_class: RuntimePreviewBrowserQueueClass,
    /// Resolved browser queue outcome.
    pub queue_outcome: RuntimePreviewBrowserQueueOutcome,
    /// Media audition orchestration posture.
    pub audition_posture: RuntimeMediaAuditionOrchestrationPosture,
    /// Authority governing audition orchestration.
    pub audition_authority: RuntimeMediaAuditionOrchestrationAuthority,
    /// Resolved audition continuity outcome.
    pub audition_continuity_outcome: RuntimeMediaAuditionContinuityOutcome,
    /// Transform scheduling posture.
    pub transform_scheduling_posture: RuntimePreviewTransformSchedulingPosture,
    /// Authority governing transform scheduling.
    pub transform_scheduling_authority: RuntimePreviewTransformSchedulingAuthority,
    /// Resolved transform scheduling outcome.
    pub transform_scheduling_outcome: RuntimePreviewTransformSchedulingOutcome,
    /// Number of queued preview requests.
    pub queued_preview_request_count: usize,
    /// Number of assets eligible for preview.
    pub previewable_asset_count: usize,
    /// Number of clips with an active audition session.
    pub active_audition_clip_count: usize,
    /// Number of clips with a pending transform.
    pub pending_transform_clip_count: usize,
    /// Number of clips with a ready transform.
    pub ready_transform_clip_count: usize,
    /// Number of clips using a fallback transform.
    pub fallback_transform_clip_count: usize,
}

/// Aggregate snapshot of the preview transform service across all clips.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimePreviewTransformServiceSnapshot {
    /// Total number of clips tracked.
    pub clip_count: usize,
    /// Number of clips with an active audition session.
    pub active_audition_clip_count: usize,
    /// Number of clips that support scrub playback.
    pub scrub_supported_clip_count: usize,
    /// Number of clips with a ready preview transform.
    pub ready_clip_count: usize,
    /// Number of clips with a pending preview transform.
    pub pending_clip_count: usize,
    /// Number of clips with a degraded preview transform.
    pub degraded_clip_count: usize,
    /// Number of clips with an invalidated preview transform.
    pub invalidated_clip_count: usize,
    /// Number of clips where preview is unsupported.
    pub unsupported_clip_count: usize,
    /// Number of clips using stretch-aligned preview.
    pub stretch_aligned_clip_count: usize,
    /// Number of clips using artifact-backed preview.
    pub artifact_backed_clip_count: usize,
    /// Number of clips using fallback preview.
    pub fallback_clip_count: usize,
    /// Resolved preview device policy.
    pub preview_device_policy: RuntimePreviewDevicePolicySummary,
    /// Resolved preview workflow summary.
    pub preview_workflow: RuntimePreviewWorkflowSummary,
    /// Per-clip preview transform snapshots.
    pub clips: Vec<RuntimePreviewTransformClipSnapshot>,
}
