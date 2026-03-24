use super::*;

pub(crate) fn format_runtime_offline_render_session_snapshot_compact(
    snapshot: &RuntimeOfflineRenderSessionSnapshot,
) -> String {
    let active = snapshot
        .active_sessions
        .first()
        .map(|session| format!("{}:{:?}", session.request_id, session.state))
        .unwrap_or_else(|| "none".into());
    let last = snapshot
        .last_session
        .as_ref()
        .map(|session| format!("{}:{:?}", session.request_id, session.state))
        .unwrap_or_else(|| "none".into());
    let last_checkpoint = snapshot
        .last_session
        .as_ref()
        .and_then(|session| session.last_checkpoint.as_ref())
        .map(|checkpoint| format!("{:?}", checkpoint.stage))
        .unwrap_or_else(|| "none".into());
    format!(
        " offline_render_sessions={}/{}/{} active={} last={} last_checkpoint={} last_cancellation={} last_purge={}",
        snapshot.active_session_count,
        snapshot.paused_session_count,
        snapshot.recoverable_session_count,
        active,
        last,
        last_checkpoint,
        snapshot.last_cancellation.is_some(),
        snapshot.last_purge.is_some(),
    )
}

pub(crate) fn format_runtime_offline_render_session_snapshot_multiline(
    snapshot: &RuntimeOfflineRenderSessionSnapshot,
) -> String {
    let active = snapshot
        .active_sessions
        .iter()
        .map(|session| session.summary.as_str())
        .collect::<Vec<_>>()
        .join(" | ");
    format!(
        concat!(
            "\noffline_render_session_active_count={}",
            "\noffline_render_session_paused_count={}",
            "\noffline_render_session_recoverable_count={}",
            "\noffline_render_session_active_summaries={}",
            "\noffline_render_session_last_summary={}",
            "\noffline_render_session_last_cancellation={}",
            "\noffline_render_session_last_purge={}",
            "\noffline_render_session_summary={}",
        ),
        snapshot.active_session_count,
        snapshot.paused_session_count,
        snapshot.recoverable_session_count,
        if active.is_empty() {
            "none"
        } else {
            active.as_str()
        },
        snapshot
            .last_session
            .as_ref()
            .map(|session| session.summary.as_str())
            .unwrap_or("none"),
        snapshot
            .last_cancellation
            .as_ref()
            .map(|receipt| receipt.summary.as_str())
            .unwrap_or("none"),
        snapshot
            .last_purge
            .as_ref()
            .map(|receipt| receipt.summary.as_str())
            .unwrap_or("none"),
        snapshot.summary,
    )
}
