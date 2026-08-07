use super::support::*;
use super::*;

#[test]
fn compile_rejects_live_event_flag_without_processor_and_gain_fast_path_treats_it_structural() {
    let (mut controller, _executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut spec = live_instrument_spec(handle);
    spec.stages[1].processor = None;
    let error = controller.install_plan(&spec).unwrap_err();
    assert!(
        error
            .message
            .contains("accepts live events without a plugin processor"),
        "{error}",
    );

    // Non-Sum stages cannot accept live events either (no processor is
    // even representable there).
    let mut lane_flagged = tone_spec(440.0);
    lane_flagged.stages[0].accepts_live_events = true;
    assert!(controller.install_plan(&lane_flagged).is_err());

    // Flipping the flag is a structural change, never a gain fast path.
    let with_flag = tone_spec(440.0);
    let mut without_flag = with_flag.clone();
    without_flag.stages[0].accepts_live_events = false;
    let mut flagged = with_flag.clone();
    flagged.stages[0].accepts_live_events = true;
    assert_eq!(without_flag.differs_only_in_gains(&flagged), None);
}

#[test]
fn push_live_events_validates_stage_identity_and_flag() {
    let (mut controller, _executor) = render_plane();
    let events = [live_note_on(0, 60, 0.5)];
    assert!(
        controller
            .push_live_events(LIVE_INSERT_ID, &events)
            .is_err(),
        "push without an installed plan must error",
    );

    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&live_instrument_spec(handle))
        .unwrap();
    assert!(
        controller.push_live_events(9_999, &events).is_err(),
        "unknown stage must error",
    );
    assert!(
        controller.push_live_events(LANE_ID, &events).is_err(),
        "stage without accepts_live_events must error",
    );
    controller
        .push_live_events(LIVE_INSERT_ID, &events)
        .expect("accepting stage takes the push");
    controller
        .push_live_events(LIVE_INSERT_ID, &[])
        .expect("empty push is a no-op");
}

#[test]
fn live_events_sound_through_a_hosted_instrument_while_transport_is_stopped() {
    let (mut controller, mut executor) = render_plane();
    let backend = Arc::new(EventInstrumentProcessor {
        amplitude_bits: AtomicU32::new(0.0f32.to_bits()),
    });
    let handle = RenderPluginProcessor::new(backend as Arc<_>);
    controller
        .install_plan(&live_instrument_spec(handle))
        .unwrap();

    // Stopped, posture off: the render gate silences everything.
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Posture on, still stopped: a pushed note sounds.
    controller.set_live_render(true).unwrap();
    controller
        .push_live_events(LIVE_INSERT_ID, &[live_note_on(0, 60, 0.5)])
        .unwrap();
    executor.render_block(&mut frames); // Edge envelope ramps in.
    assert!(controller.live_render());
    executor.render_block(&mut frames);
    assert!(
        frames.iter().all(|sample| (*sample - 0.5).abs() < 1e-3),
        "held live note renders at its velocity while stopped",
    );
    // Meters publish as normal under the posture.
    assert!(
        controller
            .meters()
            .iter()
            .any(|(id, peak, _)| *id == LIVE_INSERT_ID && *peak > 0.4),
        "instrument stage meters while stopped: {:?}",
        controller.meters(),
    );
    // The transport position never advanced.
    assert_eq!(controller.position_frames(), 0);

    // A note-off with a stale (past) frame clamps to "now" and stops
    // the voice.
    controller
        .push_live_events(LIVE_INSERT_ID, &[live_note_off(0, 60)])
        .unwrap();
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Posture off again: back to the silent early return.
    controller.set_live_render(false).unwrap();
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert!(!controller.live_render());
    assert_eq!(controller.position_frames(), 0);
}

#[test]
fn live_and_compiled_events_merge_ordered_by_offset_while_playing() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![
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
    let mut spec = events_spec(handle, buffer);
    spec.stages[1].accepts_live_events = true;
    controller.install_plan(&spec).unwrap();
    controller.set_playing(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames); // Block 1: frames 0..512.

    // Before block 2 (frames 512..1024): one live event already in the
    // past (clamps to offset 0) and one inside the block (offset 88).
    controller
        .push_live_events(
            LIVE_INSERT_ID,
            &[live_note_on(200, 1, 0.9), live_note_on(600, 2, 0.8)],
        )
        .unwrap();
    executor.render_block(&mut frames);

    let calls = backend.calls();
    assert_eq!(calls.len(), 2);
    let offsets: Vec<u32> = calls[1].iter().map(|event| event.offset_frames).collect();
    assert_eq!(
        offsets,
        vec![0, 7, 88, 188],
        "live + compiled events interleave sorted by in-block offset",
    );
    assert_eq!(
        calls[1][0].kind,
        RenderPluginEventKind::NoteOn {
            key: 1,
            velocity: 0.9,
        },
        "past live event clamps to offset 0",
    );
    assert_eq!(
        calls[1][2].kind,
        RenderPluginEventKind::NoteOn {
            key: 2,
            velocity: 0.8,
        },
    );
}

