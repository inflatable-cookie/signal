use super::*;
use crate::ClapHostPlatform;

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
fn clap_adapter_default_scan_roots_cover_macos_and_linux() {
    let adapter = ClapPluginHostAdapter::default();
    let macos = adapter
        .default_scan_roots(ClapHostPlatform::MacOs)
        .into_iter()
        .map(|root| root.root)
        .collect::<Vec<_>>();
    assert_eq!(
        &macos[..2],
        [
            "~/Library/Audio/Plug-Ins/CLAP".to_string(),
            "/Library/Audio/Plug-Ins/CLAP".to_string(),
        ]
    );

    let linux = adapter
        .default_scan_roots(ClapHostPlatform::Linux)
        .into_iter()
        .map(|root| root.root)
        .collect::<Vec<_>>();
    assert_eq!(
        &linux[..3],
        [
            "~/.clap".to_string(),
            "/usr/lib/clap".to_string(),
            "/usr/local/lib/clap".to_string(),
        ]
    );
}

#[test]
fn clap_adapter_default_scan_roots_cover_windows() {
    let adapter = ClapPluginHostAdapter::default();
    let windows = adapter
        .default_scan_roots(ClapHostPlatform::Windows)
        .into_iter()
        .map(|root| root.root)
        .collect::<Vec<_>>();
    assert_eq!(
        &windows[..2],
        [
            r"%COMMONPROGRAMFILES%\CLAP".to_string(),
            r"%LOCALAPPDATA%\Programs\Common\CLAP".to_string(),
        ]
    );
}

#[test]
fn clap_adapter_descriptor_only_scan_never_instantiates_plugins() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.descriptor-scan-fixture",
        "Signal Descriptor Scan Fixture",
        1,
    );

    let discovered = adapter.discover_plugins_for_roots(&[scan_root.root()]);

    assert_eq!(discovered.len(), 1);
    let discovered = &discovered[0];
    assert_eq!(
        discovered.plugin_type_id.0,
        "com.signal.descriptor-scan-fixture"
    );
    assert_eq!(
        discovered.descriptor.plugin_id,
        "com.signal.descriptor-scan-fixture"
    );
    assert_eq!(discovered.descriptor.name, "Signal Descriptor Scan Fixture");
    assert_eq!(discovered.descriptor.vendor, "Signal");
    assert_eq!(discovered.descriptor.version.as_deref(), Some("0.1.0"));
    assert_eq!(discovered.descriptor.format, PluginFormat::Clap);
    assert!(!discovered.descriptor.features.is_empty());
    // Descriptor-only discovery must not create plugin instances, so the
    // instance-derived shape stays empty/conservative.
    assert_eq!(discovered.default_io_layout.audio_inputs, 0);
    assert_eq!(discovered.default_io_layout.audio_outputs, 0);
    assert_eq!(discovered.default_io_layout.midi_inputs, 0);
    assert_eq!(discovered.default_io_layout.midi_outputs, 0);
    assert!(discovered.descriptor.audio_buses.is_empty());
    assert!(discovered.descriptor.parameters.is_empty());
    assert!(!discovered.descriptor.state_contract.supports_snapshot);
    assert!(!discovered.descriptor.state_contract.exposes_latency);
    assert!(!discovered.descriptor.state_contract.exposes_tail);

    // The catalog only resolves what the scan actually found.
    assert!(adapter
        .discover_plugin_type("com.signal.descriptor-scan-fixture")
        .is_some());
    assert!(adapter
        .discover_plugin_type("plugin:clap:default")
        .is_none());
}

