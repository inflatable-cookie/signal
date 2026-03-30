use super::*;
#[path = "projection/clip_snapshot.rs"]
mod clip_snapshot;

use clip_snapshot::build_preview_transform_clip_snapshot;

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
                build_preview_transform_clip_snapshot(
                    clip,
                    media_service,
                    &stretch_clips,
                    &marker_clips,
                    &artifact_clips,
                )
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
