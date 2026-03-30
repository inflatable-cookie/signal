use super::super::*;

pub(super) fn assert_server_linux_parity_report(report: &signal_runtime::RuntimeSupervisorReport) {
    let discovery = &report.observation.plugin_discovery_snapshot;

    let clap = discovery
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Clap)
        .expect("server linux clap parity should be exported");
    assert_eq!(clap.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(clap.linux_supported);
    assert_eq!(
        clap.linux_preferred_sandbox_outcome,
        Some(RuntimePluginIsolationOutcome::IsolatedSandbox)
    );
    assert!(clap.linux_strict_sandbox_default);

    let vst3 = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Vst3)
        .expect("server linux vst3 parity should be exported");
    assert_eq!(vst3.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(vst3.linux_supported);
    assert_eq!(vst3.in_process_sandbox_count, 1);
    assert_eq!(vst3.restarting_sandbox_count, 1);
    assert_eq!(vst3.rebindable_sandbox_count, 1);

    let lv2 = report
        .observation
        .plugin_lifecycle_snapshot
        .parity_coverage
        .iter()
        .find(|record| record.format == PluginFormat::Lv2)
        .expect("server linux lv2 parity should be exported");
    assert_eq!(lv2.linux_parity_band, RuntimePluginParityBand::Portable);
    assert!(lv2.linux_supported);
    assert_eq!(lv2.faulted_sandbox_count, 1);
    assert_eq!(
        lv2.unsupported_platforms,
        vec![
            RuntimePluginHostPlatform::MacOs,
            RuntimePluginHostPlatform::Windows,
        ]
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"linux_parity_band\":\"Portable\""));
    assert!(rendered.contains("\"linux_supported\":true"));
    assert!(rendered.contains("\"linux_preferred_sandbox_outcome\":\"IsolatedSandbox\""));
    assert!(rendered.contains("\"restarting_sandbox_count\":1"));
    assert!(rendered.contains("\"faulted_sandbox_count\":1"));
}