#[test]
fn clap_adapter_discovers_direct_macos_bundle_root() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.bundle-root-fixture",
        "Signal Bundle Root Fixture",
        0,
    );
    let compiled = scan_root.path().join("signal-bundle-root-fixture.clap");
    let bundle = scan_root.path().join("Signal Bundle Root Fixture.clap");
    let bundle_binary = bundle
        .join("Contents")
        .join("MacOS")
        .join("Signal Bundle Root Fixture");
    std::fs::create_dir_all(bundle_binary.parent().expect("bundle binary parent"))
        .expect("bundle dirs");
    std::fs::rename(compiled, &bundle_binary).expect("fixture moved into bundle");

    let discovered = adapter
        .discover_plugins_for_platform(ClapHostPlatform::MacOs, &[bundle.display().to_string()]);

    assert_eq!(discovered.len(), 1);
    let discovered = &discovered[0];
    assert_eq!(
        discovered.plugin_type_id.0,
        "com.signal.bundle-root-fixture"
    );
    assert_eq!(discovered.library_path, bundle_binary.display().to_string());
}

#[test]
fn clap_adapter_discovers_host_architecture_linux_bundle() {
    if !crate::fixture::rustc_available() {
        return;
    }
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.linux-bundle-fixture",
        "Signal Linux Bundle Fixture",
        0,
    );
    let compiled = scan_root.path().join("signal-linux-bundle-fixture.clap");
    let bundle = scan_root.path().join("Signal Linux Bundle Fixture.clap");
    let architecture = if cfg!(target_arch = "aarch64") {
        "aarch64-linux"
    } else {
        "x86_64-linux"
    };
    let bundle_binary = bundle
        .join("Contents")
        .join(architecture)
        .join("Signal Linux Bundle Fixture.so");
    std::fs::create_dir_all(bundle_binary.parent().expect("bundle binary parent"))
        .expect("bundle dirs");
    std::fs::rename(compiled, &bundle_binary).expect("fixture moved into Linux bundle");

    let discovered = adapter
        .discover_plugins_for_platform(ClapHostPlatform::Linux, &[bundle.display().to_string()]);

    assert_eq!(discovered.len(), 1);
    assert_eq!(
        discovered[0].plugin_type_id.0,
        "com.signal.linux-bundle-fixture"
    );
    assert_eq!(
        discovered[0].library_path,
        bundle_binary.display().to_string()
    );
}

#[test]
fn clap_adapter_does_not_discover_macos_only_bundle_on_linux() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.macos-only-bundle-fixture",
        "Signal macOS Only Bundle Fixture",
        0,
    );
    let compiled = scan_root
        .path()
        .join("signal-macos-only-bundle-fixture.clap");
    let bundle = scan_root
        .path()
        .join("Signal macOS Only Bundle Fixture.clap");
    let bundle_binary = bundle
        .join("Contents")
        .join("MacOS")
        .join("Signal macOS Only Bundle Fixture");
    std::fs::create_dir_all(bundle_binary.parent().expect("bundle binary parent"))
        .expect("bundle dirs");
    std::fs::rename(compiled, &bundle_binary).expect("fixture moved into macOS bundle");

    let discovered = adapter
        .discover_plugins_for_platform(ClapHostPlatform::Linux, &[bundle.display().to_string()]);

    assert!(discovered.is_empty());
}

#[test]
fn clap_adapter_discovers_flat_clap_file_on_windows() {
    if !crate::fixture::rustc_available() {
        return;
    }
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.windows-flat-fixture",
        "Signal Windows Flat Fixture",
        0,
    );
    let library = scan_root.path().join("signal-windows-flat-fixture.clap");

    let discovered =
        adapter.discover_plugins_for_platform(ClapHostPlatform::Windows, &[scan_root.root()]);

    assert_eq!(discovered.len(), 1);
    assert_eq!(
        discovered[0].plugin_type_id.0,
        "com.signal.windows-flat-fixture"
    );
    assert_eq!(discovered[0].library_path, library.display().to_string());
}

#[test]
fn clap_adapter_does_not_discover_macos_only_bundle_on_windows() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.macos-bundle-on-windows-fixture",
        "Signal macOS Bundle On Windows Fixture",
        0,
    );
    let compiled = scan_root
        .path()
        .join("signal-macos-bundle-on-windows-fixture.clap");
    let bundle = scan_root
        .path()
        .join("Signal macOS Bundle On Windows Fixture.clap");
    let bundle_binary = bundle
        .join("Contents")
        .join("MacOS")
        .join("Signal macOS Bundle On Windows Fixture");
    std::fs::create_dir_all(bundle_binary.parent().expect("bundle binary parent"))
        .expect("bundle dirs");
    std::fs::rename(compiled, &bundle_binary).expect("fixture moved into macOS bundle");

    let discovered = adapter
        .discover_plugins_for_platform(ClapHostPlatform::Windows, &[bundle.display().to_string()]);

    assert!(discovered.is_empty());
}

