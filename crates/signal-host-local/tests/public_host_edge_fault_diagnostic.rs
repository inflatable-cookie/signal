use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeFaultDiagnosticFamily, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeOfflineRenderRequest, SafeModeRequest, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_fault_diagnostic_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("local host-edge fault diagnostic safe mode should enable");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:host-local:fault-diagnostic".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("local host-edge fault diagnostic queue should defer");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.fault_diagnostic_receipt.primary_family,
        Some(RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(
        report
            .observation
            .fault_diagnostic_receipt
            .interruption_class,
        RuntimeInterruptionClass::Recoverable
    );
    assert!(report
        .observation
        .fault_diagnostic_receipt
        .contributions
        .iter()
        .any(|entry| {
            entry.family == RuntimeFaultDiagnosticFamily::DeferredWorkPressure && entry.active
        }));

}
