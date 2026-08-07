//! In-process backend unit tests.

use super::prelude::*;

#[test]
fn real_clap_instrument_generates_metered_realtime_and_offline_audio_from_silence() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-instrument-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let plugin_id = "com.signal.bridge-inproc-instrument";
    let library =
        compile_clap_instrument_fixture(&directory, plugin_id, "Signal Bridge InProc Instrument")
            .expect("instrument fixture should compile");
    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(&library, plugin_id, 48_000, 512)
            .expect("zero-input stereo instrument should activate"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let events = RenderPluginEventBuffer {
        events: vec![
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
        ]
        .into(),
    };
    let edge = |source_stage_id| RenderEdgeSpec {
        source_stage_id,
        gain: 1.0,
        matrix: None,
    };
    let spec = RenderPlanSpec {
        sample_rate_hz: 48_000,
        master_gain: 1.0,
        master_limiter: None,
        stages: vec![
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                stage_id: 1,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Source { clips: Vec::new() },
                inputs: Vec::new(),
                processor: None,
                events: None,
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                stage_id: 2,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Sum,
                inputs: vec![edge(1)],
                processor: Some(handle),
                events: Some(events),
            },
            RenderStageSpec {
                parameter_envelopes: Vec::new(),
                accepts_live_events: false,
                stage_id: 3,
                format: ChannelFormat::stereo(),
                gain: 1.0,
                gain_automation: None,
                kind: RenderStageKind::Output,
                inputs: vec![edge(2)],
                processor: None,
                events: None,
            },
        ],
    };

    let (mut controller, mut executor) = render_plane();
    controller
        .install_plan(&spec)
        .expect("install instrument plan");
    controller.set_playing(true).expect("start transport");
    let mut realtime = vec![0.0f32; 512 * 2];
    executor.render_block(&mut realtime);
    assert!(realtime[..64 * 2].iter().all(|sample| *sample == 0.0));
    assert!(realtime[96 * 2..256 * 2].iter().all(|sample| *sample > 0.0));
    assert!(realtime[320 * 2..].iter().all(|sample| *sample == 0.0));
    assert!(controller
        .meters()
        .iter()
        .any(|(stage_id, peak, _)| *stage_id == 2 && *peak > 0.0));

    let offline = render_plan_to_pcm(
        &spec,
        &OfflineRenderOptions {
            start_frame: 0,
            frame_count: 512,
            block_frames: 128,
            capture_stage_ids: Vec::new(),
        },
    )
    .expect("offline instrument render");
    assert!(offline.master[..64 * 2].iter().all(|sample| *sample == 0.0));
    assert!(offline.master[96 * 2..256 * 2]
        .iter()
        .all(|sample| *sample > 0.0));
    assert!(offline.master[320 * 2..]
        .iter()
        .all(|sample| *sample == 0.0));

    // Starting transport inside the held note chases a note-on at the
    // destination, then the original note-off lands 120 frames later.
    let (mut seek_controller, mut seek_executor) = render_plane();
    seek_controller
        .install_plan(&spec)
        .expect("install seek plan");
    seek_controller.seek(200).expect("seek into held note");
    seek_controller.set_playing(true).expect("play from seek");
    let mut sought = vec![0.0f32; 512 * 2];
    seek_executor.render_block(&mut sought);
    assert!(sought[32 * 2..100 * 2].iter().all(|sample| *sample > 0.0));
    assert!(sought[120 * 2..].iter().all(|sample| *sample == 0.0));

    // A note crossing the loop end is explicitly released at the wrap,
    // then its event at frame 64 retriggers in the wrapped segment.
    let mut loop_spec = spec.clone();
    loop_spec.stages[1].events = Some(RenderPluginEventBuffer {
        events: vec![
            RenderPluginEvent {
                frame: 64,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 0.5,
                },
            },
            RenderPluginEvent {
                frame: 500,
                channel: 0,
                kind: RenderPluginEventKind::NoteOff { key: 60 },
            },
        ]
        .into(),
    });
    let (mut loop_controller, mut loop_executor) = render_plane();
    loop_controller
        .install_plan(&loop_spec)
        .expect("install loop plan");
    loop_controller
        .set_loop_region(Some((0, 384)))
        .expect("set loop");
    loop_controller.set_playing(true).expect("play loop");
    let mut looped = vec![0.0f32; 512 * 2];
    loop_executor.render_block(&mut looped);
    assert!(looped[96 * 2..256 * 2].iter().all(|sample| *sample > 0.0));
    assert!(looped[384 * 2..448 * 2].iter().all(|sample| *sample == 0.0));
    assert!(looped[480 * 2..].iter().all(|sample| *sample > 0.0));
    assert_eq!(backend.miss_count(), 0);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn real_clap_instrument_parameter_scales_held_note_independently() {
    use signal_plugin_clap::fixture::CLAP_FIXTURE_GAIN_PARAM_ID;

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-instrument-param-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let plugin_id = "com.signal.bridge-inproc-instrument-param";
    let library =
        compile_clap_instrument_fixture(&directory, plugin_id, "Signal Bridge Instrument Param")
            .expect("instrument fixture should compile");
    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(&library, plugin_id, 48_000, 128)
            .expect("instrument should activate"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let note_on = [RenderBlockPluginEvent {
        offset_frames: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn {
            key: 60,
            velocity: 0.8,
        },
    }];

    backend
        .set_parameter_normalized(CLAP_FIXTURE_GAIN_PARAM_ID, 0.25)
        .expect("set Gain");
    let queued_state = backend
        .save_state()
        .expect("capture queued parameter without an audio block");
    assert_eq!(
        f32::from_le_bytes(queued_state[0..4].try_into().expect("Gain bytes")),
        0.25,
        "state capture must flush queued CLAP parameter writes",
    );
    let mut quarter = vec![0.0f32; 128 * 2];
    assert!(handle.process_with_events(&mut quarter, 128, 2, &note_on));
    assert!(quarter.iter().all(|sample| (*sample - 0.2).abs() < 1e-6));
    let saved = backend.save_state().expect("capture instrument state");
    assert_eq!(saved.len(), 8, "fixture stores Gain + held-note level");

    backend
        .set_parameter_normalized(CLAP_FIXTURE_GAIN_PARAM_ID, 0.5)
        .expect("change Gain while note is held");
    let mut half = vec![0.0f32; 128 * 2];
    assert!(handle.process(&mut half, 128, 2));
    assert!(half.iter().all(|sample| (*sample - 0.4).abs() < 1e-6));

    backend
        .load_state(&saved)
        .expect("restore instrument state");
    let mut recalled = vec![0.0f32; 128 * 2];
    assert!(handle.process(&mut recalled, 128, 2));
    assert!(recalled.iter().all(|sample| (*sample - 0.2).abs() < 1e-6));

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn dead_clap_instrument_bypasses_silence_and_replacement_recovers() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-instrument-restart-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let plugin_id = "com.signal.bridge-inproc-instrument-restart";
    let library = compile_clap_instrument_fixture(
        &directory,
        plugin_id,
        "Signal Bridge InProc Instrument Restart",
    )
    .expect("instrument fixture should compile");
    let load = || {
        Arc::new(
            InProcessClapProcessor::load_and_activate(&library, plugin_id, 48_000, 128)
                .expect("instrument should activate"),
        )
    };
    let event = [RenderBlockPluginEvent {
        offset_frames: 0,
        channel: 0,
        kind: RenderPluginEventKind::NoteOn {
            key: 60,
            velocity: 0.5,
        },
    }];

    let backend = load();
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut live = vec![0.0f32; 128 * 2];
    assert!(handle.process_with_events(&mut live, 128, 2, &event));
    assert!(live.iter().all(|sample| *sample == 0.5));

    backend.shutdown();
    let mut fallback = vec![0.0f32; 128 * 2];
    assert!(!handle.process_with_events(&mut fallback, 128, 2, &event));
    assert!(fallback.iter().all(|sample| *sample == 0.0));
    assert_eq!(backend.miss_count(), 1);

    let replacement = load();
    let replacement_handle = RenderPluginProcessor::new(Arc::clone(&replacement) as Arc<_>);
    let mut recovered = vec![0.0f32; 128 * 2];
    assert!(replacement_handle.process_with_events(&mut recovered, 128, 2, &event));
    assert!(recovered.iter().all(|sample| *sample == 0.5));
    assert_eq!(replacement.miss_count(), 0);

    drop(replacement_handle);
    drop(replacement);
    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}
