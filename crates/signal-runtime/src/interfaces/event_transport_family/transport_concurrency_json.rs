use super::*;

pub(crate) fn json_runtime_transport_concurrency_snapshot(
    snapshot: &RuntimeTransportConcurrencySnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"steady_session_limit\":{},",
            "\"recovery_session_limit\":{},",
            "\"current_attached_sessions\":{},",
            "\"peak_attached_sessions\":{},",
            "\"current_recovery_overlap_sessions\":{},",
            "\"peak_recovery_overlap_sessions\":{},",
            "\"current_lingering_sessions\":{},",
            "\"peak_lingering_sessions\":{},",
            "\"current_detach_requested_sessions\":{},",
            "\"current_detach_faulted_sessions\":{},",
            "\"pending_cleanup_work_items\":{},",
            "\"pending_deferred_retry_work_items\":{},",
            "\"next_cleanup_epoch\":{},",
            "\"oldest_pending_cleanup_ready_epoch\":{},",
            "\"pending_cleanup_waves\":{},",
            "\"active_sessions\":{},",
            "\"last_admitted_sandbox_id\":{},",
            "\"last_rejected_sandbox_id\":{},",
            "\"last_rejection_reason\":{}",
            "}}"
        ),
        snapshot.steady_session_limit,
        snapshot.recovery_session_limit,
        snapshot.current_attached_sessions,
        snapshot.peak_attached_sessions,
        snapshot.current_recovery_overlap_sessions,
        snapshot.peak_recovery_overlap_sessions,
        snapshot.current_lingering_sessions,
        snapshot.peak_lingering_sessions,
        snapshot.current_detach_requested_sessions,
        snapshot.current_detach_faulted_sessions,
        snapshot.pending_cleanup_work_items,
        snapshot.pending_deferred_retry_work_items,
        snapshot.next_cleanup_epoch,
        json_option_u64(snapshot.oldest_pending_cleanup_ready_epoch),
        json_pending_lingering_cleanup_wave_summary_vec(&snapshot.pending_cleanup_waves),
        json_active_transport_concurrency_session_vec(&snapshot.active_sessions),
        json_option_string(snapshot.last_admitted_sandbox_id.as_deref()),
        json_option_string(snapshot.last_rejected_sandbox_id.as_deref()),
        json_option_string(snapshot.last_rejection_reason.as_deref()),
    )
}

pub(crate) fn json_active_transport_concurrency_session(
    session: &ActiveTransportConcurrencySession,
) -> String {
    let last_cleanup_mode = session.last_cleanup_mode.map(|mode| format!("{mode:?}"));
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"intent\":{},",
            "\"provenance\":{},",
            "\"attach_sequence\":{},",
            "\"attach_processing_epoch\":{},",
            "\"state\":{},",
            "\"backing_path\":{},",
            "\"total_bytes\":{},",
            "\"cleanup_attempt_count\":{},",
            "\"last_cleanup_mode\":{},",
            "\"last_cleanup_wave\":{},",
            "\"cleanup_in_progress\":{},",
            "\"last_cleanup_epoch\":{},",
            "\"last_cleanup_error\":{}",
            "}}"
        ),
        json_escape_string(&session.sandbox_id),
        json_escape_string(&session.lease_id),
        json_escape_string(&session.region_id),
        json_escape_string(&format!("{:?}", session.intent)),
        json_escape_string(&format!("{:?}", session.provenance)),
        session.attach_sequence,
        json_option_u64(session.attach_processing_epoch),
        json_escape_string(&format!("{:?}", session.state)),
        json_option_string(session.backing_path.as_deref()),
        json_option_u64(session.total_bytes.map(u64::from)),
        session.cleanup_attempt_count,
        json_option_string(last_cleanup_mode.as_deref()),
        json_option_u64(session.last_cleanup_wave),
        session.cleanup_in_progress,
        json_option_u64(session.last_cleanup_epoch),
        json_option_string(session.last_cleanup_error.as_deref()),
    )
}

pub(crate) fn json_active_transport_concurrency_session_vec(
    sessions: &[ActiveTransportConcurrencySession],
) -> String {
    let joined = sessions
        .iter()
        .map(json_active_transport_concurrency_session)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

pub(crate) fn json_pending_lingering_cleanup_wave_summary(
    wave: &PendingLingeringCleanupWaveSummary,
) -> String {
    format!(
        concat!(
            "{{",
            "\"sandbox_id\":{},",
            "\"cleanup_wave\":{},",
            "\"mode\":{},",
            "\"first_trigger\":{},",
            "\"latest_trigger\":{},",
            "\"pending_work_items\":{},",
            "\"deferred_retry_work_items\":{},",
            "\"first_cleanup_epoch\":{},",
            "\"latest_cleanup_epoch\":{},",
            "\"first_processing_epoch\":{},",
            "\"latest_processing_epoch\":{},",
            "\"oldest_ready_at_processing_epoch\":{},",
            "\"newest_ready_at_processing_epoch\":{}",
            "}}"
        ),
        json_escape_string(&wave.sandbox_id),
        wave.cleanup_wave,
        json_escape_string(&format!("{:?}", wave.mode)),
        json_escape_string(&format!("{:?}", wave.first_trigger)),
        json_escape_string(&format!("{:?}", wave.latest_trigger)),
        wave.pending_work_items,
        wave.deferred_retry_work_items,
        wave.first_cleanup_epoch,
        wave.latest_cleanup_epoch,
        wave.first_processing_epoch,
        wave.latest_processing_epoch,
        wave.oldest_ready_at_processing_epoch,
        wave.newest_ready_at_processing_epoch,
    )
}

pub(crate) fn json_pending_lingering_cleanup_wave_summary_vec(
    waves: &[PendingLingeringCleanupWaveSummary],
) -> String {
    let joined = waves
        .iter()
        .map(json_pending_lingering_cleanup_wave_summary)
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}
