//! In-process backend unit tests.

use super::common::{
    convert_block_events, AU_EVENT_SUPPORT, CLAP_EVENT_SUPPORT, EVENT_SCRATCH_CAPACITY,
};
use super::*;
use signal_plugin::{MidiEvent, NoteExpressionEvent, NoteExpressionKind, PluginEvent};
use signal_plugin_vst3::VST3_RESTART_IO_CHANGED;
use signal_render_plane::RenderPluginEventSupport;
use std::sync::atomic::Ordering;

use signal_plugin_clap::fixture::{
    compile_clap_fixture, compile_clap_instrument_fixture, rustc_available, CLAP_FIXTURE_GAIN,
};
use signal_plugin_vst3::fixture::{
    compile_vst3_fixture, VST3_FIXTURE_CLASS_ID_HEX, VST3_FIXTURE_GAIN,
};
use signal_render_plane::{
    render_plan_to_pcm, render_plane, ChannelFormat, OfflineRenderOptions, RenderBlockPluginEvent,
    RenderEdgeSpec, RenderNoteExpressionKind, RenderPlanSpec, RenderPluginEvent,
    RenderPluginEventBuffer, RenderPluginEventKind, RenderPluginProcessor, RenderStageKind,
    RenderStageSpec,
};

#[test]
fn expressive_render_events_retain_values_at_the_neutral_boundary() {
    let events = [
        RenderBlockPluginEvent {
            offset_frames: 3,
            channel: 2,
            kind: RenderPluginEventKind::PitchBend { value: 0.0 },
        },
        RenderBlockPluginEvent {
            offset_frames: 5,
            channel: 2,
            kind: RenderPluginEventKind::ChannelPressure { value: 0.5 },
        },
        RenderBlockPluginEvent {
            offset_frames: 7,
            channel: 2,
            kind: RenderPluginEventKind::NoteExpression {
                key: 64,
                expression: RenderNoteExpressionKind::Tuning,
                value: 37.5,
            },
        },
    ];
    let mut scratch = Vec::with_capacity(EVENT_SCRATCH_CAPACITY);
    convert_block_events(&events, &mut scratch);

    assert_eq!(
        scratch[0],
        PluginEvent::Midi(MidiEvent {
            offset_frames: 3,
            status: 0xE2,
            data1: 0,
            data2: 64,
        })
    );
    assert_eq!(
        scratch[1],
        PluginEvent::Midi(MidiEvent {
            offset_frames: 5,
            status: 0xD2,
            data1: 64,
            data2: 0,
        })
    );
    assert_eq!(
        scratch[2],
        PluginEvent::NoteExpression(NoteExpressionEvent {
            offset_frames: 7,
            note_id: -1,
            port_index: 0,
            channel: 2,
            key: 64,
            expression: NoteExpressionKind::Tuning,
            value: 37.5,
        })
    );
}
use std::sync::Arc;

