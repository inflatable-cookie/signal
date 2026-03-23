use super::*;

fn json_runtime_media_asset_snapshot(snapshot: &RuntimeMediaAssetSnapshot) -> String {
    format!(
        concat!(
            "{{",
            "\"asset_id\":{},",
            "\"content_hash\":{},",
            "\"source_path\":{},",
            "\"file_name\":{},",
            "\"byte_size\":{},",
            "\"sample_rate_hz\":{},",
            "\"channel_count\":{},",
            "\"duration_samples\":{},",
            "\"waveform_bin_count\":{},",
            "\"state\":{},",
            "\"cache_path\":{},",
            "\"cache_byte_size\":{},",
            "\"rebuild_count\":{},",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.asset_id.as_str())),
        json_option_string(Some(snapshot.content_hash.as_str())),
        json_option_string(Some(snapshot.source_path.as_str())),
        json_option_string(Some(snapshot.file_name.as_str())),
        snapshot.byte_size,
        snapshot.sample_rate_hz,
        snapshot.channel_count,
        snapshot.duration_samples,
        snapshot.waveform_bin_count,
        json_option_string(snapshot.state.map(|value| format!("{value:?}")).as_deref()),
        json_option_string(snapshot.cache_path.as_deref()),
        json_option_u64(snapshot.cache_byte_size),
        snapshot.rebuild_count,
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_media_asset_snapshot_vec(snapshots: &[RuntimeMediaAssetSnapshot]) -> String {
    let joined = snapshots
        .iter()
        .map(json_runtime_media_asset_snapshot)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_runtime_media_pipeline_snapshot(
    snapshot: &RuntimeMediaPipelineSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"cache_root_path\":{},",
            "\"asset_count\":{},",
            "\"ready_asset_count\":{},",
            "\"invalid_asset_count\":{},",
            "\"ingesting_asset_count\":{},",
            "\"conforming_asset_count\":{},",
            "\"rebuilding_asset_count\":{},",
            "\"assets\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.cache_root_path.as_str())),
        snapshot.asset_count,
        snapshot.ready_asset_count,
        snapshot.invalid_asset_count,
        snapshot.ingesting_asset_count,
        snapshot.conforming_asset_count,
        snapshot.rebuilding_asset_count,
        json_runtime_media_asset_snapshot_vec(&snapshot.assets),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

pub(crate) fn json_runtime_media_service_snapshot(
    snapshot: &RuntimeMediaServiceSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"indexed_asset_count\":{},",
            "\"analysis_ready_asset_count\":{},",
            "\"waveform_ready_asset_count\":{},",
            "\"waveform_pending_asset_count\":{},",
            "\"previewable_asset_count\":{},",
            "\"invalidated_asset_count\":{},",
            "\"invalidation_active\":{},",
            "\"indexing_state\":{},",
            "\"preview_state\":{},",
            "\"previewing_asset_id\":{},",
            "\"last_invalidated_asset_id\":{},",
            "\"last_invalidation_error\":{},",
            "\"last_preview_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.indexed_asset_count,
        snapshot.analysis_ready_asset_count,
        snapshot.waveform_ready_asset_count,
        snapshot.waveform_pending_asset_count,
        snapshot.previewable_asset_count,
        snapshot.invalidated_asset_count,
        snapshot.invalidation_active,
        json_option_string(Some(&format!("{:?}", snapshot.indexing_state))),
        json_option_string(Some(&format!("{:?}", snapshot.preview_state))),
        json_option_string(snapshot.previewing_asset_id.as_deref()),
        json_option_string(snapshot.last_invalidated_asset_id.as_deref()),
        json_option_string(snapshot.last_invalidation_error.as_deref()),
        json_option_string(snapshot.last_preview_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
