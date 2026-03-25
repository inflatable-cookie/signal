use super::*;

pub(crate) fn format_runtime_transform_artifact_snapshot_compact(
    snapshot: &RuntimeTransformArtifactSnapshot,
) -> String {
    format!(
        " transform_artifacts={}/{}/{}/{}/{} transform_artifact_reuse={}/{}/{} transform_artifact_cached_media_ready={}",
        snapshot.ready_clip_count,
        snapshot.clip_count,
        snapshot.pending_media_clip_count,
        snapshot.degraded_clip_count,
        snapshot.invalidated_clip_count + snapshot.unsupported_clip_count,
        snapshot.reusable_clip_count,
        snapshot.requires_render_clip_count,
        snapshot.guarded_reuse_clip_count,
        snapshot.cached_media_ready_clip_count,
    )
}

pub(crate) fn format_runtime_transform_artifact_snapshot_multiline(
    snapshot: &RuntimeTransformArtifactSnapshot,
) -> String {
    let clip_lines = snapshot
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            format!(
                "\ntransform_artifact_clip_{}={}/artifact={}/readiness={:?}/invalidation={:?}/reuse={:?}/cached_media_ready={}/stretch={:?}/{:?}/analysis={:?}",
                index,
                clip.clip_id,
                clip.artifact_identity,
                clip.readiness,
                clip.invalidation_state,
                clip.reuse_state,
                clip.cached_media_ready,
                clip.stretch_engine_class,
                clip.stretch_readiness,
                clip.marker_analysis_readiness,
            )
        })
        .collect::<String>();
    format!(
        "\ntransform_artifact_clip_count={}\ntransform_artifact_ready_clip_count={}\ntransform_artifact_pending_media_clip_count={}\ntransform_artifact_degraded_clip_count={}\ntransform_artifact_invalidated_clip_count={}\ntransform_artifact_unsupported_clip_count={}\ntransform_artifact_cached_media_ready_clip_count={}\ntransform_artifact_reusable_clip_count={}\ntransform_artifact_requires_render_clip_count={}\ntransform_artifact_guarded_reuse_clip_count={}{}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.degraded_clip_count,
        snapshot.invalidated_clip_count,
        snapshot.unsupported_clip_count,
        snapshot.cached_media_ready_clip_count,
        snapshot.reusable_clip_count,
        snapshot.requires_render_clip_count,
        snapshot.guarded_reuse_clip_count,
        clip_lines,
    )
}
