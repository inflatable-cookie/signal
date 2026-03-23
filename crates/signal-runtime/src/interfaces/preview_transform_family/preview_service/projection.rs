use super::*;

impl RuntimePreviewTransformServiceSnapshot {
    pub fn from_runtime_preview_state(
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
        media_service: &RuntimeMediaServiceSnapshot,
        stretch_engine: &RuntimeStretchEngineSnapshot,
        marker_analysis: &RuntimeMarkerAnalysisSnapshot,
        transform_artifact: &RuntimeTransformArtifactSnapshot,
    ) -> RuntimePreviewTransformServiceSnapshot {
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
        let artifact_clips = transform_artifact
            .clips
            .iter()
            .map(|clip| (clip.clip_id.as_str(), clip))
            .collect::<BTreeMap<_, _>>();

        let clips = clip_processing
            .clips
            .iter()
            .map(|clip| {
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
                            if artifact.readiness
                                == RuntimeTransformArtifactReadiness::Invalidated =>
                        {
                            (
                                RuntimePreviewTransformServiceClass::Fallback,
                                RuntimePreviewTransformReadiness::Invalidated,
                                RuntimePreviewTransformDegradedState::InvalidatedInputs,
                                if media_preview_possible {
                                    RuntimePreviewTransformFallbackKind::MediaOnly
                                } else {
                                    RuntimePreviewTransformFallbackKind::OfflineOnly
                                },
                            )
                        }
                        (_, _, Some(marker), _)
                            if marker.readiness
                                == RuntimeMarkerAnalysisReadiness::Invalidated =>
                        {
                            (
                                RuntimePreviewTransformServiceClass::Fallback,
                                RuntimePreviewTransformReadiness::Invalidated,
                                RuntimePreviewTransformDegradedState::InvalidatedInputs,
                                if media_preview_possible {
                                    RuntimePreviewTransformFallbackKind::MediaOnly
                                } else {
                                    RuntimePreviewTransformFallbackKind::OfflineOnly
                                },
                            )
                        }
                        (_, Some(stretch), Some(marker), Some(artifact))
                            if artifact.reuse_state
                                == RuntimeTransformArtifactReuseState::Reusable
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
                        (_, Some(stretch), _, _)
                            if stretch.readiness == RuntimeStretchReadiness::Ready =>
                        {
                            (
                                RuntimePreviewTransformServiceClass::StretchAligned,
                                RuntimePreviewTransformReadiness::Ready,
                                RuntimePreviewTransformDegradedState::None,
                                RuntimePreviewTransformFallbackKind::None,
                            )
                        }
                        (_, Some(stretch), Some(marker), Some(artifact))
                            if matches!(
                                stretch.readiness,
                                RuntimeStretchReadiness::PendingMedia
                                    | RuntimeStretchReadiness::PendingWarp
                            ) || marker.readiness
                                == RuntimeMarkerAnalysisReadiness::PendingMedia
                                || artifact.readiness
                                    == RuntimeTransformArtifactReadiness::PendingMedia =>
                        {
                            (
                                RuntimePreviewTransformServiceClass::StretchAligned,
                                RuntimePreviewTransformReadiness::Pending,
                                RuntimePreviewTransformDegradedState::PendingInputs,
                                if media_preview_possible {
                                    RuntimePreviewTransformFallbackKind::MediaOnly
                                } else {
                                    RuntimePreviewTransformFallbackKind::OfflineOnly
                                },
                            )
                        }
                        (_, Some(stretch), Some(marker), Some(artifact))
                            if stretch.readiness == RuntimeStretchReadiness::Degraded
                                || marker.readiness
                                    == RuntimeMarkerAnalysisReadiness::Degraded
                                || artifact.readiness
                                    == RuntimeTransformArtifactReadiness::Degraded =>
                        {
                            (
                                RuntimePreviewTransformServiceClass::Fallback,
                                RuntimePreviewTransformReadiness::Degraded,
                                RuntimePreviewTransformDegradedState::FallbackOnly,
                                if media_preview_possible {
                                    RuntimePreviewTransformFallbackKind::MediaOnly
                                } else {
                                    RuntimePreviewTransformFallbackKind::OfflineOnly
                                },
                            )
                        }
                        (_, _, _, Some(artifact))
                            if artifact.readiness
                                == RuntimeTransformArtifactReadiness::Unsupported =>
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
                                || media_service.preview_state
                                    == RuntimeMediaPreviewState::Invalidated =>
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
            })
            .collect::<Vec<_>>();

        let active_audition_clip_count = clips.iter().filter(|clip| clip.audition_active).count();
        let scrub_supported_clip_count = clips.iter().filter(|clip| clip.scrub_supported).count();
        let ready_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimePreviewTransformReadiness::Ready)
            .count();
        let pending_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimePreviewTransformReadiness::Pending)
            .count();
        let degraded_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimePreviewTransformReadiness::Degraded)
            .count();
        let invalidated_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimePreviewTransformReadiness::Invalidated)
            .count();
        let unsupported_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimePreviewTransformReadiness::Unsupported)
            .count();
        let stretch_aligned_clip_count = clips
            .iter()
            .filter(|clip| {
                clip.service_class == RuntimePreviewTransformServiceClass::StretchAligned
            })
            .count();
        let artifact_backed_clip_count = clips
            .iter()
            .filter(|clip| {
                clip.service_class == RuntimePreviewTransformServiceClass::ArtifactBacked
            })
            .count();
        let fallback_clip_count = clips
            .iter()
            .filter(|clip| clip.service_class == RuntimePreviewTransformServiceClass::Fallback)
            .count();
        let preview_device_policy =
            Self::derive_preview_device_policy(media_service, active_audition_clip_count);
        let routing_posture = preview_device_policy.routing_posture;
        let audition_sink_class = preview_device_policy.audition_sink_class;
        let low_latency_device_policy_class = preview_device_policy.low_latency_device_policy_class;
        let low_latency_device_policy_outcome =
            preview_device_policy.low_latency_device_policy_outcome;
        let preview_workflow = Self::derive_preview_workflow(
            media_service,
            active_audition_clip_count,
            ready_clip_count,
            pending_clip_count,
            fallback_clip_count,
            artifact_backed_clip_count,
            unsupported_clip_count,
        );
        let queue_posture = preview_workflow.queue_posture;
        let audition_posture = preview_workflow.audition_posture;
        let transform_scheduling_posture = preview_workflow.transform_scheduling_posture;

        RuntimePreviewTransformServiceSnapshot {
            clip_count: clips.len(),
            active_audition_clip_count,
            scrub_supported_clip_count,
            ready_clip_count,
            pending_clip_count,
            degraded_clip_count,
            invalidated_clip_count,
            unsupported_clip_count,
            stretch_aligned_clip_count,
            artifact_backed_clip_count,
            fallback_clip_count,
            preview_device_policy,
            preview_workflow,
            clips,
            summary: format!(
                "preview_transform clips={} active_audition={} scrub_supported={} ready={} pending={} degraded={} invalidated={} unsupported={} stretch_aligned={} artifact_backed={} fallback={} route={:?} sink={:?} policy={:?} outcome={:?} queue={:?} audition={:?} scheduling={:?}",
                clip_processing.clip_count,
                active_audition_clip_count,
                scrub_supported_clip_count,
                ready_clip_count,
                pending_clip_count,
                degraded_clip_count,
                invalidated_clip_count,
                unsupported_clip_count,
                stretch_aligned_clip_count,
                artifact_backed_clip_count,
                fallback_clip_count,
                routing_posture,
                audition_sink_class,
                low_latency_device_policy_class,
                low_latency_device_policy_outcome,
                queue_posture,
                audition_posture,
                transform_scheduling_posture,
            ),
        }
    }
}
