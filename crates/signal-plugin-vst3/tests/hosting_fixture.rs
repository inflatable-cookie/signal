//! Hosting round-trip tests against the rustc-compiled VST3 fixture bundle:
//! load through the real module-resolution/dlopen/COM path, parameter
//! inventory via IEditController, stereo bus negotiation, activation, and
//! byte-exact fixed-gain processing through both session entry points.
//! Skips gracefully when `rustc` is unavailable (the fixture pattern).

use signal_plugin_vst3::fixture::{
    compile_vst3_fixture, rustc_available, VST3_FIXTURE_CLASS_ID_HEX, VST3_FIXTURE_GAIN,
};
use signal_plugin_vst3::{current_vst3_platform, Vst3HostAdapter, Vst3HostedInstance};

struct FixtureDir {
    path: std::path::PathBuf,
}

impl Drop for FixtureDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn unique_fixture_dir(label: &str) -> FixtureDir {
    FixtureDir {
        path: std::env::temp_dir().join(format!(
            "signal-vst3-hosting-{label}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
        )),
    }
}

#[test]
fn hosted_instance_loads_activates_and_processes_the_fixture() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = unique_fixture_dir("roundtrip");
    let bundle = compile_vst3_fixture(
        &directory.path,
        "plugin:vst3:signal-fixture",
        "Signal VST3 Fixture",
    )
    .expect("fixture should compile");

    let mut instance = Vst3HostedInstance::load(&bundle, VST3_FIXTURE_CLASS_ID_HEX)
        .expect("fixture should load and initialize");

    // Parameter inventory arrives via IEditController at load time.
    let parameters = instance.parameters();
    assert_eq!(parameters.len(), 2, "fixture exposes Gain + Bypass");
    let gain = parameters
        .iter()
        .find(|parameter| parameter.name == "Gain")
        .expect("Gain parameter in the inventory");
    assert_eq!(gain.parameter_id, 4096);
    assert!((gain.min_plain - 0.0).abs() < 1e-6);
    assert!((gain.max_plain - 1.0).abs() < 1e-6);
    assert!((gain.default_normalized - 0.5).abs() < 1e-6);
    assert!(gain.flags.automatable);
    // Descriptor enrichment (g12.013): ParameterInfo.units and step_count
    // plumb through.
    assert_eq!(gain.unit.as_deref(), Some("dB"));
    assert_eq!(gain.step_count, None);
    assert!(gain.is_automatable());
    assert!(!gain.is_bypass());
    let bypass = parameters
        .iter()
        .find(|parameter| parameter.name == "Bypass")
        .expect("Bypass parameter in the inventory");
    assert_eq!(bypass.parameter_id, 0);
    assert!(bypass.flags.stepped);
    assert_eq!(bypass.step_count, Some(1), "ParameterInfo.step_count = 1");
    assert_eq!(bypass.unit, None, "empty units field maps to None");
    assert!(bypass.is_bypass());
    assert!(bypass.is_automatable());

    // Stereo effect gate reads the main audio buses.
    let layout = instance.port_layout();
    assert_eq!(layout.main_input_channels, 2);
    assert_eq!(layout.main_output_channels, 2);
    assert!(layout.is_stereo_effect());

    // Sessions are only valid while active.
    assert!(instance.process_session().is_err());
    instance
        .activate(48_000.0, 1, 256)
        .expect("stereo fixture should activate");
    assert!(instance.activate(48_000.0, 1, 256).is_err(), "no re-entry");

    let mut session = instance
        .process_session()
        .expect("active instance builds a session");
    session.start().expect("setProcessing(true)");
    assert!(session.is_processing());

    // Split-buffer path: wet = dry × fixture gain, byte-exact.
    let frames = 128usize;
    let input: Vec<f32> = (0..frames * 2).map(|index| index as f32 / 256.0).collect();
    let mut output = vec![0.0f32; frames * 2];
    assert!(session.process_interleaved_stereo(&input, &mut output, frames));
    for (index, (wet, dry)) in output.iter().zip(input.iter()).enumerate() {
        assert!(
            (wet - dry * VST3_FIXTURE_GAIN).abs() < 1e-7,
            "sample {index}: {wet} vs {dry} * {VST3_FIXTURE_GAIN}",
        );
    }

    // In-place path used by the in-process tier.
    let mut scratch = input.clone();
    assert!(session.process_in_place(&mut scratch, frames));
    for (index, (wet, dry)) in scratch.iter().zip(input.iter()).enumerate() {
        assert!(
            (wet - dry * VST3_FIXTURE_GAIN).abs() < 1e-7,
            "in-place sample {index}: {wet} vs {dry} * {VST3_FIXTURE_GAIN}",
        );
    }

    session.stop();
    assert!(!session.is_processing());
    drop(session);
    instance.deactivate().expect("deactivate");
    assert!(instance.deactivate().is_err(), "double deactivate rejected");
}

