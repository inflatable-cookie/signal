use super::super::*;

pub(super) fn derive_transform_persistence(
    media_pipeline: &RuntimeMediaPipelineSnapshot,
    reusable_clip_count: usize,
    requires_render_clip_count: usize,
    guarded_reuse_clip_count: usize,
    invalidated_clip_count: usize,
    unsupported_clip_count: usize,
) -> RuntimeTransformPersistenceSummary {
    let has_cache_root = !media_pipeline.cache_root_path.trim().is_empty();
    let persistent_clip_count = reusable_clip_count;
    let guarded_persistence_clip_count = guarded_reuse_clip_count + requires_render_clip_count;
    let invalidated_persistence_clip_count = invalidated_clip_count;

    let (
        persistence_posture,
        retention_policy_class,
        retention_authority,
        retention_outcome,
        cache_placement_posture,
        cache_placement_authority,
        cache_placement_outcome,
    ) = if persistent_clip_count == 0
        && guarded_persistence_clip_count == 0
        && invalidated_persistence_clip_count == 0
        && unsupported_clip_count == 0
    {
        (
            RuntimeTransformPersistencePosture::NoTransformPersistence,
            RuntimeTransformRetentionPolicyClass::NoTransformRetentionPolicy,
            RuntimeTransformRetentionAuthority::RuntimeDefault,
            RuntimeTransformRetentionOutcome::IdleTransformRetention,
            RuntimeTransformCachePlacementPosture::NoCachePlacement,
            RuntimeTransformCachePlacementAuthority::RuntimeDefault,
            RuntimeTransformCachePlacementOutcome::IdleCachePlacement,
        )
    } else if !has_cache_root {
        (
            RuntimeTransformPersistencePosture::UnavailableTransformPersistence,
            RuntimeTransformRetentionPolicyClass::UnavailableTransformRetentionPolicy,
            RuntimeTransformRetentionAuthority::CacheSubstrateAdvisory,
            RuntimeTransformRetentionOutcome::TerminalTransformRetentionFailure,
            RuntimeTransformCachePlacementPosture::UnavailableCachePlacement,
            RuntimeTransformCachePlacementAuthority::CacheSubstrateAdvisory,
            RuntimeTransformCachePlacementOutcome::TerminalCachePlacementFailure,
        )
    } else if invalidated_persistence_clip_count > 0 {
        (
            RuntimeTransformPersistencePosture::GuardedTransformPersistence,
            RuntimeTransformRetentionPolicyClass::GuardedTransformRetentionPolicy,
            RuntimeTransformRetentionAuthority::CacheSubstrateAdvisory,
            RuntimeTransformRetentionOutcome::EvictInvalidatedTransforms,
            RuntimeTransformCachePlacementPosture::GuardedCachePlacement,
            RuntimeTransformCachePlacementAuthority::RuntimeDefault,
            RuntimeTransformCachePlacementOutcome::CollapseToGuardedCachePlacement,
        )
    } else if guarded_persistence_clip_count > 0 || unsupported_clip_count > 0 {
        (
            RuntimeTransformPersistencePosture::GuardedTransformPersistence,
            RuntimeTransformRetentionPolicyClass::SessionHintRetentionPolicy,
            RuntimeTransformRetentionAuthority::RuntimeDefault,
            RuntimeTransformRetentionOutcome::GuardedTransformRetention,
            RuntimeTransformCachePlacementPosture::GuardedCachePlacement,
            RuntimeTransformCachePlacementAuthority::RuntimeDefault,
            RuntimeTransformCachePlacementOutcome::CollapseToGuardedCachePlacement,
        )
    } else {
        (
            RuntimeTransformPersistencePosture::AssetScopedTransformPersistence,
            RuntimeTransformRetentionPolicyClass::AssetLifetimeRetentionPolicy,
            RuntimeTransformRetentionAuthority::RuntimeDefault,
            RuntimeTransformRetentionOutcome::PreserveAssetScopedTransforms,
            RuntimeTransformCachePlacementPosture::RuntimeCacheRootPlacement,
            RuntimeTransformCachePlacementAuthority::RuntimeDefault,
            RuntimeTransformCachePlacementOutcome::PreserveRuntimeCacheRoot,
        )
    };

    RuntimeTransformPersistenceSummary {
        persistence_posture,
        retention_policy_class,
        retention_authority,
        retention_outcome,
        cache_placement_posture,
        cache_placement_authority,
        cache_placement_outcome,
        cache_root_path: media_pipeline.cache_root_path.clone(),
        persistent_clip_count,
        guarded_persistence_clip_count,
        invalidated_persistence_clip_count,
        summary: format!(
            "persistence={:?} retention={:?}/{:?}/{:?} cache={:?}/{:?}/{:?} persistent={} guarded={} invalidated={} cache_root={}",
            persistence_posture,
            retention_policy_class,
            retention_authority,
            retention_outcome,
            cache_placement_posture,
            cache_placement_authority,
            cache_placement_outcome,
            persistent_clip_count,
            guarded_persistence_clip_count,
            invalidated_persistence_clip_count,
            media_pipeline.cache_root_path,
        ),
    }
}
