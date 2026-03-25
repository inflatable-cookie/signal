use super::*;

pub(crate) fn format_runtime_automation_snapshot_compact(
    snapshot: &RuntimeAutomationSnapshot,
) -> String {
    format!(
        " automation_param={} automation_projection={}/{}/{} automation_shapes={}/{} automation_batch_policy={}/{:?} automation_segments={} automation_first_epoch={:?} automation_last_epoch={:?} automation_lease_rollovers={}",
        snapshot.parameter_id,
        snapshot.lane_count,
        snapshot.point_count,
        snapshot.projected_segment_count,
        snapshot.hold_lane_count,
        snapshot.linear_lane_count,
        snapshot.last_batch_strategy_max_sub_blocks,
        snapshot.last_batch_min_ramp_step_samples,
        snapshot.segment_count,
        snapshot.first_epoch,
        snapshot.last_epoch,
        snapshot.lease_rollovers
    )
}

pub(crate) fn format_runtime_plugin_event_snapshot_compact(
    snapshot: &RuntimePluginEventSnapshot,
) -> String {
    format!(
        " plugin_events_last_batch={}/{}/{}/{}/{}/{}/{} plugin_events_total={}/{}/{}/{}/{}/{}/{} plugin_events_expression_last_batch={}/{}/{} plugin_events_expression_total={}/{}/{} plugin_events_posture={:?}/{:?} plugin_events_segments={} plugin_events_first_epoch={:?} plugin_events_last_epoch={:?} plugin_events_lease_rollovers={} plugin_events_last_block={:?} plugin_events_last_bytes={}",
        snapshot.last_batch_total_events,
        snapshot.last_batch_parameter_value_events,
        snapshot.last_batch_parameter_modulation_events,
        snapshot.last_batch_parameter_gesture_events,
        snapshot.last_batch_note_events,
        snapshot.last_batch_note_expression_events,
        snapshot.last_batch_midi_events,
        snapshot.total_events,
        snapshot.parameter_value_events,
        snapshot.parameter_modulation_events,
        snapshot.parameter_gesture_events,
        snapshot.note_events,
        snapshot.note_expression_events,
        snapshot.midi_events,
        snapshot.last_batch_note_expression_pressure_events,
        snapshot.last_batch_note_expression_timbre_events,
        snapshot.last_batch_note_expression_tuning_events,
        snapshot.note_expression_pressure_events,
        snapshot.note_expression_timbre_events,
        snapshot.note_expression_tuning_events,
        snapshot.mpe_posture,
        snapshot.midi2_posture,
        snapshot.segment_count,
        snapshot.first_epoch,
        snapshot.last_epoch,
        snapshot.lease_rollovers,
        snapshot.last_block_sequence,
        snapshot.last_generated_event_bytes,
    )
}

pub(crate) fn format_runtime_transport_timeline_compact(
    snapshot: &RuntimeTimelineSnapshot,
) -> String {
    format!(
        " transport_epoch={} transport_transition={:?} transport_transition_epoch={:?} transport_transition_block={:?} transport_playing={:?} transport_tempo_bpm={:?} transport_timeline_position_samples={:?} transport_loop_start_samples={:?} transport_loop_end_samples={:?} transport_last_block_start_samples={:?} transport_last_block_end_samples={:?} transport_loop_wrap_count={}",
        snapshot.transport_epoch,
        snapshot.last_transport_transition,
        snapshot.last_transport_transition_processing_epoch,
        snapshot.last_transport_transition_block_sequence,
        snapshot.last_transport_playing,
        snapshot.last_transport_tempo_bpm,
        snapshot.last_transport_timeline_position_samples,
        snapshot.last_transport_loop_start_samples,
        snapshot.last_transport_loop_end_samples,
        snapshot.last_engine_block_start_samples,
        snapshot.last_engine_block_end_samples,
        snapshot.loop_wrap_count,
    )
}

pub(crate) fn format_runtime_engine_transport_compact(
    snapshot: &RuntimeEngineBlockSnapshot,
) -> String {
    format!(
        " engine_transport_epoch={} engine_transport_transition={:?} engine_transport_block_start={:?} engine_transport_block_end={:?} engine_transport_loop_wrapped={}",
        snapshot.transport_epoch,
        snapshot.transport_transition,
        snapshot.transport_block_start_samples,
        snapshot.transport_block_end_samples,
        snapshot.transport_loop_wrapped,
    )
}

pub(crate) fn format_runtime_deferred_service_receipt_compact(
    receipt: &RuntimeDeferredServiceReceipt,
) -> String {
    format!(
        " deferred_service_class={:?} deferred_service_decision={:?} deferred_service_reason={:?} deferred_service_deferred_items={}",
        receipt.work_class,
        receipt.decision,
        receipt.reason,
        receipt.deferred_work_item_count,
    )
}
