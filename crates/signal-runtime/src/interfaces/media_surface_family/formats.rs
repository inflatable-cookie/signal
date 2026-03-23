use super::*;

pub(crate) fn format_runtime_media_pipeline_snapshot_compact(
    snapshot: &RuntimeMediaPipelineSnapshot,
) -> String {
    format!(
        " media_pipeline_assets={}/{}/{} media_pipeline_state={}/{}/{} media_pipeline_cache_root={}",
        snapshot.asset_count,
        snapshot.ready_asset_count,
        snapshot.invalid_asset_count,
        snapshot.ingesting_asset_count,
        snapshot.conforming_asset_count,
        snapshot.rebuilding_asset_count,
        snapshot.cache_root_path,
    )
}

pub(crate) fn format_runtime_media_pipeline_snapshot_multiline(
    snapshot: &RuntimeMediaPipelineSnapshot,
) -> String {
    format!(
        "\nmedia_pipeline_cache_root_path={}\nmedia_pipeline_asset_count={}\nmedia_pipeline_ready_asset_count={}\nmedia_pipeline_invalid_asset_count={}\nmedia_pipeline_ingesting_asset_count={}\nmedia_pipeline_conforming_asset_count={}\nmedia_pipeline_rebuilding_asset_count={}\nmedia_pipeline_assets={:?}\nmedia_pipeline_summary={}",
        snapshot.cache_root_path,
        snapshot.asset_count,
        snapshot.ready_asset_count,
        snapshot.invalid_asset_count,
        snapshot.ingesting_asset_count,
        snapshot.conforming_asset_count,
        snapshot.rebuilding_asset_count,
        snapshot.assets,
        snapshot.summary,
    )
}

pub(crate) fn format_runtime_media_service_snapshot_compact(
    snapshot: &RuntimeMediaServiceSnapshot,
) -> String {
    format!(
        " media_service_assets={}/{}/{}/{} media_service_preview={:?}/{:?}/{:?} media_service_invalidated={} media_service_errors={:?}/{:?}",
        snapshot.indexed_asset_count,
        snapshot.analysis_ready_asset_count,
        snapshot.waveform_ready_asset_count,
        snapshot.previewable_asset_count,
        snapshot.indexing_state,
        snapshot.preview_state,
        snapshot.previewing_asset_id,
        snapshot.invalidated_asset_count,
        snapshot.last_invalidation_error,
        snapshot.last_preview_error,
    )
}

pub(crate) fn format_runtime_media_service_snapshot_multiline(
    snapshot: &RuntimeMediaServiceSnapshot,
) -> String {
    format!(
        "\nmedia_service_indexed_asset_count={}\nmedia_service_analysis_ready_asset_count={}\nmedia_service_waveform_ready_asset_count={}\nmedia_service_waveform_pending_asset_count={}\nmedia_service_previewable_asset_count={}\nmedia_service_invalidated_asset_count={}\nmedia_service_invalidation_active={}\nmedia_service_indexing_state={:?}\nmedia_service_preview_state={:?}\nmedia_service_previewing_asset_id={:?}\nmedia_service_last_invalidated_asset_id={:?}\nmedia_service_last_invalidation_error={:?}\nmedia_service_last_preview_error={:?}\nmedia_service_summary={}",
        snapshot.indexed_asset_count,
        snapshot.analysis_ready_asset_count,
        snapshot.waveform_ready_asset_count,
        snapshot.waveform_pending_asset_count,
        snapshot.previewable_asset_count,
        snapshot.invalidated_asset_count,
        snapshot.invalidation_active,
        snapshot.indexing_state,
        snapshot.preview_state,
        snapshot.previewing_asset_id,
        snapshot.last_invalidated_asset_id,
        snapshot.last_invalidation_error,
        snapshot.last_preview_error,
        snapshot.summary,
    )
}

pub(crate) fn format_runtime_media_library_service_snapshot_compact(
    snapshot: &RuntimeMediaLibraryServiceSnapshot,
) -> String {
    format!(
        " media_library_assets={}/{}/{}/{} media_library_analysis={}/{}/{}/{}/{}",
        snapshot.indexed_asset_count,
        snapshot.ready_descriptor_count,
        snapshot.pending_descriptor_count,
        snapshot.invalidated_descriptor_count,
        snapshot.loudness_ready_descriptor_count,
        snapshot.character_ready_descriptor_count,
        snapshot.rhythm_deferred_descriptor_count,
        snapshot.tonal_deferred_descriptor_count,
        snapshot.embedding_deferred_descriptor_count,
    )
}

pub(crate) fn format_runtime_media_library_service_snapshot_multiline(
    snapshot: &RuntimeMediaLibraryServiceSnapshot,
) -> String {
    format!(
        "\nmedia_library_indexed_asset_count={}\nmedia_library_ready_descriptor_count={}\nmedia_library_pending_descriptor_count={}\nmedia_library_invalidated_descriptor_count={}\nmedia_library_unavailable_descriptor_count={}\nmedia_library_loudness_ready_descriptor_count={}\nmedia_library_character_ready_descriptor_count={}\nmedia_library_rhythm_deferred_descriptor_count={}\nmedia_library_tonal_deferred_descriptor_count={}\nmedia_library_embedding_deferred_descriptor_count={}\nmedia_library_descriptors={:?}\nmedia_library_summary={}",
        snapshot.indexed_asset_count,
        snapshot.ready_descriptor_count,
        snapshot.pending_descriptor_count,
        snapshot.invalidated_descriptor_count,
        snapshot.unavailable_descriptor_count,
        snapshot.loudness_ready_descriptor_count,
        snapshot.character_ready_descriptor_count,
        snapshot.rhythm_deferred_descriptor_count,
        snapshot.tonal_deferred_descriptor_count,
        snapshot.embedding_deferred_descriptor_count,
        snapshot.descriptors,
        snapshot.summary,
    )
}
