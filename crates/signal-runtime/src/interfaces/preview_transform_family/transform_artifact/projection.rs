use super::*;
impl RuntimeTransformArtifactSnapshot {
    fn derive_transform_persistence(
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

    pub fn from_runtime_transform_state(
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
        stretch_engine: &RuntimeStretchEngineSnapshot,
        marker_analysis: &RuntimeMarkerAnalysisSnapshot,
        media_pipeline: &RuntimeMediaPipelineSnapshot,
    ) -> RuntimeTransformArtifactSnapshot {
        let media_assets = media_pipeline
            .assets
            .iter()
            .map(|asset| (asset.asset_id.as_str(), asset))
            .collect::<BTreeMap<_, _>>();
        let stretch_clips = stretch_engine
            .clips
            .iter()
            .map(|clip| (clip.clip_id.as_str(), clip))
            .collect::<BTreeMap<_, _>>();
        let marker_clips = marker_analysis
            .clips
            .iter()
            .map(|clip| (clip.clip_id.as_str(), clip))
            .collect::<BTreeMap<_, _>>();

        let clips = clip_processing
            .clips
            .iter()
            .map(|clip| {
                let media_asset = clip
                    .media_asset_id
                    .as_deref()
                    .and_then(|asset_id| media_assets.get(asset_id).copied());
                let stretch = stretch_clips.get(clip.clip_id.as_str()).copied();
                let marker = marker_clips.get(clip.clip_id.as_str()).copied();
                let cached_media_ready = media_asset.is_some_and(|asset| {
                    asset.state == Some(RuntimeMediaAssetState::Ready)
                        && asset.cache_path.as_deref().is_some()
                });
                let stretch_engine_class = stretch
                    .map(|snapshot| snapshot.engine_class)
                    .unwrap_or(RuntimeStretchEngineClass::Disabled);
                let stretch_readiness = stretch
                    .map(|snapshot| snapshot.readiness)
                    .unwrap_or(RuntimeStretchReadiness::Disabled);
                let marker_analysis_readiness = marker
                    .map(|snapshot| snapshot.readiness)
                    .unwrap_or(RuntimeMarkerAnalysisReadiness::Empty);
                let artifact_identity = clip
                    .media_asset_id
                    .as_ref()
                    .map(|asset_id| {
                        format!(
                            "artifact:{}:{}:{:?}:{:?}",
                            asset_id, clip.clip_id, stretch_engine_class, clip.warp_mode
                        )
                    })
                    .unwrap_or_else(|| format!("artifact:unsupported:{}", clip.clip_id));

                let (readiness, invalidation_state) =
                    match (clip.media_asset_id.as_deref(), media_asset) {
                        (None, _) => (
                            RuntimeTransformArtifactReadiness::Unsupported,
                            RuntimeTransformArtifactInvalidationState::None,
                        ),
                        (_, None) => (
                            RuntimeTransformArtifactReadiness::PendingMedia,
                            RuntimeTransformArtifactInvalidationState::None,
                        ),
                        (_, Some(asset)) => match asset.state {
                            Some(RuntimeMediaAssetState::Ingesting)
                            | Some(RuntimeMediaAssetState::Conforming)
                            | Some(RuntimeMediaAssetState::Rebuilding) => (
                                RuntimeTransformArtifactReadiness::PendingMedia,
                                RuntimeTransformArtifactInvalidationState::None,
                            ),
                            Some(RuntimeMediaAssetState::Invalid) => (
                                RuntimeTransformArtifactReadiness::Invalidated,
                                RuntimeTransformArtifactInvalidationState::MediaInvalidated,
                            ),
                            Some(RuntimeMediaAssetState::Ready) => {
                                if marker.is_some_and(|snapshot| {
                                    snapshot.readiness
                                        == RuntimeMarkerAnalysisReadiness::Invalidated
                                }) {
                                    (
                                        RuntimeTransformArtifactReadiness::Invalidated,
                                        RuntimeTransformArtifactInvalidationState::AnalysisInvalidated,
                                    )
                                } else if matches!(
                                    stretch_readiness,
                                    RuntimeStretchReadiness::Degraded
                                ) {
                                    (
                                        RuntimeTransformArtifactReadiness::Degraded,
                                        RuntimeTransformArtifactInvalidationState::StretchInvalidated,
                                    )
                                } else if matches!(
                                    stretch_readiness,
                                    RuntimeStretchReadiness::PendingMedia
                                        | RuntimeStretchReadiness::PendingWarp
                                ) || matches!(
                                    marker_analysis_readiness,
                                    RuntimeMarkerAnalysisReadiness::PendingMedia
                                ) {
                                    (
                                        RuntimeTransformArtifactReadiness::PendingMedia,
                                        RuntimeTransformArtifactInvalidationState::None,
                                    )
                                } else if matches!(
                                    marker_analysis_readiness,
                                    RuntimeMarkerAnalysisReadiness::Degraded
                                ) {
                                    (
                                        RuntimeTransformArtifactReadiness::Degraded,
                                        RuntimeTransformArtifactInvalidationState::AnalysisInvalidated,
                                    )
                                } else if matches!(
                                    marker_analysis_readiness,
                                    RuntimeMarkerAnalysisReadiness::Unsupported
                                ) {
                                    (
                                        RuntimeTransformArtifactReadiness::Unsupported,
                                        RuntimeTransformArtifactInvalidationState::None,
                                    )
                                } else {
                                    (
                                        RuntimeTransformArtifactReadiness::Ready,
                                        RuntimeTransformArtifactInvalidationState::None,
                                    )
                                }
                            }
                            None => (
                                RuntimeTransformArtifactReadiness::PendingMedia,
                                RuntimeTransformArtifactInvalidationState::None,
                            ),
                        },
                    };

                let reuse_state = match readiness {
                    RuntimeTransformArtifactReadiness::Ready if cached_media_ready => {
                        RuntimeTransformArtifactReuseState::Reusable
                    }
                    RuntimeTransformArtifactReadiness::Ready => {
                        RuntimeTransformArtifactReuseState::RequiresRender
                    }
                    RuntimeTransformArtifactReadiness::Degraded
                    | RuntimeTransformArtifactReadiness::Invalidated => {
                        RuntimeTransformArtifactReuseState::Guarded
                    }
                    RuntimeTransformArtifactReadiness::Empty
                    | RuntimeTransformArtifactReadiness::PendingMedia
                    | RuntimeTransformArtifactReadiness::Unsupported => {
                        RuntimeTransformArtifactReuseState::Unavailable
                    }
                };

                RuntimeTransformArtifactClipSnapshot {
                    clip_id: clip.clip_id.clone(),
                    media_asset_id: clip.media_asset_id.clone(),
                    artifact_identity,
                    readiness,
                    invalidation_state,
                    reuse_state,
                    cached_media_ready,
                    stretch_engine_class,
                    stretch_readiness,
                    marker_analysis_readiness,
                    summary: format!(
                        "clip={} artifact={} readiness={:?} invalidation={:?} reuse={:?} cached_media={} stretch={:?}/{:?} marker={:?}",
                        clip.clip_id,
                        clip.media_asset_id.as_deref().unwrap_or("unsupported"),
                        readiness,
                        invalidation_state,
                        reuse_state,
                        cached_media_ready,
                        stretch_engine_class,
                        stretch_readiness,
                        marker_analysis_readiness,
                    ),
                }
            })
            .collect::<Vec<_>>();

        let ready_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeTransformArtifactReadiness::Ready)
            .count();
        let pending_media_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeTransformArtifactReadiness::PendingMedia)
            .count();
        let degraded_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeTransformArtifactReadiness::Degraded)
            .count();
        let invalidated_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeTransformArtifactReadiness::Invalidated)
            .count();
        let unsupported_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeTransformArtifactReadiness::Unsupported)
            .count();
        let cached_media_ready_clip_count =
            clips.iter().filter(|clip| clip.cached_media_ready).count();
        let reusable_clip_count = clips
            .iter()
            .filter(|clip| clip.reuse_state == RuntimeTransformArtifactReuseState::Reusable)
            .count();
        let requires_render_clip_count = clips
            .iter()
            .filter(|clip| clip.reuse_state == RuntimeTransformArtifactReuseState::RequiresRender)
            .count();
        let guarded_reuse_clip_count = clips
            .iter()
            .filter(|clip| clip.reuse_state == RuntimeTransformArtifactReuseState::Guarded)
            .count();
        let transform_persistence = Self::derive_transform_persistence(
            media_pipeline,
            reusable_clip_count,
            requires_render_clip_count,
            guarded_reuse_clip_count,
            invalidated_clip_count,
            unsupported_clip_count,
        );
        let persistence_posture = transform_persistence.persistence_posture;
        let retention_outcome = transform_persistence.retention_outcome;
        let cache_placement_outcome = transform_persistence.cache_placement_outcome;

        RuntimeTransformArtifactSnapshot {
            clip_count: clips.len(),
            ready_clip_count,
            pending_media_clip_count,
            degraded_clip_count,
            invalidated_clip_count,
            unsupported_clip_count,
            cached_media_ready_clip_count,
            reusable_clip_count,
            requires_render_clip_count,
            guarded_reuse_clip_count,
            transform_persistence,
            clips,
            summary: format!(
                "transform_artifacts clips={} ready={} pending_media={} degraded={} invalidated={} unsupported={} cached_media_ready={} reusable={} requires_render={} guarded_reuse={} persistence={:?} retention_outcome={:?} cache_outcome={:?}",
                clip_processing.clip_count,
                ready_clip_count,
                pending_media_clip_count,
                degraded_clip_count,
                invalidated_clip_count,
                unsupported_clip_count,
                cached_media_ready_clip_count,
                reusable_clip_count,
                requires_render_clip_count,
                guarded_reuse_clip_count,
                persistence_posture,
                retention_outcome,
                cache_placement_outcome,
            ),
        }
    }
}
