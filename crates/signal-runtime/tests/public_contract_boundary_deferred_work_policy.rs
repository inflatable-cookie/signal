#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;

use public_contract_boundary_graph_foundation_support::apply_public_render_graph;
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest,
    RuntimeDeferredServiceBackpressureSource, RuntimeDeferredServiceCancellationCause,
    RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
    RuntimeDeferredServiceReason, RuntimeEventRecorder, RuntimeLifecycleApi,
    RuntimeObservationReport, RuntimeOfflineRenderPurgeRequest, RuntimeOfflineRenderRequest,
    RuntimeSupervisorReport, SafeModeRequest, SignalRuntime,
};

#[test]
fn public_runtime_deferred_work_policy_boundary_reports_runtime_owned_scheduler_receipts() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-deferred-work-policy".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public deferred-work handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public deferred-work configure should succeed");
    apply_public_render_graph(&mut runtime, "graph:public:deferred-work-policy");

    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode for deferred-work proof");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:public:deferred-work:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 96,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer the public render queue request");
    let deferred_observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let deferred_supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let deferred_receipt = deferred_observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("deferred observation should carry a scheduler-policy receipt");
    assert_eq!(
        deferred_receipt.decision,
        RuntimeDeferredServiceDecision::Defer
    );
    assert_eq!(
        deferred_receipt.reason,
        RuntimeDeferredServiceReason::SafeMode
    );
    assert_eq!(
        deferred_receipt.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(
        deferred_receipt.blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        deferred_receipt.backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(deferred_receipt.starvation_risk);
    assert_eq!(deferred_receipt.starved_work_item_count, 1);
    assert_eq!(deferred_receipt.cancellation_cause, None);

    let deferred_performance = deferred_supervisor.performance_snapshot();
    assert_eq!(
        deferred_performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Defer)
    );
    assert_eq!(
        deferred_performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::UserVisible)
    );
    assert_eq!(
        deferred_performance.background_service_blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        deferred_performance.background_service_backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(deferred_performance.background_service_starvation_risk);
    assert_eq!(
        deferred_performance.background_service_starved_work_item_count,
        1
    );

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("disable safe mode before abort proof");
    let abort_error = runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: String::new(),
            artifact_root_path: None,
            report_path: None,
        })
        .expect_err("empty purge request id should record a typed cancellation policy");
    assert!(abort_error.message.contains("requires a request id"));

    let abort_observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let abort_supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let abort_receipt = abort_observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("abort observation should carry a scheduler-policy receipt");
    assert_eq!(
        abort_receipt.decision,
        RuntimeDeferredServiceDecision::Abort
    );
    assert_eq!(
        abort_receipt.reason,
        RuntimeDeferredServiceReason::InvalidRequest
    );
    assert_eq!(
        abort_receipt.priority_band,
        RuntimeDeferredServicePriorityBand::Maintenance
    );
    assert_eq!(abort_receipt.blocking_priority_band, None);
    assert_eq!(abort_receipt.backpressure_source, None);
    assert!(!abort_receipt.starvation_risk);
    assert_eq!(
        abort_receipt.cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(abort_receipt.cancelled_work_item_count, 1);

    let abort_performance = abort_supervisor.performance_snapshot();
    assert_eq!(
        abort_performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Abort)
    );
    assert_eq!(
        abort_performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::Maintenance)
    );
    assert_eq!(
        abort_performance.background_service_cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(
        abort_performance.background_service_cancelled_work_item_count,
        1
    );

    let trace = RuntimeObservationReport::build_performance_trace_receipt(&[
        deferred_observation.clone(),
        abort_observation.clone(),
    ]);
    assert_eq!(trace.observation_count, 2);
    assert_eq!(trace.background_service_defer_count, 1);
    assert_eq!(trace.background_service_abort_count, 1);
    assert_eq!(trace.background_starvation_observation_count, 1);
    assert_eq!(trace.peak_background_starved_work_item_count, 1);
    assert_eq!(trace.background_cancellation_observation_count, 1);
    assert_eq!(trace.peak_background_cancelled_work_item_count, 1);
    assert_eq!(trace.background_realtime_backpressure_observation_count, 0);
    assert_eq!(trace.background_recovery_backpressure_observation_count, 1);

    let deferred_json = deferred_supervisor.render_json();
    assert!(deferred_json.contains("\"last_deferred_service\":{"));
    assert!(deferred_json.contains("\"backpressure_source\":\"SafeMode\""));
    assert!(deferred_json.contains("\"starvation_risk\":true"));

    let abort_json = abort_supervisor.render_json();
    assert!(abort_json.contains("\"cancellation_cause\":\"InvalidRequest\""));
    assert!(abort_json.contains("\"cancelled_work_item_count\":1"));

    let trace_json = trace.render_json();
    assert!(trace_json.contains("\"background_cancellation_observation_count\":1"));
    assert!(trace_json.contains("\"peak_background_cancelled_work_item_count\":1"));
}
