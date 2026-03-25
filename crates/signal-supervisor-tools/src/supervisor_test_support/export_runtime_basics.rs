use crate::{render_supervisor_export_json, HostProfile, Scenario};
use signal_runtime::{
    RuntimeConfig, RuntimeLifecycleApi, RuntimeOfflineRenderPurgeRequest, RuntimeSupervisorReport,
    SafeModeRequest, SignalRuntime,
};

pub(crate) fn verify_export_json_is_versioned() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
    let profiling = report.profiling_receipt();
    let soak = report.soak_receipt();
    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Default,
        "{}".into(),
        &profiling,
        &soak,
        &report,
    );
    assert!(export.contains("\"schema\":\"signal.supervisor.export\""));
    assert!(export.contains("\"schema_version\":1"));
    assert!(export.contains("\"profiling_receipt\":{"));
    assert!(export.contains("\"soak_receipt\":{"));
}

pub(crate) fn verify_export_json_carries_last_deferred_service_receipt() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");
    let purge_receipt = runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: "purge:export-proof".into(),
            artifact_root_path: Some("/tmp/nonexistent-artifacts".into()),
            report_path: Some("/tmp/nonexistent-report.json".into()),
        })
        .expect("safe mode should defer purge export proof");
    assert!(!purge_receipt.purged_report);
    assert!(!purge_receipt.purged_artifact_root);

    let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
    let export = render_supervisor_export_json(
        HostProfile::Local,
        Scenario::Default,
        "{}".into(),
        &report.profiling_receipt(),
        &report.soak_receipt(),
        &report,
    );

    assert!(export.contains("\"last_deferred_service\":{"));
    assert!(export.contains("\"work_class\":\"OfflineRenderPurge\""));
    assert!(export.contains("\"decision\":\"Defer\""));
    assert!(export.contains("\"reason\":\"SafeMode\""));
}
