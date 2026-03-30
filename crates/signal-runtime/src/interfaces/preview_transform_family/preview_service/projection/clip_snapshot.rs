use super::*;

pub(super) fn build_preview_transform_clip_snapshot(
    clip: &RuntimeClipProcessingSnapshot,
    media_service: &RuntimeMediaServiceSnapshot,
    stretch_clips: &BTreeMap<&str, &RuntimeStretchClipSnapshot>,
    marker_clips: &BTreeMap<&str, &RuntimeMarkerAnalysisClipSnapshot>,
    artifact_clips: &BTreeMap<&str, &RuntimeTransformArtifactClipSnapshot>,
) -> RuntimePreviewTransformClipSnapshot {
    let stretch = stretch_clips.get(clip.clip_id.as_str()).copied();
    let marker = marker_clips.get(clip.clip_id.as_str()).copied();
    let artifact = artifact_clips.get(clip.clip_id.as_str()).copied();
    let audition_active =
        clip.media_asset_id.as_deref() == media_service.previewing_asset_id.as_deref();

    let media_preview_possible = matches!(
        media_service.preview_state,
        RuntimeMediaPreviewState::Ready | RuntimeMediaPreviewState::Previewing
    ) || media_service.previewable_asset_count > 0;

    let (service_class, readiness, degraded_state, fallback_kind) =
        match (clip.media_asset_id.as_deref(), stretch, marker, artifact) {
            (None, _, _, _) => (
                RuntimePreviewTransformServiceClass::Unavailable,
                RuntimePreviewTransformReadiness::Unsupported,
                RuntimePreviewTransformDegradedState::UnsupportedScope,
                RuntimePreviewTransformFallbackKind::OfflineOnly,
            ),
            (_, _, _, Some(artifact))
                if artifact.readiness == RuntimeTransformArtifactReadiness::Invalidated =>
            {
                (
                    RuntimePreviewTransformServiceClass::Fallback,
                    RuntimePreviewTransformReadiness::Invalidated,
                    RuntimePreviewTransformDegradedState::InvalidatedInputs,
                    fallback_kind_for_media_state(media_preview_possible),
                )
            }
            (_, _, Some(marker), _)
                if marker.readiness == RuntimeMarkerAnalysisReadiness::Invalidated =>
            {
                (
                    RuntimePreviewTransformServiceClass::Fallback,
                    RuntimePreviewTransformReadiness::Invalidated,
                    RuntimePreviewTransformDegradedState::InvalidatedInputs,
                    fallback_kind_for_media_state(media_preview_possible),
                )
            }
            (_, Some(stretch), Some(marker), Some(artifact))
                if artifact.reuse_state == RuntimeTransformArtifactReuseState::Reusable
                    && stretch.readiness == RuntimeStretchReadiness::Ready
                    && marker.readiness == RuntimeMarkerAnalysisReadiness::Ready =>
            {
                (
                    RuntimePreviewTransformServiceClass::ArtifactBacked,
                    RuntimePreviewTransformReadiness::Ready,
                    RuntimePreviewTransformDegradedState::None,
                    RuntimePreviewTransformFallbackKind::None,
                )
            }
            (_, Some(stretch), _, _) if stretch.readiness == RuntimeStretchReadiness::Ready => (
                RuntimePreviewTransformServiceClass::StretchAligned,
                RuntimePreviewTransformReadiness::Ready,
                RuntimePreviewTransformDegradedState::None,
                RuntimePreviewTransformFallbackKind::None,
            ),
            (_, Some(stretch), Some(marker), Some(artifact))
                if matches!(
                    stretch.readiness,
                    RuntimeStretchReadiness::PendingMedia | RuntimeStretchReadiness::PendingWarp
                ) || marker.readiness == RuntimeMarkerAnalysisReadiness::PendingMedia
                    || artifact.readiness == RuntimeTransformArtifactReadiness::PendingMedia =>
            {
                (
                    RuntimePreviewTransformServiceClass::StretchAligned,
                    RuntimePreviewTransformReadiness::Pending,
                    RuntimePreviewTransformDegradedState::PendingInputs,
                    fallback_kind_for_media_state(media_preview_possible),
                )
            }
            (_, Some(stretch), Some(marker), Some(artifact))
                if stretch.readiness == RuntimeStretchReadiness::Degraded
                    || marker.readiness == RuntimeMarkerAnalysisReadiness::Degraded
                    || artifact.readiness == RuntimeTransformArtifactReadiness::Degraded =>
            {
                (
                    RuntimePreviewTransformServiceClass::Fallback,
                    RuntimePreviewTransformReadiness::Degraded,
                    RuntimePreviewTransformDegradedState::FallbackOnly,
                    fallback_kind_for_media_state(media_preview_possible),
                )
            }
            (_, _, _, Some(artifact))
                if artifact.readiness == RuntimeTransformArtifactReadiness::Unsupported =>
            {
                (
                    RuntimePreviewTransformServiceClass::Unavailable,
                    RuntimePreviewTransformReadiness::Unsupported,
                    RuntimePreviewTransformDegradedState::UnsupportedScope,
                    RuntimePreviewTransformFallbackKind::OfflineOnly,
                )
            }
            (_, _, _, _)
                if media_preview_possible
                    || media_service.preview_state == RuntimeMediaPreviewState::Invalidated =>
            {
                (
                    RuntimePreviewTransformServiceClass::Fallback,
                    RuntimePreviewTransformReadiness::Pending,
                    RuntimePreviewTransformDegradedState::PendingInputs,
                    RuntimePreviewTransformFallbackKind::MediaOnly,
                )
            }
            _ => (
                RuntimePreviewTransformServiceClass::Unavailable,
                RuntimePreviewTransformReadiness::Empty,
                RuntimePreviewTransformDegradedState::PendingInputs,
                RuntimePreviewTransformFallbackKind::OfflineOnly,
            ),
        };

    let scrub_supported = matches!(
        readiness,
        RuntimePreviewTransformReadiness::Ready
            | RuntimePreviewTransformReadiness::Degraded
            | RuntimePreviewTransformReadiness::Invalidated
    );
    let artifact_reuse_state = artifact
        .map(|artifact| artifact.reuse_state)
        .unwrap_or(RuntimeTransformArtifactReuseState::Unavailable);

    RuntimePreviewTransformClipSnapshot {
        clip_id: clip.clip_id.clone(),
        media_asset_id: clip.media_asset_id.clone(),
        service_class,
        readiness,
        degraded_state,
        fallback_kind,
        artifact_reuse_state,
        audition_active,
        scrub_supported,
        summary: format!(
            "clip={} class={:?} readiness={:?} degraded={:?} fallback={:?} artifact_reuse={:?} audition_active={} scrub_supported={}",
            clip.clip_id,
            service_class,
            readiness,
            degraded_state,
            fallback_kind,
            artifact_reuse_state,
            audition_active,
            scrub_supported,
        ),
    }
}

fn fallback_kind_for_media_state(
    media_preview_possible: bool,
) -> RuntimePreviewTransformFallbackKind {
    if media_preview_possible {
        RuntimePreviewTransformFallbackKind::MediaOnly
    } else {
        RuntimePreviewTransformFallbackKind::OfflineOnly
    }
}
