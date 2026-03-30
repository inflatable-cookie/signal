use super::*;

#[test]
fn server_shared_host_edge_exports_runtime_deferred_work_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-deferred-work".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("server host-edge deferred-work handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("server host-edge deferred-work configure should succeed");
    runtime
        .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
            request_id: String::new(),
            artifact_root_path: None,
            report_path: None,
        })
        .expect_err("empty purge request id should record terminal deferred-work policy");

    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let receipt = report
        .observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("server host-edge report should expose deferred-work policy receipt");
    assert_eq!(receipt.decision, RuntimeDeferredServiceDecision::Abort);
    assert_eq!(receipt.reason, RuntimeDeferredServiceReason::InvalidRequest);
    assert_eq!(
        receipt.priority_band,
        RuntimeDeferredServicePriorityBand::Maintenance
    );
    assert_eq!(receipt.blocking_priority_band, None);
    assert_eq!(
        receipt.cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(receipt.cancelled_work_item_count, 1);

    let performance = report.performance_snapshot();
    assert_eq!(
        performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Abort)
    );
    assert_eq!(
        performance.background_service_reason,
        Some(RuntimeDeferredServiceReason::InvalidRequest)
    );
    assert_eq!(
        performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::Maintenance)
    );
    assert_eq!(
        performance.background_service_cancellation_cause,
        Some(RuntimeDeferredServiceCancellationCause::InvalidRequest)
    );
    assert_eq!(performance.background_service_cancelled_work_item_count, 1);

    let rendered = report.render_json();
    assert!(rendered.contains("\"last_deferred_service\":{"));
    assert!(rendered.contains("\"priority_band\":\"Maintenance\""));
    assert!(rendered.contains("\"cancellation_cause\":\"InvalidRequest\""));
}
