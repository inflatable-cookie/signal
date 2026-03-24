use super::*;

pub(crate) fn json_runtime_recording_capture_checkpoint(
    checkpoint: &RuntimeRecordingCaptureCheckpointSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"capture_kind\":{},",
            "\"checkpoint_class\":{},",
            "\"interruption_class\":{},",
            "\"take_id\":{},",
            "\"track_id\":{},",
            "\"capture_start_samples\":{},",
            "\"capture_path\":{},",
            "\"buffered_block_count\":{},",
            "\"buffered_frame_count\":{},",
            "\"buffered_event_count\":{},",
            "\"captured_channel_count\":{},",
            "\"peak_level\":{},",
            "\"pressure_event_count\":{},",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        json_option_string(Some(match checkpoint.capture_kind {
            RuntimeRecordingCaptureKind::Audio => "Audio",
            RuntimeRecordingCaptureKind::Midi => "Midi",
        })),
        json_option_string(Some(match checkpoint.checkpoint_class {
            RuntimeRecordingCaptureCheckpointClass::Armed => "Armed",
            RuntimeRecordingCaptureCheckpointClass::Streaming => "Streaming",
            RuntimeRecordingCaptureCheckpointClass::Buffered => "Buffered",
            RuntimeRecordingCaptureCheckpointClass::Committed => "Committed",
            RuntimeRecordingCaptureCheckpointClass::Failed => "Failed",
        })),
        json_option_string(Some(match checkpoint.interruption_class {
            RuntimeInterruptionClass::Steady => "Steady",
            RuntimeInterruptionClass::Resumable => "Resumable",
            RuntimeInterruptionClass::Restartable => "Restartable",
            RuntimeInterruptionClass::Recoverable => "Recoverable",
            RuntimeInterruptionClass::Terminal => "Terminal",
        })),
        json_option_string(Some(checkpoint.take_id.as_str())),
        json_option_string(Some(checkpoint.track_id.as_str())),
        checkpoint.capture_start_samples,
        json_option_string(Some(checkpoint.capture_path.as_str())),
        checkpoint.buffered_block_count,
        checkpoint.buffered_frame_count,
        checkpoint.buffered_event_count,
        checkpoint.captured_channel_count,
        json_option_f32(checkpoint.peak_level),
        checkpoint.pressure_event_count,
        json_option_string(checkpoint.last_error.as_deref()),
        json_option_string(Some(checkpoint.summary.as_str())),
    )
}

pub(crate) fn json_runtime_recording_capture_snapshot(
    snapshot: &RuntimeRecordingCaptureSnapshot,
) -> String {
    format!(
        concat!(
            "{{",
            "\"capture_ready\":{},",
            "\"state\":{},",
            "\"capture_kind\":{},",
            "\"active_take_id\":{},",
            "\"active_track_id\":{},",
            "\"capture_start_samples\":{},",
            "\"active_capture_path\":{},",
            "\"buffered_block_count\":{},",
            "\"buffered_frame_count\":{},",
            "\"buffered_event_count\":{},",
            "\"captured_channel_count\":{},",
            "\"peak_level\":{},",
            "\"pressure_event_count\":{},",
            "\"active_checkpoint\":{},",
            "\"last_checkpoint\":{},",
            "\"last_committed_take_id\":{},",
            "\"last_committed_path\":{},",
            "\"last_committed_duration_samples\":{},",
            "\"last_error\":{},",
            "\"summary\":{}",
            "}}"
        ),
        snapshot.capture_ready,
        json_option_string(snapshot.state.map(|state| match state {
            RuntimeRecordingCaptureState::Idle => "Idle",
            RuntimeRecordingCaptureState::Capturing => "Capturing",
            RuntimeRecordingCaptureState::Failed => "Failed",
        })),
        json_option_string(snapshot.capture_kind.map(|kind| match kind {
            RuntimeRecordingCaptureKind::Audio => "Audio",
            RuntimeRecordingCaptureKind::Midi => "Midi",
        })),
        json_option_string(snapshot.active_take_id.as_deref()),
        json_option_string(snapshot.active_track_id.as_deref()),
        json_option_i64(snapshot.capture_start_samples),
        json_option_string(snapshot.active_capture_path.as_deref()),
        snapshot.buffered_block_count,
        snapshot.buffered_frame_count,
        snapshot.buffered_event_count,
        snapshot.captured_channel_count,
        json_option_f32(snapshot.peak_level),
        snapshot.pressure_event_count,
        snapshot
            .active_checkpoint
            .as_ref()
            .map(json_runtime_recording_capture_checkpoint)
            .unwrap_or_else(|| "null".into()),
        snapshot
            .last_checkpoint
            .as_ref()
            .map(json_runtime_recording_capture_checkpoint)
            .unwrap_or_else(|| "null".into()),
        json_option_string(snapshot.last_committed_take_id.as_deref()),
        json_option_string(snapshot.last_committed_path.as_deref()),
        json_option_u32(snapshot.last_committed_duration_samples),
        json_option_string(snapshot.last_error.as_deref()),
        json_option_string(Some(snapshot.summary.as_str())),
    )
}
