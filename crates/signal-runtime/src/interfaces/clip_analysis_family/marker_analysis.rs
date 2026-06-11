use super::*;

fn runtime_estimated_analysis_event_count(
    density: f32,
    duration_samples: u32,
    sample_rate_hz: u32,
) -> usize {
    if !density.is_finite() || density <= 0.0 || duration_samples == 0 || sample_rate_hz == 0 {
        return 0;
    }
    let duration_seconds = duration_samples as f64 / sample_rate_hz as f64;
    (f64::from(density) * duration_seconds).round().max(1.0) as usize
}

impl RuntimeMarkerAnalysisSnapshot {
    /// Derives a marker analysis snapshot from the clip processing pipeline, stretch engine, warp pipeline, and media library snapshots.
    pub fn from_clip_processing_and_media_library(
        clip_processing: &RuntimeClipProcessingPipelineSnapshot,
        stretch_engine: &RuntimeStretchEngineSnapshot,
        warp_pipeline: &RuntimeWarpPipelineSnapshot,
        media_library: &RuntimeMediaLibraryServiceSnapshot,
        sample_rate_hz: u32,
    ) -> RuntimeMarkerAnalysisSnapshot {
        let media_descriptors = media_library
            .descriptors
            .iter()
            .map(|descriptor| (descriptor.asset_id.as_str(), descriptor))
            .collect::<BTreeMap<_, _>>();
        let stretch_clips = stretch_engine
            .clips
            .iter()
            .map(|clip| (clip.clip_id.as_str(), clip))
            .collect::<BTreeMap<_, _>>();
        let warp_clips = warp_pipeline
            .clips
            .iter()
            .map(|clip| (clip.clip_id.as_str(), clip))
            .collect::<BTreeMap<_, _>>();
        let clips = clip_processing
            .clips
            .iter()
            .map(|clip| {
                let descriptor = clip
                    .media_asset_id
                    .as_deref()
                    .and_then(|asset_id| media_descriptors.get(asset_id).copied());
                let stretch = stretch_clips.get(clip.clip_id.as_str()).copied();
                let warp = warp_clips.get(clip.clip_id.as_str()).copied();

                let (
                    readiness,
                    invalidation_state,
                    warp_marker_count,
                    transient_anchor_count,
                    last_error,
                ) = match (clip.media_asset_id.as_deref(), descriptor) {
                    (None, _) => (
                        RuntimeMarkerAnalysisReadiness::Unsupported,
                        RuntimeMarkerAnalysisInvalidationState::None,
                        0,
                        0,
                        Some("marker analysis requires media-backed clip".to_string()),
                    ),
                    (_, None) => (
                        RuntimeMarkerAnalysisReadiness::PendingMedia,
                        RuntimeMarkerAnalysisInvalidationState::None,
                        0,
                        0,
                        Some("marker analysis is waiting for media descriptors".to_string()),
                    ),
                    (_, Some(descriptor)) => match descriptor.metadata_state {
                        RuntimeMediaAnalysisDescriptorState::Missing
                        | RuntimeMediaAnalysisDescriptorState::Pending => (
                            RuntimeMarkerAnalysisReadiness::PendingMedia,
                            RuntimeMarkerAnalysisInvalidationState::None,
                            0,
                            0,
                            descriptor.last_error.clone(),
                        ),
                        RuntimeMediaAnalysisDescriptorState::Invalidated => (
                            RuntimeMarkerAnalysisReadiness::Invalidated,
                            RuntimeMarkerAnalysisInvalidationState::MediaInvalidated,
                            0,
                            0,
                            descriptor
                                .last_error
                                .clone()
                                .or_else(|| Some("marker analysis invalidated with media".into())),
                        ),
                        RuntimeMediaAnalysisDescriptorState::Unavailable => (
                            RuntimeMarkerAnalysisReadiness::Unsupported,
                            RuntimeMarkerAnalysisInvalidationState::None,
                            0,
                            0,
                            descriptor.last_error.clone().or_else(|| {
                                Some("marker analysis unavailable for this asset".into())
                            }),
                        ),
                        RuntimeMediaAnalysisDescriptorState::Ready => {
                            let degraded_by_stretch = matches!(
                                clip.readiness,
                                RuntimeClipProcessingReadiness::Invalid
                            ) || stretch.is_some_and(|stretch| {
                                matches!(stretch.readiness, RuntimeStretchReadiness::Degraded)
                            });
                            if degraded_by_stretch {
                                (
                                    RuntimeMarkerAnalysisReadiness::Degraded,
                                    RuntimeMarkerAnalysisInvalidationState::StretchInvalidated,
                                    0,
                                    0,
                                    clip.last_error.clone().or_else(|| {
                                        Some("marker analysis is stale against stretch state".into())
                                    }),
                                )
                            } else if let Some(character) = descriptor.character.as_ref() {
                                (
                                    RuntimeMarkerAnalysisReadiness::Ready,
                                    RuntimeMarkerAnalysisInvalidationState::None,
                                    runtime_estimated_analysis_event_count(
                                        character.onset_density,
                                        clip.duration_samples,
                                        sample_rate_hz,
                                    ),
                                    runtime_estimated_analysis_event_count(
                                        character.transient_density,
                                        clip.duration_samples,
                                        sample_rate_hz,
                                    ),
                                    clip.last_error.clone(),
                                )
                            } else {
                                (
                                    RuntimeMarkerAnalysisReadiness::Degraded,
                                    RuntimeMarkerAnalysisInvalidationState::None,
                                    0,
                                    0,
                                    descriptor.last_error.clone().or_else(|| {
                                        Some("marker analysis missing character descriptors".into())
                                    }),
                                )
                            }
                        }
                    },
                };

                let (tempo_assist_posture, tempo_assist_hint_bpm, tempo_assist_hint_source) =
                    if readiness == RuntimeMarkerAnalysisReadiness::Ready {
                        if let Some(source_tempo_bpm) = warp.and_then(|warp| warp.source_tempo_bpm) {
                            (
                                RuntimeTempoAssistPosture::Ready,
                                Some(source_tempo_bpm),
                                RuntimeTempoAssistHintSource::SourceTempo,
                            )
                        } else if let Some(warp) = warp {
                            (
                                RuntimeTempoAssistPosture::Ready,
                                Some(warp.project_tempo_bpm),
                                RuntimeTempoAssistHintSource::ProjectTempo,
                            )
                        } else {
                            (
                                RuntimeTempoAssistPosture::Guarded,
                                None,
                                RuntimeTempoAssistHintSource::None,
                            )
                        }
                    } else {
                        (
                            RuntimeTempoAssistPosture::Unavailable,
                            None,
                            RuntimeTempoAssistHintSource::None,
                        )
                    };

                RuntimeMarkerAnalysisClipSnapshot {
                    clip_id: clip.clip_id.clone(),
                    media_asset_id: clip.media_asset_id.clone(),
                    readiness,
                    invalidation_state,
                    warp_marker_count,
                    transient_anchor_count,
                    tempo_assist_posture,
                    tempo_assist_hint_bpm,
                    tempo_assist_hint_source,
                    last_error,
                }
            })
            .collect::<Vec<_>>();

        let ready_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeMarkerAnalysisReadiness::Ready)
            .count();
        let pending_media_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeMarkerAnalysisReadiness::PendingMedia)
            .count();
        let degraded_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeMarkerAnalysisReadiness::Degraded)
            .count();
        let invalidated_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeMarkerAnalysisReadiness::Invalidated)
            .count();
        let unsupported_clip_count = clips
            .iter()
            .filter(|clip| clip.readiness == RuntimeMarkerAnalysisReadiness::Unsupported)
            .count();
        let tempo_assist_ready_clip_count = clips
            .iter()
            .filter(|clip| clip.tempo_assist_posture == RuntimeTempoAssistPosture::Ready)
            .count();
        let warp_marker_count = clips.iter().map(|clip| clip.warp_marker_count).sum();
        let transient_anchor_count = clips.iter().map(|clip| clip.transient_anchor_count).sum();

        RuntimeMarkerAnalysisSnapshot {
            clip_count: clips.len(),
            ready_clip_count,
            pending_media_clip_count,
            degraded_clip_count,
            invalidated_clip_count,
            unsupported_clip_count,
            tempo_assist_ready_clip_count,
            warp_marker_count,
            transient_anchor_count,
            clips,
        }
    }
}
