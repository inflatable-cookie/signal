use super::super::support::*;
use super::super::*;

#[test]
fn loop_wrap_delivers_both_segments_with_buffer_relative_offsets() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 100,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.5,
            },
        },
        RenderPluginEvent {
            frame: 550,
            channel: 1,
            kind: RenderPluginEventKind::ControlChange {
                controller: 1,
                value: 1.0,
            },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_loop_region(Some((0, 600))).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames); // [0, 512): note at 100
    executor.render_block(&mut frames); // [512, 600) + wrap [0, 424)

    let calls = backend.calls();
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].len(), 1);
    assert_eq!(calls[0][0].offset_frames, 100);
    assert_eq!(
        calls[1],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 38, // 550 − 512, first segment
                channel: 1,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 1,
                    value: 1.0,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 88, // wrap: release note active at loop end
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 60 },
            },
            RenderBlockPluginEvent {
                offset_frames: 188, // 88 wrap offset + frame 100
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 0.5,
                },
            },
        ],
        "wrapped block delivers both segments, buffer-relative",
    );
}

#[test]
fn loop_wrap_chases_held_note_controller_and_expression_at_wrap_offset() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 50,
            channel: 3,
            kind: RenderPluginEventKind::PitchBend { value: -0.25 },
        },
        RenderPluginEvent {
            frame: 100,
            channel: 3,
            kind: RenderPluginEventKind::NoteOn {
                key: 67,
                velocity: 0.9,
            },
        },
        RenderPluginEvent {
            frame: 150,
            channel: 3,
            kind: RenderPluginEventKind::NoteExpression {
                key: 67,
                expression: RenderNoteExpressionKind::Timbre,
                value: 0.45,
            },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_loop_region(Some((300, 600))).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames); // [0, 512)
    executor.render_block(&mut frames); // [512, 600) + wrap [300, 724)

    assert_eq!(
        backend.calls()[1],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::NoteOff { key: 67 },
            },
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::PitchBend { value: -0.25 },
            },
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::NoteOn {
                    key: 67,
                    velocity: 0.9,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 88,
                channel: 3,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 67,
                    expression: RenderNoteExpressionKind::Timbre,
                    value: 0.45,
                },
            },
        ],
    );
}
