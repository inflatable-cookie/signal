//! Hosting round-trip tests against the rustc-compiled LV2 fixture bundle:
//! load through the real TTL-reparse/dlopen/descriptor-walk path,
//! parameter inventory from the TTL (control ports as parameters), the
//! stereo gate (atom-input variant rejected), activation (instantiate at
//! activate, urid:map delivered), and byte-exact fixed-gain processing
//! through both session entry points — wet = dry × the Gain port's
//! NON-UNITY TTL default, no param set involved.
//! Skips gracefully when `rustc` is unavailable (the fixture pattern).

use signal_plugin_lv2::fixture::{
    compile_lv2_atom_fixture, compile_lv2_fixture, rustc_available, LV2_FIXTURE_BYPASS_PORT_INDEX,
    LV2_FIXTURE_GAIN, LV2_FIXTURE_GAIN_PORT_INDEX,
};
use signal_plugin_lv2::{current_lv2_platform, Lv2HostAdapter, Lv2HostedInstance};

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
            "signal-lv2-hosting-{label}-{}-{}",
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
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = unique_fixture_dir("roundtrip");
    let plugin_uri = "https://signal.dev/fixtures/lv2/roundtrip";
    let bundle = compile_lv2_fixture(&directory.path, plugin_uri, "Signal LV2 Fixture")
        .expect("fixture should compile");

    let mut instance =
        Lv2HostedInstance::load(&bundle, plugin_uri).expect("fixture should load by URI");

    // Parameter inventory comes from the bundle TTL: control input ports
    // double as parameters, parameter_id = port index.
    let parameters = instance.parameters();
    assert_eq!(parameters.len(), 2, "fixture exposes Gain + Bypass");
    let gain = parameters
        .iter()
        .find(|parameter| parameter.name == "Gain")
        .expect("Gain parameter in the inventory");
    assert_eq!(gain.parameter_id, LV2_FIXTURE_GAIN_PORT_INDEX);
    assert!((gain.min_plain - 0.0).abs() < 1e-6);
    assert!((gain.max_plain - 1.0).abs() < 1e-6);
    assert!((gain.default_normalized - LV2_FIXTURE_GAIN).abs() < 1e-6);
    // Descriptor enrichment (g12.013): units:unit from the TTL, continuous
    // range, automatable, not bypass.
    assert_eq!(gain.unit.as_deref(), Some("coef"));
    assert_eq!(gain.step_count, None);
    assert!(gain.is_automatable());
    assert!(!gain.is_bypass());
    let bypass = parameters
        .iter()
        .find(|parameter| parameter.name == "Bypass")
        .expect("Bypass parameter in the inventory");
    assert_eq!(bypass.parameter_id, LV2_FIXTURE_BYPASS_PORT_INDEX);
    // lv2:portProperty lv2:toggled = one step; lv2:designation lv2:enabled
    // marks the bypass control.
    assert_eq!(bypass.step_count, Some(1));
    assert!(bypass.flags.stepped);
    assert!(bypass.is_bypass());
    assert_eq!(bypass.unit, None);

    // Stereo effect gate reads the TTL port model.
    let layout = instance.port_layout();
    assert_eq!(layout.main_input_channels, 2);
    assert_eq!(layout.main_output_channels, 2);
    assert_eq!(layout.required_event_inputs, 0);
    assert!(layout.is_stereo_effect());

    // Sessions are only valid while active; instantiate runs at activate.
    assert!(instance.process_session().is_err());
    instance
        .activate(48_000.0, 1, 256)
        .expect("stereo fixture should activate (urid:map delivered)");
    assert!(instance.activate(48_000.0, 1, 256).is_err(), "no re-entry");

    {
        let mut session = instance
            .process_session()
            .expect("active instance builds a session");
        session.start().expect("start (push model, always ok)");
        assert!(session.is_processing());

        // Split-buffer path: wet = dry × the Gain port's TTL default,
        // byte-exact (our own fixture math).
        let frames = 128usize;
        let input: Vec<f32> = (0..frames * 2).map(|index| index as f32 / 256.0).collect();
        let mut output = vec![0.0f32; frames * 2];
        assert!(session.process_interleaved_stereo(&input, &mut output, frames));
        for (index, (wet, dry)) in output.iter().zip(input.iter()).enumerate() {
            assert!(
                (wet - dry * LV2_FIXTURE_GAIN).abs() < 1e-7,
                "sample {index}: {wet} vs {dry} * {LV2_FIXTURE_GAIN}",
            );
        }

        // In-place path used by the in-process tier.
        let mut scratch = input.clone();
        assert!(session.process_in_place(&mut scratch, frames));
        for (index, (wet, dry)) in scratch.iter().zip(input.iter()).enumerate() {
            assert!(
                (wet - dry * LV2_FIXTURE_GAIN).abs() < 1e-7,
                "in-place sample {index}: {wet} vs {dry} * {LV2_FIXTURE_GAIN}",
            );
        }

        session.stop();
        assert!(!session.is_processing());
    }
    instance.deactivate().expect("deactivate");
    assert!(instance.deactivate().is_err(), "double deactivate rejected");

    // Reactivation re-instantiates at the (possibly new) rate.
    instance
        .activate(44_100.0, 1, 128)
        .expect("reactivate at a new rate");
    instance.deactivate().expect("deactivate again");
}

