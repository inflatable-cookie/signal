use super::super::*;

#[test]
fn clap_shared_memory_header_scales_with_channel_count() {
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-c",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 0,
        },
        1024,
    );
    let header = protocol.block_header(1, 2, 512);
    assert_eq!(header.block.channel_count, 2);
    assert!(header.layout.audio_input.size_bytes > 0);
}

#[test]
fn clap_event_translation_upgrades_note_and_modulation_semantics() {
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-translate",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        1024,
    );
    let payload = protocol.test_input_payload(3, 512);

    let clap_events = protocol.translate_input_events(&payload.events);
    assert_eq!(clap_events.events.len(), 11);
    assert!(matches!(clap_events.events[0], ClapEvent::ParamGesture(_)));
    assert!(matches!(clap_events.events[1], ClapEvent::ParamGesture(_)));
    assert!(matches!(clap_events.events[2], ClapEvent::ParamValue(_)));
    assert!(matches!(clap_events.events[3], ClapEvent::ParamValue(_)));
    assert!(matches!(
        clap_events.events[4],
        ClapEvent::ParamModulation(_)
    ));
    assert!(matches!(
        clap_events.events[5],
        ClapEvent::ParamModulation(_)
    ));
    assert!(matches!(clap_events.events[6], ClapEvent::Note(_)));
    assert!(matches!(
        clap_events.events[7],
        ClapEvent::NoteExpression(ClapNoteExpressionEvent {
            expression: ClapNoteExpressionKind::Timbre,
            ..
        })
    ));
    assert!(matches!(
        clap_events.events[8],
        ClapEvent::NoteExpression(ClapNoteExpressionEvent {
            expression: ClapNoteExpressionKind::Tuning,
            ..
        })
    ));
    assert!(matches!(
        clap_events.events[9],
        ClapEvent::NoteExpression(ClapNoteExpressionEvent {
            expression: ClapNoteExpressionKind::Pressure,
            ..
        })
    ));
    assert!(matches!(clap_events.events[10], ClapEvent::Midi(_)));
    assert!(matches!(
        clap_events.events[1],
        ClapEvent::ParamGesture(ClapParamGestureEvent {
            phase: ClapParamGesturePhase::End,
            ..
        })
    ));

    let round_tripped = protocol.translate_output_events(&clap_events);
    let summary = round_tripped.summary();
    assert_eq!(summary.parameter_value_events, 2);
    assert_eq!(summary.parameter_gesture_events, 2);
    assert_eq!(summary.parameter_modulation_events, 2);
    assert_eq!(summary.note_events, 1);
    assert_eq!(summary.note_expression_events, 3);
    assert_eq!(summary.midi_events, 1);
    let automation = round_tripped.parameter_automation_summary(protocol.automation_parameter_id());
    assert_eq!(automation.value_events, 1);
    assert_eq!(automation.modulation_events, 1);
    assert_eq!(automation.gesture_begin_events, 0);
    assert_eq!(automation.gesture_end_events, 1);
    assert_eq!(automation.first_value, Some(0.25));
    assert_eq!(automation.last_value, Some(0.25));
    assert_eq!(automation.last_modulation, Some(-0.02));
}
