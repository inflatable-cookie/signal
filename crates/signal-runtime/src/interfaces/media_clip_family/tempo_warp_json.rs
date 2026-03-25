use super::*;

fn json_runtime_tempo_map_segment_snapshot(snapshot: &RuntimeTempoMapSegmentSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"segment_id\":{},",
            "\"start_samples\":{},",
            "\"end_samples\":{},",
            "\"start_tempo_bpm\":{},",
            "\"end_tempo_bpm\":{},",
            "\"interpolation\":\"{:?}\",",
            "\"covers_timeline_position\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.segment_id.as_str())),
        snapshot.start_samples,
        json_option_i64(snapshot.end_samples),
        snapshot.start_tempo_bpm,
        json_option_f64(snapshot.end_tempo_bpm),
        snapshot.interpolation,
        snapshot.covers_timeline_position,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_tempo_map_segment_snapshot_vec(
    snapshots: &[RuntimeTempoMapSegmentSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_tempo_map_segment_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_tempo_map_snapshot(snapshot: &RuntimeTempoMapSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"segment_count\":{},",
            "\"active_segment_id\":{},",
            "\"active_segment_index\":{},",
            "\"next_segment_start_samples\":{},",
            "\"resolved_tempo_bpm\":{},",
            "\"tempo_source\":\"{:?}\",",
            "\"timeline_position_samples\":{},",
            "\"segments\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.segment_count,
        json_option_string(snapshot.active_segment_id.as_deref()),
        json_option_usize(snapshot.active_segment_index),
        json_option_i64(snapshot.next_segment_start_samples),
        snapshot.resolved_tempo_bpm,
        snapshot.tempo_source,
        json_option_i64(snapshot.timeline_position_samples),
        json_runtime_tempo_map_segment_snapshot_vec(&snapshot.segments),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_warp_clip_snapshot(snapshot: &RuntimeWarpClipSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"mode\":\"{:?}\",",
            "\"source_tempo_bpm\":{},",
            "\"project_tempo_bpm\":{},",
            "\"project_tempo_source\":\"{:?}\",",
            "\"project_tempo_segment_id\":{},",
            "\"realized_ratio\":{},",
            "\"anchor_timeline_samples\":{},",
            "\"start_samples\":{},",
            "\"duration_samples\":{},",
            "\"readiness\":\"{:?}\",",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        snapshot.mode,
        json_option_f64(snapshot.source_tempo_bpm),
        snapshot.project_tempo_bpm,
        snapshot.project_tempo_source,
        json_option_string(snapshot.project_tempo_segment_id.as_deref()),
        snapshot.realized_ratio,
        snapshot.anchor_timeline_samples,
        snapshot.start_samples,
        snapshot.duration_samples,
        snapshot.readiness,
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_warp_clip_snapshot_vec(snapshots: &[RuntimeWarpClipSnapshot]) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_warp_clip_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_warp_pipeline_snapshot(
    snapshot: &RuntimeWarpPipelineSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_count\":{},",
            "\"ready_clip_count\":{},",
            "\"degraded_clip_count\":{},",
            "\"bypassed_clip_count\":{},",
            "\"active_warp_count\":{},",
            "\"resolved_project_tempo_bpm\":{},",
            "\"resolved_project_tempo_source\":\"{:?}\",",
            "\"resolved_project_tempo_segment_id\":{},",
            "\"clips\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.degraded_clip_count,
        snapshot.bypassed_clip_count,
        snapshot.active_warp_count,
        snapshot.resolved_project_tempo_bpm,
        snapshot.resolved_project_tempo_source,
        json_option_string(snapshot.resolved_project_tempo_segment_id.as_deref()),
        json_runtime_warp_clip_snapshot_vec(&snapshot.clips),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
