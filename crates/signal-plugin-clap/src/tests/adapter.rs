use super::*;

#[test]
fn clap_adapter_reports_supported_format_and_extensions() {
    let adapter = ClapPluginHostAdapter::default();
    assert!(adapter.supports_format(PluginFormat::Clap));
    assert!(adapter
        .minimum_extension_set()
        .contains(&ClapHostExtension::Params));
    assert_eq!(adapter.minimum_extension_set()[0].as_str(), "audio-ports");
}

#[test]
fn clap_adapter_discovers_concrete_plugin_type_metadata() {
    let adapter = ClapPluginHostAdapter::default();
    let discovered = adapter
        .discover_plugin_type("plugin:clap:sandbox")
        .expect("discovered sandbox plugin");

    assert_eq!(discovered.plugin_type_id.0, "plugin:clap:sandbox");
    assert_eq!(discovered.descriptor.plugin_id, "plugin:clap:sandbox");
    assert_eq!(discovered.descriptor.name, "Signal Sandbox CLAP Plugin");
    assert_eq!(discovered.descriptor.format, PluginFormat::Clap);
    assert_eq!(discovered.default_io_layout.audio_inputs, 2);
    assert_eq!(discovered.default_io_layout.audio_outputs, 2);
    assert_eq!(discovered.default_io_layout.midi_inputs, 1);
    assert_eq!(discovered.default_io_layout.midi_outputs, 1);
}

#[test]
fn clap_adapter_scans_real_clap_libraries_from_roots() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.real-scan-fixture",
        "Signal Real Scan Fixture",
        1,
    );

    let discovered = adapter.discover_plugins_for_roots(&[scan_root.root()]);

    assert_eq!(discovered.len(), 1);
    let discovered = &discovered[0];
    assert_eq!(discovered.plugin_type_id.0, "com.signal.real-scan-fixture");
    assert_eq!(discovered.descriptor.plugin_id, "com.signal.real-scan-fixture");
    assert_eq!(discovered.descriptor.name, "Signal Real Scan Fixture");
    assert_eq!(discovered.descriptor.vendor, "Signal");
    assert_eq!(discovered.default_io_layout.audio_inputs, 2);
    assert_eq!(discovered.default_io_layout.audio_outputs, 2);
    assert_eq!(discovered.default_io_layout.midi_inputs, 1);
    assert_eq!(discovered.default_io_layout.midi_outputs, 1);
    assert_eq!(discovered.descriptor.audio_buses.len(), 2);
    assert_eq!(discovered.descriptor.parameters.len(), 2);
    assert!(discovered.descriptor.state_contract.supports_snapshot);
    assert!(discovered.descriptor.state_contract.exposes_latency);
    assert!(discovered.descriptor.state_contract.exposes_tail);
}

#[test]
fn clap_protocol_descriptor_projects_plugin_neutral_contract_surface() {
    let protocol = ClapBlockProtocol::new(
        "plugin:clap:test",
        "instance-a",
        PluginIoLayout {
            audio_inputs: 2,
            audio_outputs: 2,
            midi_inputs: 1,
            midi_outputs: 1,
        },
        2048,
    );

    let descriptor = protocol.descriptor();
    assert_eq!(descriptor.plugin_id, "plugin:clap:test");
    assert_eq!(descriptor.version.as_deref(), Some("0.1.0"));
    assert_eq!(descriptor.audio_buses.len(), 2);
    assert_eq!(descriptor.parameters.len(), 2);
    assert!(descriptor.processing_contract.sample_accurate_automation);
    assert!(descriptor.processing_contract.accepts_midi);
    assert!(descriptor.state_contract.supports_snapshot);
    assert!(descriptor.lifecycle_contract.supports_reset_while_active);
}
