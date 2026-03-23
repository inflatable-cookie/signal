use super::*;
pub(super) fn json_runtime_transform_artifact_snapshot(
    snapshot: &RuntimeTransformArtifactSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_count\":{},",
            "\"ready_clip_count\":{},",
            "\"pending_media_clip_count\":{},",
            "\"degraded_clip_count\":{},",
            "\"invalidated_clip_count\":{},",
            "\"unsupported_clip_count\":{},",
            "\"cached_media_ready_clip_count\":{},",
            "\"reusable_clip_count\":{},",
            "\"requires_render_clip_count\":{},",
            "\"guarded_reuse_clip_count\":{},",
            "\"transform_persistence\":{},",
            "\"clips\":{},",
            "\"summary\":{}",
            "}}"
        ),
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
        json_runtime_transform_persistence_summary(&snapshot.transform_persistence),
        format!(
            "[{}]",
            snapshot
                .clips
                .iter()
                .map(json_runtime_transform_artifact_clip_snapshot)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_transform_artifact_clip_snapshot(
    snapshot: &RuntimeTransformArtifactClipSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"artifact_identity\":{},",
            "\"readiness\":{},",
            "\"invalidation_state\":{},",
            "\"reuse_state\":{},",
            "\"cached_media_ready\":{},",
            "\"stretch_engine_class\":{},",
            "\"stretch_readiness\":{},",
            "\"marker_analysis_readiness\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        json_option_string(Some(snapshot.artifact_identity.as_str())),
        json_string(&format!("{:?}", snapshot.readiness)),
        json_string(&format!("{:?}", snapshot.invalidation_state)),
        json_string(&format!("{:?}", snapshot.reuse_state)),
        snapshot.cached_media_ready,
        json_string(&format!("{:?}", snapshot.stretch_engine_class)),
        json_string(&format!("{:?}", snapshot.stretch_readiness)),
        json_string(&format!("{:?}", snapshot.marker_analysis_readiness)),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_transform_persistence_summary(
    summary: &RuntimeTransformPersistenceSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"persistence_posture\":{},",
            "\"retention_policy_class\":{},",
            "\"retention_authority\":{},",
            "\"retention_outcome\":{},",
            "\"cache_placement_posture\":{},",
            "\"cache_placement_authority\":{},",
            "\"cache_placement_outcome\":{},",
            "\"cache_root_path\":{},",
            "\"persistent_clip_count\":{},",
            "\"guarded_persistence_clip_count\":{},",
            "\"invalidated_persistence_clip_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", summary.persistence_posture)),
        json_string(&format!("{:?}", summary.retention_policy_class)),
        json_string(&format!("{:?}", summary.retention_authority)),
        json_string(&format!("{:?}", summary.retention_outcome)),
        json_string(&format!("{:?}", summary.cache_placement_posture)),
        json_string(&format!("{:?}", summary.cache_placement_authority)),
        json_string(&format!("{:?}", summary.cache_placement_outcome)),
        json_option_string(Some(summary.cache_root_path.as_str())),
        summary.persistent_clip_count,
        summary.guarded_persistence_clip_count,
        summary.invalidated_persistence_clip_count,
        json_option_string(Some(summary.summary.as_str())),
    )
}