#[test]
fn clap_adapter_does_not_discover_linux_only_bundle_on_windows() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.linux-bundle-on-windows-fixture",
        "Signal Linux Bundle On Windows Fixture",
        0,
    );
    let compiled = scan_root
        .path()
        .join("signal-linux-bundle-on-windows-fixture.clap");
    let bundle = scan_root
        .path()
        .join("Signal Linux Bundle On Windows Fixture.clap");
    let bundle_binary = bundle
        .join("Contents")
        .join("x86_64-linux")
        .join("Signal Linux Bundle On Windows Fixture.so");
    std::fs::create_dir_all(bundle_binary.parent().expect("bundle binary parent"))
        .expect("bundle dirs");
    std::fs::rename(compiled, &bundle_binary).expect("fixture moved into Linux bundle");

    let discovered = adapter
        .discover_plugins_for_platform(ClapHostPlatform::Windows, &[bundle.display().to_string()]);

    assert!(discovered.is_empty());
}

#[test]
fn clap_adapter_does_not_discover_windows_directory_as_clap_bundle() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.windows-directory-bundle-fixture",
        "Signal Windows Directory Bundle Fixture",
        0,
    );
    let compiled = scan_root
        .path()
        .join("signal-windows-directory-bundle-fixture.clap");
    let bundle = scan_root
        .path()
        .join("Signal Windows Directory Bundle Fixture.clap");
    let bundle_binary = bundle
        .join("Contents")
        .join("x86_64-win")
        .join("Signal Windows Directory Bundle Fixture.clap");
    std::fs::create_dir_all(bundle_binary.parent().expect("bundle binary parent"))
        .expect("bundle dirs");
    std::fs::rename(compiled, &bundle_binary).expect("fixture moved into Windows directory");

    let discovered = adapter
        .discover_plugins_for_platform(ClapHostPlatform::Windows, &[bundle.display().to_string()]);
    assert!(discovered.is_empty());

    let scanned_root =
        adapter.discover_plugins_for_platform(ClapHostPlatform::Windows, &[scan_root.root()]);
    assert!(scanned_root.is_empty());
}

#[test]
fn clap_adapter_opt_in_capability_probe_reads_full_plugin_shape() {
    let adapter = ClapPluginHostAdapter::default();
    let scan_root = temp_real_clap_scan_root(
        "com.signal.probe-scan-fixture",
        "Signal Probe Scan Fixture",
        1,
    );

    let discovered = adapter.discover_plugins_for_roots_with_options(&[scan_root.root()], true);

    assert_eq!(discovered.len(), 1);
    let discovered = &discovered[0];
    assert_eq!(discovered.plugin_type_id.0, "com.signal.probe-scan-fixture");
    assert_eq!(discovered.descriptor.name, "Signal Probe Scan Fixture");
    assert_eq!(discovered.default_io_layout.audio_inputs, 2);
    assert_eq!(discovered.default_io_layout.audio_outputs, 2);
    assert_eq!(discovered.default_io_layout.midi_inputs, 1);
    assert_eq!(discovered.default_io_layout.midi_outputs, 1);
    assert_eq!(discovered.descriptor.audio_buses.len(), 2);
    assert_eq!(discovered.descriptor.parameters.len(), 2);
    assert!(discovered.descriptor.state_contract.supports_snapshot);
    assert!(discovered.descriptor.state_contract.exposes_latency);
    assert!(discovered.descriptor.state_contract.exposes_tail);
    assert!(discovered.descriptor.processing_contract.accepts_midi);
}

#[test]
fn clap_adapter_scans_nothing_for_empty_roots() {
    let adapter = ClapPluginHostAdapter::default();
    assert!(adapter.discover_plugins_for_roots(&[]).is_empty());
}
