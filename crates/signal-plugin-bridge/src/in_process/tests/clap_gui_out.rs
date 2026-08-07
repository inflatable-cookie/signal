//! In-process backend unit tests.

use super::prelude::*;

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
