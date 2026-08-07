//! In-process backend unit tests.

use super::prelude::*;

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
