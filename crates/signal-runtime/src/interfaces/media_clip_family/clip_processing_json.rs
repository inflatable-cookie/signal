use super::*;

fn json_runtime_clip_processing_stage_vec(stages: &[RuntimeClipProcessingStage]) -> String {
    let joined = stages
        .iter()
        .map(|stage| json_escape_string(&format!("{stage:?}")))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

fn json_runtime_clip_processing_snapshot(snapshot: &RuntimeClipProcessingSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"warp_mode\":\"{:?}\",",
            "\"start_samples\":{},",
            "\"duration_samples\":{},",
            "\"fade_in\":{{\"duration_samples\":{},\"shape\":\"{:?}\"}},",
            "\"fade_out\":{{\"duration_samples\":{},\"shape\":\"{:?}\"}},",
            "\"fade_in_end_samples\":{},",
            "\"fade_out_start_samples\":{},",
            "\"clip_gain\":{{\"start_linear\":{},\"end_linear\":{},\"shape\":\"{:?}\"}},",
            "\"treatment_stages\":{},",
            "\"realized_warp_ratio\":{},",
            "\"project_tempo_source\":{},",
            "\"project_tempo_segment_id\":{},",
            "\"readiness\":\"{:?}\",",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        snapshot.warp_mode,
        snapshot.start_samples,
        snapshot.duration_samples,
        snapshot.fade_in.duration_samples,
        snapshot.fade_in.shape,
        snapshot.fade_out.duration_samples,
        snapshot.fade_out.shape,
        snapshot.fade_in_end_samples,
        snapshot.fade_out_start_samples,
        snapshot.clip_gain.start_linear,
        snapshot.clip_gain.end_linear,
        snapshot.clip_gain.shape,
        json_runtime_clip_processing_stage_vec(&snapshot.treatment_stages),
        json_option_f64(snapshot.realized_warp_ratio),
        json_option_string(
            snapshot
                .project_tempo_source
                .map(|value| format!("{value:?}"))
                .as_deref(),
        ),
        json_option_string(snapshot.project_tempo_segment_id.as_deref()),
        snapshot.readiness,
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_clip_processing_snapshot_vec(
    snapshots: &[RuntimeClipProcessingSnapshot],
) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_clip_processing_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_clip_processing_pipeline_snapshot(
    snapshot: &RuntimeClipProcessingPipelineSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_count\":{},",
            "\"ready_clip_count\":{},",
            "\"pending_media_clip_count\":{},",
            "\"pending_warp_clip_count\":{},",
            "\"invalid_clip_count\":{},",
            "\"faded_clip_count\":{},",
            "\"gain_shaped_clip_count\":{},",
            "\"warped_clip_count\":{},",
            "\"treatment_stage_count\":{},",
            "\"clips\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count,
        snapshot.invalid_clip_count,
        snapshot.faded_clip_count,
        snapshot.gain_shaped_clip_count,
        snapshot.warped_clip_count,
        snapshot.treatment_stage_count,
        json_runtime_clip_processing_snapshot_vec(&snapshot.clips),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
