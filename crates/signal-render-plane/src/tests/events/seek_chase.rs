use super::super::support::*;
use super::super::*;

#[test]
fn seek_chases_held_plugin_note_controller_and_expression_state() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 50,
            channel: 2,
            kind: RenderPluginEventKind::ControlChange {
                controller: 74,
                value: 0.25,
            },
        },
        RenderPluginEvent {
            frame: 50,
            channel: 2,
            kind: RenderPluginEventKind::PitchBend { value: 0.4 },
        },
        RenderPluginEvent {
            frame: 50,
            channel: 2,
            kind: RenderPluginEventKind::ChannelPressure { value: 0.6 },
        },
        RenderPluginEvent {
            frame: 100,
            channel: 2,
            kind: RenderPluginEventKind::NoteOn {
                key: 64,
                velocity: 0.75,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Pressure,
                value: 0.7,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Timbre,
                value: 0.8,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Tuning,
                value: 12.0,
            },
        },
        RenderPluginEvent {
            frame: 500,
            channel: 2,
            kind: RenderPluginEventKind::NoteOff { key: 64 },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.seek(300).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    assert_eq!(
        backend.calls()[0],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 74,
                    value: 0.25,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::PitchBend { value: 0.4 },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::ChannelPressure { value: 0.6 },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteOn {
                    key: 64,
                    velocity: 0.75,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Pressure,
                    value: 0.7,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Timbre,
                    value: 0.8,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 2,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 64,
                    expression: RenderNoteExpressionKind::Tuning,
                    value: 12.0,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 200,
                channel: 2,
                kind: RenderPluginEventKind::NoteOff { key: 64 },
            },
        ],
    );
}
