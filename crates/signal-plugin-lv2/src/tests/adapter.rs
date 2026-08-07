use super::*;

#[test]
fn lv2_adapter_reports_supported_format_and_capabilities() {
    let adapter = Lv2HostAdapter::default();
    assert!(adapter.supports_format(PluginFormat::Lv2));
    assert!(!adapter.supports_format(PluginFormat::Clap));
    assert!(adapter.strict_sandbox_default());
    assert!(adapter.advertised_capabilities(2048).supports_state);
}

#[test]
fn default_scan_roots_cover_macos_and_linux() {
    let adapter = Lv2HostAdapter::default();
    let macos: Vec<_> = adapter
        .default_scan_roots(Lv2HostPlatform::MacOs)
        .into_iter()
        .map(|root| root.root)
        .collect();
    assert!(macos.contains(&"~/Library/Audio/Plug-Ins/LV2".to_string()));
    assert!(macos.contains(&"/Library/Audio/Plug-Ins/LV2".to_string()));
    let linux: Vec<_> = adapter
        .default_scan_roots(Lv2HostPlatform::Linux)
        .into_iter()
        .map(|root| root.root)
        .collect();
    assert!(linux.contains(&"~/.lv2".to_string()));
    assert!(linux.contains(&"/usr/lib/lv2".to_string()));
    assert!(linux.contains(&"/usr/local/lib/lv2".to_string()));
}
