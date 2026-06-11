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
