use super::*;

#[test]
fn event_packet_summary_counts_richer_event_types() {
    let packet = EventPacket::new(vec![
        PluginEvent::ParameterValue(ParameterValueEvent {
            offset_frames: 0,
            parameter_id: 3,
            normalized_value: 0.1,
        }),
        PluginEvent::ParameterGesture(ParameterGestureEvent {
            offset_frames: 4,
            parameter_id: 3,
            phase: ParameterGesturePhase::Begin,
        }),
        PluginEvent::ParameterModulation(ParameterModulationEvent {
            offset_frames: 8,
            parameter_id: 9,
            amount: -0.2,
        }),
        PluginEvent::Note(NoteEvent {
            offset_frames: 16,
            note_id: 7,
            port_index: 0,
            channel: 0,
            key: 60,
            velocity: 0.8,
            kind: NoteEventKind::NoteOn,
        }),
        PluginEvent::NoteExpression(NoteExpressionEvent {
            offset_frames: 24,
            note_id: 7,
            port_index: 0,
            channel: 0,
            key: 60,
            expression: NoteExpressionKind::Pressure,
            value: 0.7,
        }),
        PluginEvent::NoteExpression(NoteExpressionEvent {
            offset_frames: 28,
            note_id: 7,
            port_index: 0,
            channel: 1,
            key: 61,
            expression: NoteExpressionKind::Timbre,
            value: 0.5,
        }),
        PluginEvent::NoteExpression(NoteExpressionEvent {
            offset_frames: 30,
            note_id: 7,
            port_index: 0,
            channel: 2,
            key: 62,
            expression: NoteExpressionKind::Tuning,
            value: 0.2,
        }),
        PluginEvent::Midi(MidiEvent {
            offset_frames: 32,
            status: 0xB0,
            data1: 1,
            data2: 100,
        }),
    ]);

    let summary = packet.summary();
    assert_eq!(summary.total_events, 8);
    assert_eq!(summary.parameter_value_events, 1);
    assert_eq!(summary.parameter_gesture_events, 1);
    assert_eq!(summary.parameter_modulation_events, 1);
    assert_eq!(summary.note_events, 1);
    assert_eq!(summary.note_expression_events, 3);
    assert_eq!(summary.note_expression_pressure_events, 1);
    assert_eq!(summary.note_expression_timbre_events, 1);
    assert_eq!(summary.note_expression_tuning_events, 1);
    assert_eq!(summary.midi_events, 1);
}

#[test]
fn parameter_automation_summary_tracks_values_modulation_and_gestures() {
    let packet = EventPacket::new(vec![
        PluginEvent::ParameterGesture(ParameterGestureEvent {
            offset_frames: 0,
            parameter_id: 77,
            phase: ParameterGesturePhase::Begin,
        }),
        PluginEvent::ParameterValue(ParameterValueEvent {
            offset_frames: 4,
            parameter_id: 77,
            normalized_value: 0.2,
        }),
        PluginEvent::ParameterModulation(ParameterModulationEvent {
            offset_frames: 8,
            parameter_id: 77,
            amount: -0.1,
        }),
        PluginEvent::ParameterValue(ParameterValueEvent {
            offset_frames: 16,
            parameter_id: 77,
            normalized_value: 0.6,
        }),
        PluginEvent::ParameterGesture(ParameterGestureEvent {
            offset_frames: 20,
            parameter_id: 77,
            phase: ParameterGesturePhase::End,
        }),
        PluginEvent::ParameterValue(ParameterValueEvent {
            offset_frames: 24,
            parameter_id: 9,
            normalized_value: 0.9,
        }),
    ]);

    let summary = packet.parameter_automation_summary(77);
    assert_eq!(
        summary,
        ParameterAutomationSummary {
            parameter_id: 77,
            value_events: 2,
            modulation_events: 1,
            gesture_begin_events: 1,
            gesture_end_events: 1,
            first_value: Some(0.2),
            last_value: Some(0.6),
            last_modulation: Some(-0.1),
        }
    );
}

#[test]
fn automation_continuity_report_tracks_segments_and_lease_rollovers() {
    let mut report = AutomationContinuityReport::default();
    report.record(
        2,
        "lease-a",
        ParameterAutomationSummary {
            parameter_id: 77,
            value_events: 1,
            modulation_events: 1,
            gesture_begin_events: 1,
            gesture_end_events: 0,
            first_value: Some(0.1),
            last_value: Some(0.1),
            last_modulation: Some(0.02),
        },
    );
    report.record(
        2,
        "lease-a",
        ParameterAutomationSummary {
            parameter_id: 77,
            value_events: 1,
            modulation_events: 1,
            gesture_begin_events: 0,
            gesture_end_events: 1,
            first_value: Some(0.15),
            last_value: Some(0.15),
            last_modulation: Some(0.04),
        },
    );
    report.record(
        3,
        "lease-b",
        ParameterAutomationSummary {
            parameter_id: 77,
            value_events: 2,
            modulation_events: 2,
            gesture_begin_events: 1,
            gesture_end_events: 1,
            first_value: Some(0.2),
            last_value: Some(0.25),
            last_modulation: Some(0.06),
        },
    );

    assert_eq!(report.parameter_id, 77);
    assert_eq!(report.segment_count(), 2);
    assert_eq!(report.lease_rollovers, 1);
    assert_eq!(report.first_epoch(), Some(2));
    assert_eq!(report.last_epoch(), Some(3));
    assert_eq!(report.segment_epochs(), vec![2, 3]);

    let aggregate = report.aggregate();
    assert_eq!(aggregate.value_events, 4);
    assert_eq!(aggregate.modulation_events, 4);
    assert_eq!(aggregate.gesture_begin_events, 2);
    assert_eq!(aggregate.gesture_end_events, 2);
    assert_eq!(aggregate.first_value, Some(0.1));
    assert_eq!(aggregate.last_value, Some(0.25));
    assert_eq!(aggregate.last_modulation, Some(0.06));
}

#[test]
fn block_sequence_continuity_report_tracks_rollovers_and_gaps() {
    let mut report = BlockSequenceContinuityReport::default();
    report.record(2, "lease-a", 0);
    report.record(2, "lease-a", 1);
    report.record(2, "lease-a", 3);
    report.record(3, "lease-b", 4);
    report.record(3, "lease-b", 5);

    assert_eq!(report.segment_count(), 3);
    assert_eq!(report.segment_epochs(), vec![2, 2, 3]);
    assert_eq!(report.first_block_sequence(), Some(0));
    assert_eq!(report.last_block_sequence(), Some(5));
    assert_eq!(report.sequence_gaps, 1);
    assert_eq!(report.lease_rollovers, 1);
}
