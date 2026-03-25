use super::*;

pub(crate) fn format_runtime_tempo_map_snapshot_compact(
    snapshot: &RuntimeTempoMapSnapshot,
) -> String {
    format!(
        " tempo_map_segments={} tempo_map_active={:?}/{:?} tempo_map_source={:?} tempo_map_tempo={:.3} tempo_map_next_segment={:?}",
        snapshot.segment_count,
        snapshot.active_segment_index,
        snapshot.active_segment_id,
        snapshot.tempo_source,
        snapshot.resolved_tempo_bpm,
        snapshot.next_segment_start_samples,
    )
}

pub(crate) fn format_runtime_tempo_map_snapshot_multiline(
    snapshot: &RuntimeTempoMapSnapshot,
) -> String {
    let segment_lines = snapshot
        .segments
        .iter()
        .enumerate()
        .map(|(index, segment)| {
            format!(
                "\ntempo_map_segment_{}={}/interp={:?}/start={}/end={:?}/tempo={:.3}->{:?}/active={}",
                index,
                segment.segment_id,
                segment.interpolation,
                segment.start_samples,
                segment.end_samples,
                segment.start_tempo_bpm,
                segment.end_tempo_bpm,
                segment.covers_timeline_position,
            )
        })
        .collect::<String>();
    format!(
        "\ntempo_map_segment_count={}\ntempo_map_active_segment_id={:?}\ntempo_map_active_segment_index={:?}\ntempo_map_next_segment_start_samples={:?}\ntempo_map_resolved_tempo_bpm={:.3}\ntempo_map_source={:?}\ntempo_map_timeline_position_samples={:?}{}",
        snapshot.segment_count,
        snapshot.active_segment_id,
        snapshot.active_segment_index,
        snapshot.next_segment_start_samples,
        snapshot.resolved_tempo_bpm,
        snapshot.tempo_source,
        snapshot.timeline_position_samples,
        segment_lines,
    )
}

pub(crate) fn format_runtime_warp_pipeline_snapshot_compact(
    snapshot: &RuntimeWarpPipelineSnapshot,
) -> String {
    format!(
        " warp_clips={}/{}/{}/{} warp_tempo={:.3}/{:?}/{:?}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.degraded_clip_count,
        snapshot.bypassed_clip_count,
        snapshot.resolved_project_tempo_bpm,
        snapshot.resolved_project_tempo_source,
        snapshot.resolved_project_tempo_segment_id,
    )
}

pub(crate) fn format_runtime_warp_pipeline_snapshot_multiline(
    snapshot: &RuntimeWarpPipelineSnapshot,
) -> String {
    let clip_lines = snapshot
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            format!(
                "\nwarp_clip_{}={}/mode={:?}/readiness={:?}/ratio={:.3}/project_tempo={:.3}/{:?}/{:?}/source_tempo={:?}/error={:?}",
                index,
                clip.clip_id,
                clip.mode,
                clip.readiness,
                clip.realized_ratio,
                clip.project_tempo_bpm,
                clip.project_tempo_source,
                clip.project_tempo_segment_id,
                clip.source_tempo_bpm,
                clip.last_error,
            )
        })
        .collect::<String>();
    format!(
        "\nwarp_clip_count={}\nwarp_ready_clip_count={}\nwarp_degraded_clip_count={}\nwarp_bypassed_clip_count={}\nwarp_active_clip_count={}\nwarp_resolved_project_tempo_bpm={:.3}\nwarp_resolved_project_tempo_source={:?}\nwarp_resolved_project_tempo_segment_id={:?}{}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.degraded_clip_count,
        snapshot.bypassed_clip_count,
        snapshot.active_warp_count,
        snapshot.resolved_project_tempo_bpm,
        snapshot.resolved_project_tempo_source,
        snapshot.resolved_project_tempo_segment_id,
        clip_lines,
    )
}

pub(crate) fn format_runtime_clip_processing_pipeline_snapshot_compact(
    snapshot: &RuntimeClipProcessingPipelineSnapshot,
) -> String {
    format!(
        " clip_processing_clips={}/{}/{}/{} clip_processing_shapes={}/{}/{} clip_processing_treatment_stages={}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count + snapshot.invalid_clip_count,
        snapshot.faded_clip_count,
        snapshot.gain_shaped_clip_count,
        snapshot.warped_clip_count,
        snapshot.treatment_stage_count,
    )
}

pub(crate) fn format_runtime_clip_processing_pipeline_snapshot_multiline(
    snapshot: &RuntimeClipProcessingPipelineSnapshot,
) -> String {
    let clip_lines = snapshot
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            format!(
                "\nclip_processing_clip_{}={}/readiness={:?}/warp={:?}/{:?}/{:?}/fade_in={}/{:?}/fade_out={}/{:?}/gain={:.3}->{:.3}/{:?}/stages={:?}/error={:?}",
                index,
                clip.clip_id,
                clip.readiness,
                clip.warp_mode,
                clip.realized_warp_ratio,
                clip.project_tempo_source,
                clip.fade_in.duration_samples,
                clip.fade_in.shape,
                clip.fade_out.duration_samples,
                clip.fade_out.shape,
                clip.clip_gain.start_linear,
                clip.clip_gain.end_linear,
                clip.clip_gain.shape,
                clip.treatment_stages,
                clip.last_error,
            )
        })
        .collect::<String>();
    format!(
        "\nclip_processing_clip_count={}\nclip_processing_ready_clip_count={}\nclip_processing_pending_media_clip_count={}\nclip_processing_pending_warp_clip_count={}\nclip_processing_invalid_clip_count={}\nclip_processing_faded_clip_count={}\nclip_processing_gain_shaped_clip_count={}\nclip_processing_warped_clip_count={}\nclip_processing_treatment_stage_count={}{}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count,
        snapshot.invalid_clip_count,
        snapshot.faded_clip_count,
        snapshot.gain_shaped_clip_count,
        snapshot.warped_clip_count,
        snapshot.treatment_stage_count,
        clip_lines,
    )
}
