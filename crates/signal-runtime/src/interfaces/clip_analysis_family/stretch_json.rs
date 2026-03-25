use super::*;

fn json_runtime_stretch_clip_snapshot(snapshot: &RuntimeStretchClipSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"engine_class\":\"{:?}\",",
            "\"readiness\":\"{:?}\",",
            "\"fallback_kind\":\"{:?}\",",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        snapshot.engine_class,
        snapshot.readiness,
        snapshot.fallback_kind,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_stretch_clip_snapshot_vec(snapshots: &[RuntimeStretchClipSnapshot]) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_stretch_clip_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_stretch_engine_snapshot(
    snapshot: &RuntimeStretchEngineSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_count\":{},",
            "\"disabled_clip_count\":{},",
            "\"ready_clip_count\":{},",
            "\"pending_media_clip_count\":{},",
            "\"pending_warp_clip_count\":{},",
            "\"degraded_clip_count\":{},",
            "\"sample_domain_clip_count\":{},",
            "\"ratio_only_clip_count\":{},",
            "\"fallback_clip_count\":{},",
            "\"clips\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.clip_count,
        snapshot.disabled_clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count,
        snapshot.degraded_clip_count,
        snapshot.sample_domain_clip_count,
        snapshot.ratio_only_clip_count,
        snapshot.fallback_clip_count,
        json_runtime_stretch_clip_snapshot_vec(&snapshot.clips),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
