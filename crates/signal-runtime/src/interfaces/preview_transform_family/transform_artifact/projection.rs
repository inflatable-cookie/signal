use super::*;
#[path = "projection/clip_snapshot.rs"]
mod clip_snapshot;
#[path = "projection/persistence.rs"]
mod persistence;

use clip_snapshot::build_transform_artifact_clip_snapshot;

impl RuntimeTransformArtifactSnapshot {
    /// Builds a transform artifact snapshot from the current pipeline, stretch engine, marker analysis, and media pipeline state.
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
                build_transform_artifact_clip_snapshot(
                    clip,
                    &media_assets,
                    &stretch_clips,
                    &marker_clips,
                )
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
        let transform_persistence = persistence::derive_transform_persistence(
            media_pipeline,
            reusable_clip_count,
            requires_render_clip_count,
            guarded_reuse_clip_count,
            invalidated_clip_count,
            unsupported_clip_count,
        );
        let _persistence_posture = transform_persistence.persistence_posture;
        let _retention_outcome = transform_persistence.retention_outcome;
        let _cache_placement_outcome = transform_persistence.cache_placement_outcome;

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
        }
    }
}
