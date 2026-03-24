use super::*;

pub(crate) fn json_runtime_offline_render_session_snapshot(
    snapshot: &RuntimeOfflineRenderSessionSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"active_session_count\":{},",
            "\"paused_session_count\":{},",
            "\"recoverable_session_count\":{},",
            "\"active_sessions\":{},",
            "\"last_session\":{},",
            "\"last_cancellation\":{},",
            "\"last_purge\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.active_session_count,
        snapshot.paused_session_count,
        snapshot.recoverable_session_count,
        format!(
            "[{}]",
            snapshot
                .active_sessions
                .iter()
                .map(json_runtime_offline_render_session_state_snapshot)
                .collect::<Vec<_>>()
                .join(",")
        ),
        snapshot
            .last_session
            .as_ref()
            .map(json_runtime_offline_render_session_state_snapshot)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_cancellation
            .as_ref()
            .map(json_runtime_offline_render_execution_cancellation_receipt)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_purge
            .as_ref()
            .map(json_runtime_offline_render_purge_receipt)
            .unwrap_or_else(|| "null".into()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
