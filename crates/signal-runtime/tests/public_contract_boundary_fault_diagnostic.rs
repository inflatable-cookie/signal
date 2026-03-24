use signal_runtime::{
    RuntimeConfig, RuntimeError, RuntimeErrorKind, RuntimeEventRecorder, RuntimeInterruptionClass,
    RuntimeLifecycleApi, RuntimeObservationReport, RuntimeOfflineRenderRequest, SignalRuntime,
};

#[test]
fn public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(signal_runtime::SafeModeRequest { enabled: true })
        .expect("public fault diagnostic safe mode should enable");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:public:fault-diagnostic".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("public fault diagnostic render queue should defer");
    assert_eq!(deferred.orchestration.deferred_work_item_count, 1);

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = signal_runtime::RuntimeSupervisorReport::capture(&runtime, &recorder);
    let deferred_contribution = observation
        .fault_diagnostic_receipt
        .contributions
        .iter()
        .find(|entry| {
            entry.family == signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure
        })
        .expect("public fault diagnostic deferred-work contribution should be present");

    assert_eq!(
        observation.fault_diagnostic_receipt.primary_family,
        Some(signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert_eq!(
        observation.fault_diagnostic_receipt.interruption_class,
        RuntimeInterruptionClass::Recoverable
    );
    assert!(deferred_contribution.active);
    assert!(deferred_contribution.event_count >= 1);
    assert_eq!(
        supervisor
            .observation
            .fault_diagnostic_receipt
            .primary_family,
        observation.fault_diagnostic_receipt.primary_family
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(observation_json.contains("\"primary_family\":\"DeferredWorkPressure\""));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"fault_diagnostic_receipt\":{"));
    assert!(supervisor_json.contains("\"primary_family\":\"DeferredWorkPressure\""));

    let failure = RuntimeError::new(
        RuntimeErrorKind::HardwareFailure,
        "public runtime diagnostic sentinel",
    );
    assert_eq!(failure.kind, RuntimeErrorKind::HardwareFailure);
}
