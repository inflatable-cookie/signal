use super::*;

fn json_runtime_marker_analysis_clip_snapshot(
    snapshot: &RuntimeMarkerAnalysisClipSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"readiness\":\"{:?}\",",
            "\"invalidation_state\":\"{:?}\",",
            "\"warp_marker_count\":{},",
            "\"transient_anchor_count\":{},",
            "\"tempo_assist_posture\":\"{:?}\",",
            "\"tempo_assist_hint_bpm\":{},",
            "\"tempo_assist_hint_source\":\"{:?}\",",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        snapshot.readiness,
        snapshot.invalidation_state,
        snapshot.warp_marker_count,
        snapshot.transient_anchor_count,
        snapshot.tempo_assist_posture,
        json_option_f64(snapshot.tempo_assist_hint_bpm),
        snapshot.tempo_assist_hint_source,
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_marker_analysis_clip_snapshot_vec(
    snapshots: &[RuntimeMarkerAnalysisClipSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_marker_analysis_clip_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_marker_analysis_snapshot(
    snapshot: &RuntimeMarkerAnalysisSnapshot,
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
            "\"tempo_assist_ready_clip_count\":{},",
            "\"warp_marker_count\":{},",
            "\"transient_anchor_count\":{},",
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
        snapshot.tempo_assist_ready_clip_count,
        snapshot.warp_marker_count,
        snapshot.transient_anchor_count,
        json_runtime_marker_analysis_clip_snapshot_vec(&snapshot.clips),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
