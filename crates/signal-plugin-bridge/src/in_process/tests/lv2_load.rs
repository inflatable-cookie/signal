//! In-process backend unit tests.

use super::prelude::*;

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
