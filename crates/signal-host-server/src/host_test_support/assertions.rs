use signal_runtime::RuntimeSupervisorReport;

pub(crate) fn assert_runtime_automation_values(
    supervisor: &RuntimeSupervisorReport,
    value_events: usize,
    modulation_events: usize,
    gesture_begin_events: usize,
    gesture_end_events: usize,
    first_value: f32,
    last_value: f32,
    last_modulation: f32,
) {
    let snapshot = &supervisor.observation.automation_snapshot;
    assert_eq!(snapshot.parameter_id, 4096);
    assert_eq!(snapshot.value_events, value_events);
    assert_eq!(snapshot.modulation_events, modulation_events);
    assert_eq!(snapshot.gesture_begin_events, gesture_begin_events);
    assert_eq!(snapshot.gesture_end_events, gesture_end_events);
    assert!(snapshot
        .first_value
        .is_some_and(|observed| (observed - first_value).abs() < 1.0e-6));
    assert!(snapshot
        .last_value
        .is_some_and(|observed| (observed - last_value).abs() < 1.0e-6));
    assert!(snapshot
        .last_modulation
        .is_some_and(|observed| (observed - last_modulation).abs() < 1.0e-6));
}

pub(crate) fn assert_runtime_automation_continuity(
    supervisor: &RuntimeSupervisorReport,
    first_epoch: u64,
    last_epoch: u64,
    epochs: &[u64],
    lease_rollovers: usize,
) {
    let snapshot = &supervisor.observation.automation_snapshot;
    assert_eq!(snapshot.first_epoch, Some(first_epoch));
    assert_eq!(snapshot.last_epoch, Some(last_epoch));
    assert_eq!(snapshot.segment_count, epochs.len());
    assert_eq!(snapshot.segment_epochs, epochs);
    assert_eq!(snapshot.lease_rollovers, lease_rollovers);
}

pub(crate) fn assert_runtime_plugin_event_snapshot(
    supervisor: &RuntimeSupervisorReport,
    first_epoch: u64,
    last_epoch: u64,
    epochs: &[u64],
    lease_rollovers: usize,
) {
    let snapshot = &supervisor.observation.plugin_event_snapshot;
    assert!(snapshot.total_events > 0, "{snapshot:?}");
    assert!(snapshot.note_events > 0, "{snapshot:?}");
    assert!(snapshot.note_expression_events > 0, "{snapshot:?}");
    assert!(snapshot.midi_events > 0, "{snapshot:?}");
    assert!(snapshot.last_generated_event_bytes > 0, "{snapshot:?}");
    assert_eq!(snapshot.first_epoch, Some(first_epoch));
    assert_eq!(snapshot.last_epoch, Some(last_epoch));
    assert_eq!(snapshot.segment_count, epochs.len());
    assert_eq!(snapshot.segment_epochs, epochs);
    assert_eq!(snapshot.lease_rollovers, lease_rollovers);
    assert!(snapshot.last_block_sequence.is_some(), "{snapshot:?}");
}

pub(crate) fn assert_runtime_sequence_continuity(
    supervisor: &RuntimeSupervisorReport,
    epochs: &[u64],
    first_block_sequence: u64,
    last_block_sequence: u64,
    sequence_gaps: usize,
    lease_rollovers: usize,
) {
    let timeline = &supervisor
        .observation
        .timeline_snapshot
        .block_sequence_continuity;
    assert_eq!(timeline.segment_count(), epochs.len());
    assert_eq!(timeline.segment_epochs(), epochs);
    assert_eq!(timeline.first_block_sequence(), Some(first_block_sequence));
    assert_eq!(timeline.last_block_sequence(), Some(last_block_sequence));
    assert_eq!(timeline.sequence_gaps, sequence_gaps);
    assert_eq!(timeline.lease_rollovers, lease_rollovers);
}