#[test]
fn live_event_ring_overflow_drops_and_counts() {
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    controller
        .install_plan(&live_instrument_spec(handle))
        .unwrap();
    controller.set_live_render(true).unwrap();

    let flood: Vec<RenderPluginEvent> = (0..(LIVE_EVENT_RING_CAPACITY as u64 + 32))
        .map(|index| live_note_on(index, (index % 128) as u8, 0.5))
        .collect();
    controller.push_live_events(LIVE_INSERT_ID, &flood).unwrap();
    assert_eq!(controller.live_event_drop_count(), 0);

    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert_eq!(
        controller.live_event_drop_count(),
        32,
        "events past the ring capacity drop and count",
    );
    let calls = backend.calls();
    assert_eq!(
        calls.last().unwrap().len(),
        LIVE_EVENT_RING_CAPACITY,
        "the ring's worth of events delivers this block",
    );

    // The ring drained: the next block has no pending live events, so
    // the stage takes the plain (event-less) processing path — the
    // recording backend marks that with its sentinel entry.
    executor.render_block(&mut frames);
    assert_eq!(
        backend.calls().last().unwrap().as_slice(),
        &[RenderBlockPluginEvent {
            offset_frames: u32::MAX,
            channel: 0,
            kind: RenderPluginEventKind::NoteOff { key: 0 },
        }],
    );
    assert_eq!(controller.live_event_drop_count(), 32);
}

#[test]
fn live_input_monitoring_passes_while_stopped_under_live_render() {
    let (mut controller, mut executor) = render_plane();
    let (feeder, handle) = render_live_input(LIVE_INPUT_DEFAULT_CAPACITY_FRAMES);
    controller
        .install_plan(&live_input_spec(&handle, 1.0))
        .unwrap();

    // Stopped, posture off (the g11.010 limit): silence.
    let mut base = push_ramp(&feeder, 0, 256);
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));

    // Posture on, still stopped: the input monitors through the chain.
    controller.set_live_render(true).unwrap();
    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames); // Edge envelope ramps in.
    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames);
    assert!(
        frames.iter().any(|sample| sample.abs() > 0.01),
        "monitored input must be audible while stopped",
    );
    assert_eq!(controller.position_frames(), 0);

    // Posture off: one block rides the edge ramp-out (declick), then
    // the render gate silences and the position still never moved.
    controller.set_live_render(false).unwrap();
    base = push_ramp(&feeder, base, 256);
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(frames.iter().all(|sample| *sample == 0.0));
    assert_eq!(controller.position_frames(), 0);
    let _ = base;
}

#[test]
fn compiled_events_and_timeline_clips_stay_gated_while_stopped_under_live_render() {
    // Compiled plugin events must not fire while stopped (frozen
    // position would re-trigger them every block).
    let (mut controller, mut executor) = render_plane();
    let backend = RecordingEventProcessor::new();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let buffer = event_buffer(vec![RenderPluginEvent {
        frame: 100,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn {
            key: 64,
            velocity: 0.75,
        },
    }]);
    controller
        .install_plan(&events_spec(handle, buffer))
        .unwrap();
    controller.set_live_render(true).unwrap();

    let mut frames = vec![0.0f32; 1024];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    let calls = backend.calls();
    assert_eq!(calls.len(), 2, "the stage renders while stopped");
    assert!(
        calls.iter().all(|events| events.is_empty()),
        "compiled events stay playing-gated: {calls:?}",
    );
    assert_eq!(controller.position_frames(), 0);

    // Rolling delivers the compiled stream from the held position.
    controller.set_playing(true).unwrap();
    executor.render_block(&mut frames);
    let calls = backend.calls();
    assert_eq!(calls[2].len(), 1);
    assert_eq!(calls[2][0].offset_frames, 100);

    // Timeline clip content is silent while stopped under the posture.
    let (mut controller, mut executor) = render_plane();
    controller.install_plan(&tone_spec(440.0)).unwrap();
    controller.set_live_render(true).unwrap();
    let mut frames = [0.0f32; 512];
    executor.render_block(&mut frames);
    executor.render_block(&mut frames);
    assert!(
        frames.iter().all(|sample| *sample == 0.0),
        "a stopped transport must not replay frozen clip content",
    );
    controller.set_playing(true).unwrap();
    executor.render_block(&mut frames);
    assert!(frames.iter().any(|sample| sample.abs() > 0.01));
}