#[test]
fn in_process_backend_loads_and_processes_the_fixture() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let library = compile_clap_fixture(
        &directory,
        "com.signal.bridge-inproc",
        "Signal Bridge InProc",
        0,
    )
    .expect("fixture should compile");

    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(
            &library,
            "com.signal.bridge-inproc",
            48_000,
            256,
        )
        .expect("backend should load and activate"),
    );
    assert_eq!(backend.parameters().len(), 2);
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let reference = scratch.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7,
            "sample {index}: {output} vs {input} * {CLAP_FIXTURE_GAIN}",
        );
    }
    assert_eq!(backend.miss_count(), 0);

    // Shutdown: later blocks bypass and leave scratch untouched.
    backend.shutdown();
    let mut scratch = reference.clone();
    assert!(!handle.process(&mut scratch, 128, 2));
    assert_eq!(scratch, reference);
    assert_eq!(backend.miss_count(), 1);

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// g12.023: the CLAP set-then-process proof — a wire param write lands
/// in the plugin's DSP via process in-events, byte-exact, at the next
/// block (block-boundary posture).
#[test]
fn in_process_clap_param_set_reaches_the_dsp_next_block() {
    use signal_plugin_clap::fixture::CLAP_FIXTURE_GAIN_PARAM_ID;

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-set-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let library = compile_clap_fixture(
        &directory,
        "com.signal.bridge-inproc-set",
        "Signal Bridge InProc Set",
        0,
    )
    .expect("fixture should compile");

    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(
            &library,
            "com.signal.bridge-inproc-set",
            48_000,
            256,
        )
        .expect("backend should load and activate"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    // Block 1: fixture default gain.
    let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (output, input) in scratch.iter().zip(reference.iter()) {
        assert!((output - input * CLAP_FIXTURE_GAIN).abs() < 1e-7);
    }

    // Set Gain (plain range 0..1, so normalized == plain) mid-stream:
    // the NEXT block applies the new value exactly.
    backend
        .set_parameter_normalized(CLAP_FIXTURE_GAIN_PARAM_ID, 0.25)
        .expect("param set queues");
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * 0.25).abs() < 1e-7,
            "sample {index}: {output} vs {input} * 0.25",
        );
    }

    // Unknown parameters and dead backends fail typed.
    assert_eq!(
        backend.set_parameter_normalized(9999, 0.5).unwrap_err(),
        "unknown_parameter",
    );
    backend.shutdown();
    assert_eq!(
        backend
            .set_parameter_normalized(CLAP_FIXTURE_GAIN_PARAM_ID, 0.5)
            .unwrap_err(),
        "backend_dead",
    );

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// g12.034 follow-up: note, CC, and native note-expression delivery
/// through the CLAP in-process backend, sample-offset accurate. The fixture turns note events and
/// MIDI CC7 into gain steps applied FROM the event's intra-block time,
/// so the output proves the decoded bytes AND the offsets.
#[test]
fn in_process_clap_note_and_cc_events_reach_the_dsp_at_their_offsets() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-events-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let library = compile_clap_fixture(
        &directory,
        "com.signal.bridge-inproc-events",
        "Signal Bridge InProc Events",
        0,
    )
    .expect("fixture should compile");

    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(
            &library,
            "com.signal.bridge-inproc-events",
            48_000,
            256,
        )
        .expect("backend should load and activate"),
    );
    let handle = signal_render_plane::RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    assert_eq!(handle.event_support(), CLAP_EVENT_SUPPORT);
    assert_eq!(handle.latency_frames(), 0);

    // Note-on velocity 0.25 at frame 64: gain steps 0.5 → 0.25 there.
    let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 64,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.25,
            },
        }],
    ));
    for frame in 0..128 {
        let expected_gain = if frame < 64 { CLAP_FIXTURE_GAIN } else { 0.25 };
        for channel in 0..2 {
            let index = frame * 2 + channel;
            assert!(
                (scratch[index] - reference[index] * expected_gain).abs() < 1e-6,
                "frame {frame}: {} vs {} * {expected_gain}",
                scratch[index],
                reference[index],
            );
        }
    }

    // CC7 value 96/127 at frame 32: the boundary downconversion
    // (f32 0..1 → round(value·127)) must land byte-exact.
    let cc_value = 96.0f32 / 127.0;
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 32,
            channel: 0,
            kind: RenderPluginEventKind::ControlChange {
                controller: 7,
                value: cc_value,
            },
        }],
    ));
    for frame in 0..128 {
        // Gain persisted from the previous block's note-on (0.25).
        let expected_gain = if frame < 32 { 0.25 } else { 96.0 / 127.0 };
        let index = frame * 2;
        assert!(
            (scratch[index] - reference[index] * expected_gain).abs() < 1e-6,
            "frame {frame}: {} vs {} * {expected_gain}",
            scratch[index],
            reference[index],
        );
    }

    // Per-note tuning is stored as cents in Pulse/Signal and converted
    // to CLAP semitones at the native event boundary: 37.5c → 0.375.
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 48,
            channel: 0,
            kind: RenderPluginEventKind::NoteExpression {
                key: 60,
                expression: RenderNoteExpressionKind::Tuning,
                value: 37.5,
            },
        }],
    ));
    for frame in 0..128 {
        let expected_gain = if frame < 48 { 96.0 / 127.0 } else { 0.375 };
        let index = frame * 2;
        assert!(
            (scratch[index] - reference[index] * expected_gain).abs() < 1e-6,
            "frame {frame}: {} vs {} * {expected_gain}",
            scratch[index],
            reference[index],
        );
    }
    assert_eq!(backend.miss_count(), 0);
    assert_eq!(handle.unsupported_event_count(), 0);

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

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

