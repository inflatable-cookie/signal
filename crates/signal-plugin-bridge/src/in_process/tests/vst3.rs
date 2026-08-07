//! In-process backend unit tests.

use super::prelude::*;

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
