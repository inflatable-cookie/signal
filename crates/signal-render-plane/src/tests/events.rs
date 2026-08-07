use super::support::*;
use super::*;

#[test]
fn processor_stage_delivers_events_at_intra_block_sample_offsets() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
        RenderPluginEvent {
            frame: 100,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 64,
                velocity: 0.75,
            },
        },
        RenderPluginEvent {
            frame: 519,
            channel: 0,
            kind: RenderPluginEventKind::ControlChange {
                controller: 74,
                value: 0.33,
            },
        },
        RenderPluginEvent {
            frame: 700,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 64 },
        },
    ]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_playing(true).unwrap();

    // Two 512-frame blocks from position 0.
    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);

    let calls = backend.calls();
    assert_eq!(calls.len(), 2, "one delivery per rendered block");
    assert_eq!(
        calls[0],
        vec![RenderBlockPluginEvent {
            offset_frames: 100,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 64,
                velocity: 0.75,
            },
        }],
        "block 1 carries the note-on at its absolute frame",
    );
    assert_eq!(
        calls[1],
        vec![
            RenderBlockPluginEvent {
                offset_frames: 7,
                channel: 0,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 74,
                    value: 0.33,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 188,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 64 },
            },
        ],
        "block 2 events land at frame − block start",
    );
}

#[test]
fn hosted_instrument_events_generate_audio_from_a_silent_lane() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(EventInstrumentProcessor {
        amplitude_bits: AtomicU32::new(0.0f32.to_bits()),
    });
    let handle = RenderPluginProcessor::new(backend as Arc<_>);
    let events = event_buffer(vec![
        RenderPluginEvent {
            frame: 64,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.5,
            },
        },
        RenderPluginEvent {
            frame: 320,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 60 },
        },
    ]);
    let mut spec = events_spec(handle, events);
    let RenderStageKind::Source { clips } = &mut spec.stages[0].kind else {
        panic!("fixture lane source");
    };
    let RenderSource::Samples(samples) = &mut clips[0].source else {
        panic!("fixture sample source");
    };
    samples.frames = vec![0.0; samples.frames.len()].into();
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 512 * 2];
    executor.render_block(&mut frames);
    assert!(frames[..64 * 2].iter().all(|sample| *sample == 0.0));
    assert!(frames[96 * 2..256 * 2].iter().all(|sample| *sample > 0.0));
    assert!(frames[320 * 2..].iter().all(|sample| *sample == 0.0));
    assert!(controller.meters().iter().any(|(_, peak, _)| *peak > 0.0));

    let offline = crate::offline::render_plan_to_pcm(
        &spec,
        &crate::offline::OfflineRenderOptions {
            start_frame: 0,
            frame_count: 512,
            block_frames: 128,
            capture_stage_ids: Vec::new(),
        },
    )
    .expect("offline hosted instrument render");
    assert!(offline.master[..64 * 2].iter().all(|sample| *sample == 0.0));
    assert!(offline.master[96 * 2..256 * 2]
        .iter()
        .all(|sample| *sample > 0.0));
    assert!(offline.master[320 * 2..]
        .iter()
        .all(|sample| *sample == 0.0));
}

#[test]
fn event_delivery_is_playback_gated() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn {
            key: 60,
            velocity: 1.0,
        },
    }]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_playing(true).unwrap();
    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    // Stop: the edge ramp keeps rendering blocks briefly, but the
    // position no longer advances — re-delivering the same events would
    // double-trigger notes, so delivery gates on playback.
    controller.set_playing(false).unwrap();
    executor.render_block(&mut frames);

    let calls = backend.calls();
    assert!(calls.len() >= 2, "ramp-out still processes audio");
    assert_eq!(calls[0].len(), 1, "playing block delivers");
    for call in &calls[1..] {
        assert!(call.is_empty(), "stopped blocks must deliver no events");
    }
}

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

#[test]
fn compile_rejects_events_without_processor_and_unsorted_events() {
    let buffer = event_buffer(vec![RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key: 0 },
    }]);
    let mut spec = processor_spec(None);
    spec.stages[1].events = Some(buffer);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(
        error.message.contains("without a plugin processor"),
        "{error}"
    );

    let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
    let unsorted = event_buffer(vec![
        RenderPluginEvent {
            frame: 10,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        },
        RenderPluginEvent {
            frame: 5,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        },
    ]);
    let spec = events_spec(handle, unsorted);
    let (mut controller, _executor) = render_plane();
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(error.message.contains("not sorted by frame"), "{error}");
}

#[test]
fn event_buffer_swap_is_structural_not_a_gain_fast_path() {
    let handle = RenderPluginProcessor::new(RecordingEventProcessor::new() as Arc<_>);
    let event = RenderPluginEvent {
        frame: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOff { key: 0 },
    };
    let with_a = events_spec(handle, event_buffer(vec![event]));
    // Clone shares the Arc: gain-only diff logic sees no change.
    let with_a_again = with_a.clone();
    assert_eq!(with_a.differs_only_in_gains(&with_a_again), Some(vec![]));
    // A rebuilt buffer (same content, new Arc) is structural.
    let mut with_b = with_a.clone();
    with_b.stages[1].events = Some(event_buffer(vec![event]));
    assert_eq!(with_a.differs_only_in_gains(&with_b), None);
}