/// g12.034 follow-up, VST3: notes ride the input IEventList; CC 7 maps
/// through the fixture's IMidiMapping onto the Gain param and rides
/// IParameterChanges with the event's sample offset — the value stays
/// 32-bit float end to end (no 7-bit quantization on this path).
#[test]
fn in_process_vst3_note_and_cc_events_reach_the_dsp_at_their_offsets() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-vst3-events-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let bundle = compile_vst3_fixture(
        &directory,
        "plugin:vst3:bridge-inproc-events",
        "Signal Bridge InProc VST3 Events",
    )
    .expect("vst3 fixture should compile");

    let backend = Arc::new(
        InProcessVst3Processor::load_and_activate(&bundle, VST3_FIXTURE_CLASS_ID_HEX, 48_000, 256)
            .expect("backend should load and activate"),
    );
    assert!(
        backend.midi_cc_mapping_available(),
        "fixture exposes IMidiMapping",
    );
    let handle = signal_render_plane::RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    assert_eq!(
        handle.event_support(),
        RenderPluginEventSupport {
            notes: true,
            control_change: true,
            pitch_bend: true,
            channel_pressure: true,
            note_expression: false,
        }
    );
    assert_eq!(handle.latency_frames(), 0);

    // Note-on velocity 0.25 at frame 64 (input IEventList).
    let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 64,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 0.25,
            },
        }],
    ));
    for frame in 0..128 {
        let expected_gain = if frame < 64 { VST3_FIXTURE_GAIN } else { 0.25 };
        let index = frame * 2;
        assert!(
            (scratch[index] - reference[index] * expected_gain).abs() < 1e-6,
            "frame {frame}: {} vs {} * {expected_gain}",
            scratch[index],
            reference[index],
        );
    }

    // CC7 0.8 at frame 32 → IMidiMapping → Gain param point at offset
    // 32 carrying the full float value.
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 32,
            channel: 0,
            kind: RenderPluginEventKind::ControlChange {
                controller: signal_plugin_vst3::fixture::VST3_FIXTURE_GAIN_CC,
                value: 0.8,
            },
        }],
    ));
    for frame in 0..128 {
        let expected_gain = if frame < 32 { 0.25 } else { 0.8 };
        let index = frame * 2;
        assert!(
            (scratch[index] - reference[index] * expected_gain).abs() < 1e-6,
            "frame {frame}: {} vs {} * {expected_gain}",
            scratch[index],
            reference[index],
        );
    }

    // Pitch bend centre maps through VST3 controller 128 as the exact
    // 14-bit MIDI centre value, 8192/16383.
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 24,
            channel: 0,
            kind: RenderPluginEventKind::PitchBend { value: 0.0 },
        }],
    ));
    for frame in 0..128 {
        let expected_gain = if frame < 24 { 0.8 } else { 8192.0 / 16_383.0 };
        let index = frame * 2;
        assert!(
            (scratch[index] - reference[index] * expected_gain).abs() < 1e-6,
            "frame {frame}: {} vs {} * {expected_gain}",
            scratch[index],
            reference[index],
        );
    }

    // Channel pressure maps through controller 129 with 7-bit MIDI
    // quantization at the bridge boundary.
    let pressure = 32.0f32 / 127.0;
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 40,
            channel: 0,
            kind: RenderPluginEventKind::ChannelPressure { value: pressure },
        }],
    ));
    for frame in 0..128 {
        let expected_gain = if frame < 40 {
            8192.0 / 16_383.0
        } else {
            pressure
        };
        let index = frame * 2;
        assert!(
            (scratch[index] - reference[index] * expected_gain).abs() < 1e-6,
            "frame {frame}: {} vs {} * {expected_gain}",
            scratch[index],
            reference[index],
        );
    }

    // The fixture has no CC74 assignment and the VST3 adapter has no
    // per-note expression path. Both attempts stay observable.
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[
            RenderBlockPluginEvent {
                offset_frames: 8,
                channel: 0,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 74,
                    value: 0.25,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 16,
                channel: 0,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 60,
                    expression: RenderNoteExpressionKind::Pressure,
                    value: 0.5,
                },
            },
        ],
    ));
    assert_eq!(handle.unsupported_event_count(), 2);
    assert_eq!(backend.miss_count(), 0);

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// g12.023: the VST3 mirror — the write rides the block's input
/// `IParameterChanges` and the fixture's processor applies it.
#[test]
fn in_process_vst3_param_set_reaches_the_dsp_next_block() {
    use signal_plugin_vst3::fixture::VST3_FIXTURE_GAIN_PARAM_ID;

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-vst3-set-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let bundle = compile_vst3_fixture(
        &directory,
        "plugin:vst3:bridge-inproc-set",
        "Signal Bridge InProc VST3 Set",
    )
    .expect("vst3 fixture should compile");

    let backend = Arc::new(
        InProcessVst3Processor::load_and_activate(&bundle, VST3_FIXTURE_CLASS_ID_HEX, 48_000, 256)
            .expect("backend should load and activate"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (output, input) in scratch.iter().zip(reference.iter()) {
        assert!((output - input * VST3_FIXTURE_GAIN).abs() < 1e-7);
    }

    backend
        .set_parameter_normalized(VST3_FIXTURE_GAIN_PARAM_ID, 0.75)
        .expect("param set queues");
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * 0.75).abs() < 1e-7,
            "sample {index}: {output} vs {input} * 0.75",
        );
    }
    assert_eq!(
        backend.set_parameter_normalized(9999, 0.5).unwrap_err(),
        "unknown_parameter",
    );

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// g12.023: the LV2 mirror — the write lands in the connected Gain
/// control slot before the next `run()`.
#[test]
fn in_process_lv2_param_set_reaches_the_dsp_next_block() {
    use signal_plugin_lv2::fixture::{
        compile_lv2_fixture, rustc_available as lv2_rustc_available, LV2_FIXTURE_GAIN,
        LV2_FIXTURE_GAIN_PORT_INDEX,
    };
    if !lv2_rustc_available() {
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-lv2-set-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let plugin_uri = "https://signal.dev/fixtures/lv2/bridge-inproc-set";
    let bundle = compile_lv2_fixture(&directory, plugin_uri, "Signal Bridge InProc LV2 Set")
        .expect("lv2 fixture should compile");

    let backend = Arc::new(
        InProcessLv2Processor::load_and_activate(&bundle, plugin_uri, 48_000, 256)
            .expect("backend should load and activate"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (output, input) in scratch.iter().zip(reference.iter()) {
        assert!((output - input * LV2_FIXTURE_GAIN).abs() < 1e-7);
    }

    // Gain port TTL range is 0..1, so normalized == plain.
    backend
        .set_parameter_normalized(LV2_FIXTURE_GAIN_PORT_INDEX, 1.0)
        .expect("param set queues");
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input).abs() < 1e-7,
            "sample {index}: {output} vs {input} (unity gain)",
        );
    }
    assert_eq!(
        backend.set_parameter_normalized(9999, 0.5).unwrap_err(),
        "unknown_parameter",
    );

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// g12.022: gui lifecycle through the in-process backend's delegates —
/// the exact surface the Tauri host calls (open/size/resize/events/
/// close), offscreen against the fixture's bookkeeping gui, while the
/// audio path keeps processing (gui takes the instance lock, never the
/// session lock).
#[test]
fn in_process_backend_hosts_the_fixture_gui_offscreen() {
    use signal_plugin_clap::fixture::{
        CLAP_FIXTURE_GUI_INITIAL_SIZE, CLAP_FIXTURE_GUI_REQUESTED_SIZE,
    };

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-gui-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let library = compile_clap_fixture(
        &directory,
        "com.signal.bridge-inproc-gui",
        "Signal Bridge InProc Gui",
        0,
    )
    .expect("fixture should compile");

    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(
            &library,
            "com.signal.bridge-inproc-gui",
            48_000,
            256,
        )
        .expect("backend should load and activate"),
    );
    assert!(backend.gui_supported());
    assert!(!backend.gui_is_open());
    assert_eq!(backend.gui_size(), None);
    assert_eq!(backend.state_dirty_request_count(), 0);

    let mut fake_parent = 0u8;
    let size = backend
        .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
        .expect("gui opens");
    assert_eq!(size, CLAP_FIXTURE_GUI_INITIAL_SIZE);
    assert!(backend.gui_is_open());
    assert_eq!(backend.gui_size(), Some(CLAP_FIXTURE_GUI_INITIAL_SIZE));
    assert!(backend.gui_can_resize());
    assert_eq!(backend.state_dirty_request_count(), 1);

    // Audio still processes with the editor open.
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    assert!(handle.process(&mut scratch, 128, 2));

    // Fixture show() queued a host resize request.
    let events = backend.gui_take_events();
    assert!(events.contains(&PluginGuiEvent::RequestResize {
        width: CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
        height: CLAP_FIXTURE_GUI_REQUESTED_SIZE.1,
    }));

    // Granting the request through set_size sticks.
    assert_eq!(
        backend.gui_set_size(
            CLAP_FIXTURE_GUI_REQUESTED_SIZE.0,
            CLAP_FIXTURE_GUI_REQUESTED_SIZE.1
        ),
        Some(CLAP_FIXTURE_GUI_REQUESTED_SIZE)
    );
    assert_eq!(backend.gui_size(), Some(CLAP_FIXTURE_GUI_REQUESTED_SIZE));

    backend.gui_close();
    assert!(!backend.gui_is_open());
    backend.gui_close(); // idempotent

    // Dead backends refuse to open editors.
    backend.shutdown();
    let refused = backend.gui_open_embedded(&mut fake_parent as *mut u8 as usize, None);
    assert_eq!(refused.unwrap_err(), "backend_dead");

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn in_process_vst3_backend_loads_and_processes_the_fixture() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-vst3-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let bundle = compile_vst3_fixture(
        &directory,
        "plugin:vst3:bridge-inproc",
        "Signal Bridge InProc VST3",
    )
    .expect("vst3 fixture should compile");

    let backend = Arc::new(
        InProcessVst3Processor::load_and_activate(&bundle, VST3_FIXTURE_CLASS_ID_HEX, 48_000, 256)
            .expect("backend should load and activate"),
    );
    assert_eq!(backend.parameters().len(), 2);
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let reference = scratch.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * VST3_FIXTURE_GAIN).abs() < 1e-7,
            "sample {index}: {output} vs {input} * {VST3_FIXTURE_GAIN}",
        );
    }
    assert_eq!(backend.miss_count(), 0);

    // A controller-requested processing restart makes the audio thread
    // bypass at the next block boundary. The control thread can then
    // rebuild the process session and resume without reloading the plug-in.
    backend
        .pending_restart_flags
        .store(VST3_RESTART_IO_CHANGED, Ordering::Release);
    let mut bypassed = reference.clone();
    assert!(!handle.process(&mut bypassed, 128, 2));
    assert_eq!(bypassed, reference);
    assert_eq!(backend.miss_count(), 1);
    assert!(backend.service_processing_restart().expect("restart"));
    assert!(!backend.processing_restart_pending());
    let mut resumed = reference.clone();
    assert!(handle.process(&mut resumed, 128, 2));
    for (output, input) in resumed.iter().zip(reference.iter()) {
        assert!((output - input * VST3_FIXTURE_GAIN).abs() < 1e-7);
    }

    // Shutdown: later blocks bypass and leave scratch untouched.
    backend.shutdown();
    let mut scratch = reference.clone();
    assert!(!handle.process(&mut scratch, 128, 2));
    assert_eq!(scratch, reference);
    assert_eq!(backend.miss_count(), 2);

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// The LV2 mirror of the in-process gain proof: wet = dry × the Gain
/// control port's non-unity TTL default (no param set exists phase 1),
/// byte-exact through the render handle, then shutdown-bypass leaves
/// the scratch untouched.
#[test]
fn in_process_lv2_backend_loads_and_processes_the_fixture() {
    use signal_plugin_lv2::fixture::{
        compile_lv2_fixture, rustc_available as lv2_rustc_available, LV2_FIXTURE_GAIN,
    };
    if !lv2_rustc_available() {
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-lv2-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let plugin_uri = "https://signal.dev/fixtures/lv2/bridge-inproc";
    let bundle = compile_lv2_fixture(&directory, plugin_uri, "Signal Bridge InProc LV2")
        .expect("lv2 fixture should compile");

    let backend = Arc::new(
        InProcessLv2Processor::load_and_activate(&bundle, plugin_uri, 48_000, 256)
            .expect("backend should load and activate"),
    );
    assert_eq!(backend.parameters().len(), 2);
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    assert_eq!(handle.event_support(), RenderPluginEventSupport::default());

    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let reference = scratch.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input * LV2_FIXTURE_GAIN).abs() < 1e-7,
            "sample {index}: {output} vs {input} * {LV2_FIXTURE_GAIN}",
        );
    }
    assert_eq!(backend.miss_count(), 0);

    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[RenderBlockPluginEvent {
            offset_frames: 0,
            channel: 0,
            kind: RenderPluginEventKind::NoteOn {
                key: 60,
                velocity: 1.0,
            },
        }],
    ));
    assert_eq!(handle.unsupported_event_count(), 1);

    // Shutdown: later blocks bypass and leave scratch untouched.
    backend.shutdown();
    let mut scratch = reference.clone();
    assert!(!handle.process(&mut scratch, 128, 2));
    assert_eq!(scratch, reference);
    assert_eq!(backend.miss_count(), 1);

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// g12.024: the VST3 IPlugView mirror of the CLAP gui lifecycle test —
/// the exact surface the Tauri host calls (open/size/resize/events/
/// close), offscreen against the fixture's bookkeeping view, while the
/// audio path keeps processing (gui takes the instance lock, never the
/// session lock).
#[test]
fn in_process_vst3_backend_hosts_the_fixture_view_offscreen() {
    use signal_plugin_vst3::fixture::{
        VST3_FIXTURE_VIEW_INITIAL_SIZE, VST3_FIXTURE_VIEW_REQUESTED_SIZE,
    };

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-vst3-gui-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let bundle = compile_vst3_fixture(
        &directory,
        "plugin:vst3:bridge-inproc-gui",
        "Signal Bridge InProc VST3 Gui",
    )
    .expect("vst3 fixture should compile");

    let backend = Arc::new(
        InProcessVst3Processor::load_and_activate(&bundle, VST3_FIXTURE_CLASS_ID_HEX, 48_000, 256)
            .expect("backend should load and activate"),
    );
    assert!(backend.gui_supported(), "edit controller is available");
    assert!(!backend.gui_is_open());
    assert_eq!(backend.gui_size(), None);

    let mut fake_parent = 0u8;
    let size = backend
        .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
        .expect("view opens");
    assert_eq!(size, VST3_FIXTURE_VIEW_INITIAL_SIZE);
    assert!(backend.gui_is_open());
    assert_eq!(backend.gui_size(), Some(VST3_FIXTURE_VIEW_INITIAL_SIZE));
    assert!(backend.gui_can_resize());

    // Audio still processes with the editor open.
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    assert!(handle.process(&mut scratch, 128, 2));

    // Fixture attached() asked the host IPlugFrame for a resize.
    let events = backend.gui_take_events();
    assert!(events.contains(&PluginGuiEvent::RequestResize {
        width: VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
        height: VST3_FIXTURE_VIEW_REQUESTED_SIZE.1,
    }));

    // Plugin-requested sizes bypass the host/user constraint pass and are
    // granted directly through onSize.
    assert_eq!(
        backend.gui_accept_plugin_resize(
            VST3_FIXTURE_VIEW_REQUESTED_SIZE.0,
            VST3_FIXTURE_VIEW_REQUESTED_SIZE.1
        ),
        Some(VST3_FIXTURE_VIEW_REQUESTED_SIZE)
    );
    assert_eq!(backend.gui_size(), Some(VST3_FIXTURE_VIEW_REQUESTED_SIZE));

    backend.gui_close();
    assert!(!backend.gui_is_open());
    backend.gui_close(); // idempotent

    // Dead backends refuse to open editors.
    backend.shutdown();
    let refused = backend.gui_open_embedded(&mut fake_parent as *mut u8 as usize, None);
    assert_eq!(refused.unwrap_err(), "backend_dead");

    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// g12.024: plugin GUI → host param sync — the fixture's gui `show`
/// stands in for an editor tweak, pushing a Gain PARAM_VALUE out-event
/// at the next processed block; the host drains it normalized and the
/// DSP already runs at the tweaked gain.
#[test]
fn in_process_clap_gui_param_tweak_reaches_the_host_via_out_events() {
    use signal_plugin_clap::fixture::{
        CLAP_FIXTURE_GAIN_PARAM_ID, CLAP_FIXTURE_GUI_PARAM_OUT_VALUE,
    };

    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the CLAP fixture");
        return;
    }
    let directory = std::env::temp_dir().join(format!(
        "signal-plugin-bridge-inproc-gui-out-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
    ));
    let library = compile_clap_fixture(
        &directory,
        "com.signal.bridge-inproc-gui-out",
        "Signal Bridge InProc Gui Out",
        0,
    )
    .expect("fixture should compile");

    let backend = Arc::new(
        InProcessClapProcessor::load_and_activate(
            &library,
            "com.signal.bridge-inproc-gui-out",
            48_000,
            256,
        )
        .expect("backend should load and activate"),
    );
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);

    // No editor interaction yet: no out-events.
    let reference: Vec<f32> = (0..256).map(|index| index as f32 / 256.0).collect();
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    assert!(backend.take_param_out_events().is_empty());

    // Open + the fixture's show() queues the stand-in editor tweak.
    let mut fake_parent = 0u8;
    backend
        .gui_open_embedded(&mut fake_parent as *mut u8 as usize, None)
        .expect("gui opens");

    // The tweak lands at the next processed block: audible in the DSP
    // and drained by the host as a normalized (id, value) pair (the
    // fixture Gain's plain range is 0..1, so plain == normalized).
    let mut scratch = reference.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (f64::from(*output) - f64::from(*input) * CLAP_FIXTURE_GUI_PARAM_OUT_VALUE).abs()
                < 1e-7,
            "sample {index}: {output} vs {input} * {CLAP_FIXTURE_GUI_PARAM_OUT_VALUE}",
        );
    }
    let drained = backend.take_param_out_events();
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].0, CLAP_FIXTURE_GAIN_PARAM_ID);
    assert!((f64::from(drained[0].1) - CLAP_FIXTURE_GUI_PARAM_OUT_VALUE).abs() < 1e-6);
    // Drained means drained: the next take is empty.
    assert!(backend.take_param_out_events().is_empty());

    // The fixture's show() also exercised the host clap.params wiring
    // (request_flush) — observable through the params-event drain.
    let params_events = backend.take_params_events();
    assert!(params_events.contains(&signal_plugin_clap::ClapHostParamsEvent::FlushRequested));
    assert!(backend.take_params_events().is_empty());

    backend.gui_close();
    drop(handle);
    drop(backend);
    let _ = std::fs::remove_dir_all(&directory);
}

