//! In-process backend unit tests.

use super::prelude::*;

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
