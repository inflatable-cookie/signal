use super::*;
pub(super) fn json_runtime_preview_transform_service_snapshot(
    snapshot: &RuntimePreviewTransformServiceSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_count\":{},",
            "\"active_audition_clip_count\":{},",
            "\"scrub_supported_clip_count\":{},",
            "\"ready_clip_count\":{},",
            "\"pending_clip_count\":{},",
            "\"degraded_clip_count\":{},",
            "\"invalidated_clip_count\":{},",
            "\"unsupported_clip_count\":{},",
            "\"stretch_aligned_clip_count\":{},",
            "\"artifact_backed_clip_count\":{},",
            "\"fallback_clip_count\":{},",
            "\"preview_device_policy\":{},",
            "\"preview_workflow\":{},",
            "\"clips\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.clip_count,
        snapshot.active_audition_clip_count,
        snapshot.scrub_supported_clip_count,
        snapshot.ready_clip_count,
        snapshot.pending_clip_count,
        snapshot.degraded_clip_count,
        snapshot.invalidated_clip_count,
        snapshot.unsupported_clip_count,
        snapshot.stretch_aligned_clip_count,
        snapshot.artifact_backed_clip_count,
        snapshot.fallback_clip_count,
        json_runtime_preview_device_policy_summary(&snapshot.preview_device_policy),
        json_runtime_preview_workflow_summary(&snapshot.preview_workflow),
        format!(
            "[{}]",
            snapshot
                .clips
                .iter()
                .map(json_runtime_preview_transform_clip_snapshot)
                .collect::<Vec<_>>()
                .join(",")
        ),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_preview_transform_clip_snapshot(
    snapshot: &RuntimePreviewTransformClipSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"clip_id\":{},",
            "\"media_asset_id\":{},",
            "\"service_class\":{},",
            "\"readiness\":{},",
            "\"degraded_state\":{},",
            "\"fallback_kind\":{},",
            "\"artifact_reuse_state\":{},",
            "\"audition_active\":{},",
            "\"scrub_supported\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(snapshot.clip_id.as_str())),
        json_option_string(snapshot.media_asset_id.as_deref()),
        json_string(&format!("{:?}", snapshot.service_class)),
        json_string(&format!("{:?}", snapshot.readiness)),
        json_string(&format!("{:?}", snapshot.degraded_state)),
        json_string(&format!("{:?}", snapshot.fallback_kind)),
        json_string(&format!("{:?}", snapshot.artifact_reuse_state)),
        snapshot.audition_active,
        snapshot.scrub_supported,
        json_option_string(Some(snapshot.summary.as_str())),
    )
}

fn json_runtime_preview_device_policy_summary(
    summary: &RuntimePreviewDevicePolicySummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"routing_posture\":{},",
            "\"audition_sink_class\":{},",
            "\"audition_sink_authority\":{},",
            "\"low_latency_device_policy_class\":{},",
            "\"low_latency_device_policy_outcome\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", summary.routing_posture)),
        json_string(&format!("{:?}", summary.audition_sink_class)),
        json_string(&format!("{:?}", summary.audition_sink_authority)),
        json_string(&format!("{:?}", summary.low_latency_device_policy_class)),
        json_string(&format!("{:?}", summary.low_latency_device_policy_outcome)),
        json_option_string(Some(summary.summary.as_str())),
    )
}

fn json_runtime_preview_workflow_summary(summary: &RuntimePreviewWorkflowSummary) -> String {
    format!(
        concat!(
            "{{",
            "\"queue_posture\":{},",
            "\"queue_class\":{},",
            "\"queue_outcome\":{},",
            "\"audition_posture\":{},",
            "\"audition_authority\":{},",
            "\"audition_continuity_outcome\":{},",
            "\"transform_scheduling_posture\":{},",
            "\"transform_scheduling_authority\":{},",
            "\"transform_scheduling_outcome\":{},",
            "\"queued_preview_request_count\":{},",
            "\"previewable_asset_count\":{},",
            "\"active_audition_clip_count\":{},",
            "\"pending_transform_clip_count\":{},",
            "\"ready_transform_clip_count\":{},",
            "\"fallback_transform_clip_count\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_string(&format!("{:?}", summary.queue_posture)),
        json_string(&format!("{:?}", summary.queue_class)),
        json_string(&format!("{:?}", summary.queue_outcome)),
        json_string(&format!("{:?}", summary.audition_posture)),
        json_string(&format!("{:?}", summary.audition_authority)),
        json_string(&format!("{:?}", summary.audition_continuity_outcome)),
        json_string(&format!("{:?}", summary.transform_scheduling_posture)),
        json_string(&format!("{:?}", summary.transform_scheduling_authority)),
        json_string(&format!("{:?}", summary.transform_scheduling_outcome)),
        summary.queued_preview_request_count,
        summary.previewable_asset_count,
        summary.active_audition_clip_count,
        summary.pending_transform_clip_count,
        summary.ready_transform_clip_count,
        summary.fallback_transform_clip_count,
        json_option_string(Some(summary.summary.as_str())),
    )
}
