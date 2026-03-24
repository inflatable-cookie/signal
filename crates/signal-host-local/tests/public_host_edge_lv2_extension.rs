use signal_host_local::LocalRuntimeHost;
use signal_runtime::{RuntimeConfig, SignalRuntime};

#[test]
fn local_shared_host_edge_exports_runtime_lv2_extension_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local lv2 extension default boot should succeed");
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.lv2_extension_snapshot.plugin_type_count,
        0
    );
    assert_eq!(report.observation.lv2_extension_snapshot.sandbox_count, 0);
    assert!(report.observation.lv2_extension_snapshot.records.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"lv2_extension_snapshot\":{"));
    assert!(rendered.contains("\"plugin_type_count\":0"));
}
