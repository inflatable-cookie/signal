//! In-process backend unit tests.

use super::prelude::*;

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