#[test]
fn load_rejects_bad_class_ids_and_missing_bundles() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = unique_fixture_dir("errors");
    let bundle = compile_vst3_fixture(
        &directory.path,
        "plugin:vst3:signal-fixture-errors",
        "Signal VST3 Fixture Errors",
    )
    .expect("fixture should compile");

    fn error_token(
        result: Result<Vst3HostedInstance, signal_plugin_vst3::Vst3HostingError>,
    ) -> String {
        match result {
            Ok(_) => panic!("load should have failed"),
            Err(error) => error.token,
        }
    }

    let malformed = Vst3HostedInstance::load(&bundle, "not-hex");
    assert_eq!(error_token(malformed), "class_id_invalid");

    let unknown_cid = Vst3HostedInstance::load(&bundle, "00000000000000000000000000000000");
    assert_eq!(error_token(unknown_cid), "create_component_failed");

    let missing = Vst3HostedInstance::load(
        std::path::Path::new("/nonexistent/fixture.vst3"),
        VST3_FIXTURE_CLASS_ID_HEX,
    );
    #[cfg(target_os = "macos")]
    assert_eq!(error_token(missing), "module_bundle_open_failed");
    #[cfg(not(target_os = "macos"))]
    assert_eq!(error_token(missing), "module_binary_unresolved");
}

#[test]
fn discovery_reports_the_fixture_with_an_empty_scan_time_inventory() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = unique_fixture_dir("discovery");
    let _bundle = compile_vst3_fixture(
        &directory.path,
        "plugin:vst3:signal-fixture-scan",
        "Signal VST3 Fixture Scan",
    )
    .expect("fixture should compile");

    let adapter = Vst3HostAdapter::default();
    let discovered = adapter.discover_plugins_for_roots(
        current_vst3_platform(),
        &[directory.path.display().to_string()],
    );
    assert_eq!(discovered.len(), 1);
    let plugin = &discovered[0];
    assert_eq!(plugin.plugin_type_id.0, "plugin:vst3:signal-fixture-scan");
    assert_eq!(plugin.class_id, VST3_FIXTURE_CLASS_ID_HEX);
    assert!(
        plugin.descriptor.parameters.is_empty(),
        "scan-time VST3 inventory must be empty (real inventory at load)",
    );
}

#[test]
fn discovery_falls_back_to_factory_when_moduleinfo_is_missing() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the VST3 fixture");
        return;
    }
    let directory = unique_fixture_dir("discovery-fallback");
    let bundle = compile_vst3_fixture(
        &directory.path,
        "plugin:vst3:signal-fixture-fallback",
        "Signal VST3 Fixture Fallback",
    )
    .expect("fixture should compile");
    std::fs::remove_file(
        bundle
            .join("Contents")
            .join("Resources")
            .join("moduleinfo.json"),
    )
    .expect("moduleinfo should be removable");

    let adapter = Vst3HostAdapter::default();
    let discovered = adapter.discover_plugins_for_roots(
        current_vst3_platform(),
        &[directory.path.display().to_string()],
    );

    assert_eq!(discovered.len(), 1);
    let plugin = &discovered[0];
    assert_eq!(
        plugin.plugin_type_id.0,
        "plugin:vst3:signal-fixture-fallback"
    );
    assert_eq!(plugin.class_id, VST3_FIXTURE_CLASS_ID_HEX);
}
