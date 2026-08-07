//! In-process backend unit tests.

use super::prelude::*;

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