/// The AU mirror of the in-process identity proof — against the stock
/// Apple AUDelay (no compiled fixture; the AudioComponent registrar
/// cannot see temp bundles). WetDryMix=0 makes the delay line inert, so
/// output ≈ input within 1e-6 per sample (AU float paths are
/// unspecified — never byte-exact).
#[cfg(target_os = "macos")]
#[test]
fn in_process_au_backend_is_identity_when_fully_dry() {
    const AUDELAY_WET_DRY_MIX: u32 = 0;
    let backend = Arc::new(
        InProcessAuProcessor::load_and_activate(
            std::path::Path::new(signal_plugin_au::AU_REGISTRY_COMPONENT_PATH),
            "aufx:dely:appl",
            48_000,
            256,
        )
        .expect("stock AUDelay should load and activate in-process"),
    );
    assert!(!backend.parameters().is_empty());
    backend
        .set_parameter(AUDELAY_WET_DRY_MIX, 0.0)
        .expect("wet/dry mix set");
    let handle = RenderPluginProcessor::new(Arc::clone(&backend) as Arc<_>);
    assert_eq!(handle.event_support(), AU_EVENT_SUPPORT);

    let mut scratch: Vec<f32> = (0..256).map(|index| index as f32 / 256.0 - 0.5).collect();
    let reference = scratch.clone();
    assert!(handle.process(&mut scratch, 128, 2));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input).abs() <= 1e-6,
            "sample {index}: {output} vs {input} (identity, epsilon 1e-6)",
        );
    }
    assert_eq!(backend.miss_count(), 0);

    // g12.034 follow-up, AU honest fallback: AUDelay is a plain effect
    // that refuses MusicDeviceMIDIEvent per event — delivered note/CC
    // events must not crash or disturb the audio path.
    let mut scratch = reference.clone();
    assert!(handle.process_with_events(
        &mut scratch,
        128,
        2,
        &[
            RenderBlockPluginEvent {
                offset_frames: 0,
                channel: 0,
                kind: RenderPluginEventKind::NoteOn {
                    key: 60,
                    velocity: 1.0,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 64,
                channel: 0,
                kind: RenderPluginEventKind::ControlChange {
                    controller: 7,
                    value: 0.5,
                },
            },
            RenderBlockPluginEvent {
                offset_frames: 96,
                channel: 0,
                kind: RenderPluginEventKind::NoteExpression {
                    key: 60,
                    expression: RenderNoteExpressionKind::Pressure,
                    value: 0.75,
                },
            },
        ],
    ));
    for (index, (output, input)) in scratch.iter().zip(reference.iter()).enumerate() {
        assert!(
            (output - input).abs() <= 1e-6,
            "sample {index}: {output} vs {input} (identity after refused MIDI)",
        );
    }
    assert_eq!(backend.miss_count(), 0);
    assert_eq!(handle.unsupported_event_count(), 1);

    // Shutdown: later blocks bypass and leave scratch untouched.
    backend.shutdown();
    let mut scratch = reference.clone();
    assert!(!handle.process(&mut scratch, 128, 2));
    assert_eq!(scratch, reference);
    assert_eq!(backend.miss_count(), 1);

    drop(handle);
    drop(backend);
}
