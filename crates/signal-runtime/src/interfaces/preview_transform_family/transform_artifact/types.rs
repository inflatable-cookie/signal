use super::*;

/// Readiness of a cached transform artifact for a clip.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformArtifactReadiness {
    #[default]
    /// No artifact data is present.
    Empty,
    /// Waiting for the media asset to become available.
    PendingMedia,
    /// Artifact is ready for use.
    Ready,
    /// Artifact is in a degraded state.
    Degraded,
    /// Artifact has been invalidated and must be rebuilt.
    Invalidated,
    /// Transform artifacts are not supported for this clip.
    Unsupported,
}

/// Reason a transform artifact was invalidated.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformArtifactInvalidationState {
    #[default]
    /// Artifact is not invalidated.
    None,
    /// The underlying media asset was invalidated.
    MediaInvalidated,
    /// The stretch parameters were changed, invalidating the artifact.
    StretchInvalidated,
    /// Marker analysis results were invalidated.
    AnalysisInvalidated,
}

/// Whether an existing transform artifact can be reused for preview.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformArtifactReuseState {
    #[default]
    /// No artifact is available for reuse.
    Unavailable,
    /// Artifact exists but must be re-rendered before use.
    RequiresRender,
    /// Artifact can be reused directly.
    Reusable,
    /// Artifact is in a guarded reuse state.
    Guarded,
}

/// Transform artifact snapshot for one clip: readiness, invalidation, reuse
/// state, and underlying stretch/marker analysis states.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeTransformArtifactClipSnapshot {
    /// Clip identifier.
    pub clip_id: String,
    /// Media asset identifier, if a media asset is associated.
    pub media_asset_id: Option<String>,
    /// Stable identity string for the current artifact.
    pub artifact_identity: String,
    /// Readiness of the transform artifact.
    pub readiness: RuntimeTransformArtifactReadiness,
    /// Invalidation state of the transform artifact.
    pub invalidation_state: RuntimeTransformArtifactInvalidationState,
    /// Whether and how the artifact can be reused.
    pub reuse_state: RuntimeTransformArtifactReuseState,
    /// Whether cached media data is ready for this clip.
    pub cached_media_ready: bool,
    /// Stretch engine class in use for this clip.
    pub stretch_engine_class: RuntimeStretchEngineClass,
    /// Readiness of the stretch engine for this clip.
    pub stretch_readiness: RuntimeStretchReadiness,
    /// Readiness of the marker analysis for this clip.
    pub marker_analysis_readiness: RuntimeMarkerAnalysisReadiness,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Aggregate transform artifact snapshot across all clips with persistence summary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTransformArtifactSnapshot {
    /// Total number of clips tracked.
    pub clip_count: usize,
    /// Number of clips with a ready transform artifact.
    pub ready_clip_count: usize,
    /// Number of clips waiting for media to become available.
    pub pending_media_clip_count: usize,
    /// Number of clips with a degraded transform artifact.
    pub degraded_clip_count: usize,
    /// Number of clips with an invalidated transform artifact.
    pub invalidated_clip_count: usize,
    /// Number of clips where transform artifacts are unsupported.
    pub unsupported_clip_count: usize,
    /// Number of clips with cached media ready.
    pub cached_media_ready_clip_count: usize,
    /// Number of clips with a directly reusable artifact.
    pub reusable_clip_count: usize,
    /// Number of clips whose artifact requires re-rendering before reuse.
    pub requires_render_clip_count: usize,
    /// Number of clips with a guarded reuse artifact.
    pub guarded_reuse_clip_count: usize,
    /// Persistence summary for transform artifacts.
    pub transform_persistence: RuntimeTransformPersistenceSummary,
    /// Per-clip transform artifact snapshots.
    pub clips: Vec<RuntimeTransformArtifactClipSnapshot>,
    /// Human-readable one-line summary.
    pub summary: String,
}

/// Persistence posture for transform artifacts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformPersistencePosture {
    #[default]
    /// No transform persistence is configured.
    NoTransformPersistence,
    /// Artifacts are persisted within the scope of the media asset.
    AssetScopedTransformPersistence,
    /// Persistence is in a guarded fallback state.
    GuardedTransformPersistence,
    /// Transform persistence is unavailable.
    UnavailableTransformPersistence,
}

