use super::*;

pub(crate) fn format_runtime_stretch_engine_snapshot_compact(
    snapshot: &RuntimeStretchEngineSnapshot,
) -> String {
    format!(
        " stretch_clips={}/{}/{}/{}/{}/{}/{}/{}",
        snapshot.ready_clip_count,
        snapshot.clip_count,
        snapshot.sample_domain_clip_count,
        snapshot.ratio_only_clip_count,
        snapshot.fallback_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count,
        snapshot.degraded_clip_count,
    )
}

pub(crate) fn format_runtime_stretch_engine_snapshot_multiline(
    snapshot: &RuntimeStretchEngineSnapshot,
) -> String {
    let clip_lines = snapshot
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            format!(
                "\nstretch_clip_{}={}/engine={:?}/readiness={:?}/fallback={:?}",
                index, clip.clip_id, clip.engine_class, clip.readiness, clip.fallback_kind
            )
        })
        .collect::<String>();
    format!(
        "\nstretch_clip_count={}\nstretch_disabled_clip_count={}\nstretch_ready_clip_count={}\nstretch_pending_media_clip_count={}\nstretch_pending_warp_clip_count={}\nstretch_degraded_clip_count={}\nstretch_sample_domain_clip_count={}\nstretch_ratio_only_clip_count={}\nstretch_fallback_clip_count={}{}",
        snapshot.clip_count,
        snapshot.disabled_clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.pending_warp_clip_count,
        snapshot.degraded_clip_count,
        snapshot.sample_domain_clip_count,
        snapshot.ratio_only_clip_count,
        snapshot.fallback_clip_count,
        clip_lines,
    )
}

pub(crate) fn format_runtime_marker_analysis_snapshot_compact(
    snapshot: &RuntimeMarkerAnalysisSnapshot,
) -> String {
    format!(
        " marker_analysis_clips={}/{}/{}/{}/{} marker_analysis_counts={}/{} marker_analysis_tempo_assist={}",
        snapshot.ready_clip_count,
        snapshot.clip_count,
        snapshot.pending_media_clip_count,
        snapshot.degraded_clip_count,
        snapshot.invalidated_clip_count + snapshot.unsupported_clip_count,
        snapshot.warp_marker_count,
        snapshot.transient_anchor_count,
        snapshot.tempo_assist_ready_clip_count,
    )
}

pub(crate) fn format_runtime_marker_analysis_snapshot_multiline(
    snapshot: &RuntimeMarkerAnalysisSnapshot,
) -> String {
    let clip_lines = snapshot
        .clips
        .iter()
        .enumerate()
        .map(|(index, clip)| {
            format!(
                "\nmarker_analysis_clip_{}={}/readiness={:?}/invalidation={:?}/markers={}/anchors={}/tempo_assist={:?}/{:?}/{:?}/error={:?}",
                index,
                clip.clip_id,
                clip.readiness,
                clip.invalidation_state,
                clip.warp_marker_count,
                clip.transient_anchor_count,
                clip.tempo_assist_posture,
                clip.tempo_assist_hint_source,
                clip.tempo_assist_hint_bpm,
                clip.last_error,
            )
        })
        .collect::<String>();
    format!(
        "\nmarker_analysis_clip_count={}\nmarker_analysis_ready_clip_count={}\nmarker_analysis_pending_media_clip_count={}\nmarker_analysis_degraded_clip_count={}\nmarker_analysis_invalidated_clip_count={}\nmarker_analysis_unsupported_clip_count={}\nmarker_analysis_tempo_assist_ready_clip_count={}\nmarker_analysis_warp_marker_count={}\nmarker_analysis_transient_anchor_count={}{}",
        snapshot.clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_media_clip_count,
        snapshot.degraded_clip_count,
        snapshot.invalidated_clip_count,
        snapshot.unsupported_clip_count,
        snapshot.tempo_assist_ready_clip_count,
        snapshot.warp_marker_count,
        snapshot.transient_anchor_count,
        clip_lines,
    )
}
