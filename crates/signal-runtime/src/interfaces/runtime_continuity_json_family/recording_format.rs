use super::*;

pub(crate) fn format_runtime_recording_capture_snapshot_compact(
    snapshot: &RuntimeRecordingCaptureSnapshot,
) -> String {
    let checkpoint_class = snapshot
        .active_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.checkpoint_class)
        .or_else(|| {
            snapshot
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.checkpoint_class)
        });
    let interruption_class = snapshot
        .active_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.interruption_class)
        .or_else(|| {
            snapshot
                .last_checkpoint
                .as_ref()
                .map(|checkpoint| checkpoint.interruption_class)
        });
    format!(
        " recording_capture={:?}/{:?}/{:?} ready={} take={:?} track={:?} frames={} events={} blocks={} pressure={} last_take={:?} last_path={:?} last_duration={:?}",
        snapshot.state,
        snapshot.capture_kind,
        checkpoint_class,
        snapshot.capture_ready,
        snapshot.active_take_id,
        snapshot.active_track_id,
        snapshot.buffered_frame_count,
        snapshot.buffered_event_count,
        snapshot.buffered_block_count,
        snapshot.pressure_event_count,
        snapshot.last_committed_take_id,
        snapshot.last_committed_path,
        snapshot.last_committed_duration_samples,
    ) + &format!(" recording_capture_interruption={interruption_class:?}")
}

pub(crate) fn format_runtime_recording_capture_snapshot_multiline(
    snapshot: &RuntimeRecordingCaptureSnapshot,
) -> String {
    format!(
        concat!(
            "\nrecording_capture_ready={}",
            "\nrecording_capture_state={:?}",
            "\nrecording_capture_kind={:?}",
            "\nrecording_capture_active_take_id={:?}",
            "\nrecording_capture_active_track_id={:?}",
            "\nrecording_capture_start_samples={:?}",
            "\nrecording_capture_active_path={:?}",
            "\nrecording_capture_buffered_blocks={}",
            "\nrecording_capture_buffered_frames={}",
            "\nrecording_capture_buffered_events={}",
            "\nrecording_capture_channel_count={}",
            "\nrecording_capture_peak_level={:?}",
            "\nrecording_capture_pressure_events={}",
            "\nrecording_capture_active_checkpoint={}",
            "\nrecording_capture_last_checkpoint={}",
            "\nrecording_capture_last_committed_take_id={:?}",
            "\nrecording_capture_last_committed_path={:?}",
            "\nrecording_capture_last_committed_duration_samples={:?}",
            "\nrecording_capture_last_error={:?}",
            "\nrecording_capture_summary={}",
        ),
        snapshot.capture_ready,
        snapshot.state,
        snapshot.capture_kind,
        snapshot.active_take_id,
        snapshot.active_track_id,
        snapshot.capture_start_samples,
        snapshot.active_capture_path,
        snapshot.buffered_block_count,
        snapshot.buffered_frame_count,
        snapshot.buffered_event_count,
        snapshot.captured_channel_count,
        snapshot.peak_level,
        snapshot.pressure_event_count,
        snapshot
            .active_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.summary.as_str())
            .unwrap_or("none"),
        snapshot
            .last_checkpoint
            .as_ref()
            .map(|checkpoint| checkpoint.summary.as_str())
            .unwrap_or("none"),
        snapshot.last_committed_take_id,
        snapshot.last_committed_path,
        snapshot.last_committed_duration_samples,
        snapshot.last_error,
        snapshot.summary,
    )
}