/// Retention policy class for transform artifacts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformRetentionPolicyClass {
    #[default]
    /// No retention policy is active.
    NoTransformRetentionPolicy,
    /// Artifacts are retained for the lifetime of the media asset.
    AssetLifetimeRetentionPolicy,
    /// Retention follows a session-scoped hint.
    SessionHintRetentionPolicy,
    /// Retention is in a guarded fallback state.
    GuardedTransformRetentionPolicy,
    /// Retention policy is unavailable.
    UnavailableTransformRetentionPolicy,
}

/// Who authorises the transform artifact retention policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformRetentionAuthority {
    #[default]
    /// Authority comes from the runtime default.
    RuntimeDefault,
    /// Authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Authority was forwarded from the workflow layer.
    WorkflowForwarded,
    /// Authority is advisory from the cache substrate.
    CacheSubstrateAdvisory,
}

/// Resolved outcome of the transform artifact retention policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformRetentionOutcome {
    #[default]
    /// No retention action is required.
    IdleTransformRetention,
    /// Asset-scoped transforms are preserved.
    PreserveAssetScopedTransforms,
    /// Retention is in a guarded fallback state.
    GuardedTransformRetention,
    /// Invalidated transforms are evicted from the cache.
    EvictInvalidatedTransforms,
    /// Retention policy has permanently failed.
    TerminalTransformRetentionFailure,
}

/// Cache placement posture for transform artifacts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformCachePlacementPosture {
    #[default]
    /// No cache placement is configured.
    NoCachePlacement,
    /// Artifacts are placed at the runtime cache root.
    RuntimeCacheRootPlacement,
    /// Cache placement is in a guarded fallback state.
    GuardedCachePlacement,
    /// Cache placement is unavailable.
    UnavailableCachePlacement,
}

/// Who authorises the transform artifact cache placement.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformCachePlacementAuthority {
    #[default]
    /// Authority comes from the runtime default.
    RuntimeDefault,
    /// Authority was explicitly declared by the runtime.
    RuntimeDeclared,
    /// Authority was forwarded from the host.
    HostForwarded,
    /// Authority is advisory from the cache substrate.
    CacheSubstrateAdvisory,
}

/// Resolved outcome of the transform artifact cache placement policy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformCachePlacementOutcome {
    #[default]
    /// No cache placement action is required.
    IdleCachePlacement,
    /// The runtime cache root is preserved.
    PreserveRuntimeCacheRoot,
    /// Cache placement collapsed to a guarded path.
    CollapseToGuardedCachePlacement,
    /// Cache placement has permanently failed.
    TerminalCachePlacementFailure,
}

/// Summary of transform artifact persistence: retention policy, cache placement,
/// and per-clip counts.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTransformPersistenceSummary {
    /// Overall persistence posture for transform artifacts.
    pub persistence_posture: RuntimeTransformPersistencePosture,
    /// Retention policy class governing artifact lifetime.
    pub retention_policy_class: RuntimeTransformRetentionPolicyClass,
    /// Authority that determined the active retention policy.
    pub retention_authority: RuntimeTransformRetentionAuthority,
    /// Resolved outcome of the retention policy.
    pub retention_outcome: RuntimeTransformRetentionOutcome,
    /// Cache placement posture for persisted transform artifacts.
    pub cache_placement_posture: RuntimeTransformCachePlacementPosture,
    /// Authority that determined the active cache placement policy.
    pub cache_placement_authority: RuntimeTransformCachePlacementAuthority,
    /// Resolved outcome of the cache placement policy.
    pub cache_placement_outcome: RuntimeTransformCachePlacementOutcome,
    /// Root path used for the transform artifact cache.
    pub cache_root_path: String,
    /// Number of clips with fully persisted transform artifacts.
    pub persistent_clip_count: usize,
    /// Number of clips with guarded (protected) persistence entries.
    pub guarded_persistence_clip_count: usize,
    /// Number of clips whose persisted artifacts have been invalidated.
    pub invalidated_persistence_clip_count: usize,
    /// Human-readable one-line summary.
    pub summary: String,
}
