use super::super::support::*;
use super::super::*;

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