/// The atom-input variant loads (valid LV2) but fails the stereo gate:
/// exactly 2+2 audio ports AND zero required atom/event inputs.
#[test]
fn atom_input_variant_is_rejected_by_the_stereo_gate() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = unique_fixture_dir("atomgate");
    let plugin_uri = "https://signal.dev/fixtures/lv2/atom-gate";
    let bundle = compile_lv2_atom_fixture(&directory.path, plugin_uri, "Signal LV2 Atom Fixture")
        .expect("atom fixture should compile");

    let instance = Lv2HostedInstance::load(&bundle, plugin_uri).expect("atom variant loads");
    let layout = instance.port_layout();
    assert_eq!(layout.main_input_channels, 2);
    assert_eq!(layout.main_output_channels, 2);
    assert_eq!(layout.required_event_inputs, 1);
    assert!(
        !layout.is_stereo_effect(),
        "required atom input must fail the stereo gate",
    );
}

#[test]
fn load_rejects_bad_uris_missing_bundles_and_unsupported_features() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = unique_fixture_dir("errors");
    let plugin_uri = "https://signal.dev/fixtures/lv2/errors";
    let bundle = compile_lv2_fixture(&directory.path, plugin_uri, "Signal LV2 Fixture Errors")
        .expect("fixture should compile");

    fn error_token(
        result: Result<Lv2HostedInstance, signal_plugin_lv2::Lv2HostingError>,
    ) -> String {
        match result {
            Ok(_) => panic!("load should have failed"),
            Err(error) => error.token,
        }
    }

    let unknown_uri = Lv2HostedInstance::load(&bundle, "https://example.com/not-in-this-bundle");
    assert_eq!(error_token(unknown_uri), "plugin_uri_not_found");

    let missing = Lv2HostedInstance::load(std::path::Path::new("/nonexistent/x.lv2"), plugin_uri);
    assert_eq!(error_token(missing), "bundle_parse_failed");

    // requiredFeature beyond urid#map fails at LOAD with the typed token
    // (in addition to the scan pre-filter).
    let worker_bundle = directory.path.join("worker-required.lv2");
    std::fs::create_dir_all(&worker_bundle).expect("bundle dir");
    std::fs::write(
        worker_bundle.join("manifest.ttl"),
        "@prefix lv2: <http://lv2plug.in/ns/lv2core#> .\n\
         <https://example.com/worker-required> a lv2:Plugin ;\n\
         \tlv2:binary <w.so> ;\n\
         \tlv2:requiredFeature <http://lv2plug.in/ns/ext/worker#schedule> ;\n\
         \tlv2:port [ a lv2:AudioPort , lv2:InputPort ; lv2:index 0 ; lv2:symbol \"in\" ] .\n",
    )
    .expect("manifest write");
    let unsupported =
        Lv2HostedInstance::load(&worker_bundle, "https://example.com/worker-required");
    assert_eq!(error_token(unsupported), "unsupported_required_feature");
}

/// Discovery over the compiled fixture bundle: the same TTL parse the
/// hosting side uses, through the scan entry point.
#[test]
fn discovery_reports_the_fixture_with_its_ttl_inventory() {
    if !rustc_available() {
        eprintln!("skipping: rustc unavailable for the LV2 fixture");
        return;
    }
    let directory = unique_fixture_dir("discovery");
    let plugin_uri = "https://signal.dev/fixtures/lv2/discovery";
    let _bundle = compile_lv2_fixture(&directory.path, plugin_uri, "Signal LV2 Fixture Scan")
        .expect("fixture should compile");

    let adapter = Lv2HostAdapter::default();
    let discovered = adapter.discover_plugins_for_roots(
        current_lv2_platform(),
        &[directory.path.display().to_string()],
    );
    assert_eq!(discovered.len(), 1);
    let plugin = &discovered[0];
    assert_eq!(plugin.plugin_type_id.0, format!("plugin:lv2:{plugin_uri}"));
    assert_eq!(plugin.plugin_uri, plugin_uri);
    assert!(plugin.bundle_root.ends_with(".lv2"));
    assert_eq!(plugin.descriptor.name, "Signal LV2 Fixture Scan");
    assert_eq!(plugin.descriptor.parameters.len(), 2);
    assert_eq!(plugin.default_io_layout.audio_inputs, 2);
    assert_eq!(plugin.default_io_layout.audio_outputs, 2);
    assert_eq!(
        plugin.required_features,
        vec!["http://lv2plug.in/ns/ext/urid#map".to_string()],
    );
}
