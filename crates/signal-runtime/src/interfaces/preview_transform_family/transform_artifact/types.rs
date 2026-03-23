use super::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformArtifactReadiness {
    #[default]
    Empty,
    PendingMedia,
    Ready,
    Degraded,
    Invalidated,
    Unsupported,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformArtifactInvalidationState {
    #[default]
    None,
    MediaInvalidated,
    StretchInvalidated,
    AnalysisInvalidated,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformArtifactReuseState {
    #[default]
    Unavailable,
    RequiresRender,
    Reusable,
    Guarded,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeTransformArtifactClipSnapshot {
    pub clip_id: String,
    pub media_asset_id: Option<String>,
    pub artifact_identity: String,
    pub readiness: RuntimeTransformArtifactReadiness,
    pub invalidation_state: RuntimeTransformArtifactInvalidationState,
    pub reuse_state: RuntimeTransformArtifactReuseState,
    pub cached_media_ready: bool,
    pub stretch_engine_class: RuntimeStretchEngineClass,
    pub stretch_readiness: RuntimeStretchReadiness,
    pub marker_analysis_readiness: RuntimeMarkerAnalysisReadiness,
    pub summary: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTransformArtifactSnapshot {
    pub clip_count: usize,
    pub ready_clip_count: usize,
    pub pending_media_clip_count: usize,
    pub degraded_clip_count: usize,
    pub invalidated_clip_count: usize,
    pub unsupported_clip_count: usize,
    pub cached_media_ready_clip_count: usize,
    pub reusable_clip_count: usize,
    pub requires_render_clip_count: usize,
    pub guarded_reuse_clip_count: usize,
    pub transform_persistence: RuntimeTransformPersistenceSummary,
    pub clips: Vec<RuntimeTransformArtifactClipSnapshot>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformPersistencePosture {
    #[default]
    NoTransformPersistence,
    AssetScopedTransformPersistence,
    GuardedTransformPersistence,
    UnavailableTransformPersistence,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformRetentionPolicyClass {
    #[default]
    NoTransformRetentionPolicy,
    AssetLifetimeRetentionPolicy,
    SessionHintRetentionPolicy,
    GuardedTransformRetentionPolicy,
    UnavailableTransformRetentionPolicy,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformRetentionAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    WorkflowForwarded,
    CacheSubstrateAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformRetentionOutcome {
    #[default]
    IdleTransformRetention,
    PreserveAssetScopedTransforms,
    GuardedTransformRetention,
    EvictInvalidatedTransforms,
    TerminalTransformRetentionFailure,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformCachePlacementPosture {
    #[default]
    NoCachePlacement,
    RuntimeCacheRootPlacement,
    GuardedCachePlacement,
    UnavailableCachePlacement,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformCachePlacementAuthority {
    #[default]
    RuntimeDefault,
    RuntimeDeclared,
    HostForwarded,
    CacheSubstrateAdvisory,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeTransformCachePlacementOutcome {
    #[default]
    IdleCachePlacement,
    PreserveRuntimeCacheRoot,
    CollapseToGuardedCachePlacement,
    TerminalCachePlacementFailure,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTransformPersistenceSummary {
    pub persistence_posture: RuntimeTransformPersistencePosture,
    pub retention_policy_class: RuntimeTransformRetentionPolicyClass,
    pub retention_authority: RuntimeTransformRetentionAuthority,
    pub retention_outcome: RuntimeTransformRetentionOutcome,
    pub cache_placement_posture: RuntimeTransformCachePlacementPosture,
    pub cache_placement_authority: RuntimeTransformCachePlacementAuthority,
    pub cache_placement_outcome: RuntimeTransformCachePlacementOutcome,
    pub cache_root_path: String,
    pub persistent_clip_count: usize,
    pub guarded_persistence_clip_count: usize,
    pub invalidated_persistence_clip_count: usize,
    pub summary: String,
}
