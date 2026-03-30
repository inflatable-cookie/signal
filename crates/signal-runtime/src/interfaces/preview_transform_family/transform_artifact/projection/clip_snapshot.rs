use super::super::*;

pub(super) fn build_transform_artifact_clip_snapshot(
    clip: &RuntimeClipProcessingSnapshot,
    media_assets: &BTreeMap<&str, &RuntimeMediaAssetSnapshot>,
    stretch_clips: &BTreeMap<&str, &RuntimeStretchClipSnapshot>,
    marker_clips: &BTreeMap<&str, &RuntimeMarkerAnalysisClipSnapshot>,
) -> RuntimeTransformArtifactClipSnapshot {
    let media_asset = clip
        .media_asset_id
        .as_deref()
        .and_then(|asset_id| media_assets.get(asset_id).copied());
    let stretch = stretch_clips.get(clip.clip_id.as_str()).copied();
    let marker = marker_clips.get(clip.clip_id.as_str()).copied();
    let cached_media_ready = media_asset.is_some_and(|asset| {
        asset.state == Some(RuntimeMediaAssetState::Ready) && asset.cache_path.as_deref().is_some()
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

    let (readiness, invalidation_state) = match (clip.media_asset_id.as_deref(), media_asset) {
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
                    snapshot.readiness == RuntimeMarkerAnalysisReadiness::Invalidated
                }) {
                    (
                        RuntimeTransformArtifactReadiness::Invalidated,
                        RuntimeTransformArtifactInvalidationState::AnalysisInvalidated,
                    )
                } else if matches!(stretch_readiness, RuntimeStretchReadiness::Degraded) {
                    (
                        RuntimeTransformArtifactReadiness::Degraded,
                        RuntimeTransformArtifactInvalidationState::StretchInvalidated,
                    )
                } else if matches!(
                    stretch_readiness,
                    RuntimeStretchReadiness::PendingMedia | RuntimeStretchReadiness::PendingWarp
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
}
