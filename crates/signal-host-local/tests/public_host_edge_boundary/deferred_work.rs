use super::*;

#[test]
fn local_shared_host_edge_exports_runtime_deferred_work_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-deferred-work".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("local host-edge deferred-work handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("local host-edge deferred-work configure should succeed");
    apply_public_render_graph(&mut runtime, "graph:host-local:deferred-work");
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode for local deferred-work policy proof");
    runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:host-local:deferred-work".into(),
            timeline_start_samples: 0,
            duration_samples: 96,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer local host-edge deferred work");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let receipt = report
        .observation
        .last_deferred_service_receipt
        .as_ref()
        .expect("local host-edge report should expose deferred-work policy receipt");
    assert_eq!(receipt.decision, RuntimeDeferredServiceDecision::Defer);
    assert_eq!(receipt.reason, RuntimeDeferredServiceReason::SafeMode);
    assert_eq!(
        receipt.priority_band,
        RuntimeDeferredServicePriorityBand::UserVisible
    );
    assert_eq!(
        receipt.blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        receipt.backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(receipt.starvation_risk);
    assert_eq!(receipt.starved_work_item_count, 1);

    let performance = report.performance_snapshot();
    assert_eq!(
        performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Defer)
    );
    assert_eq!(
        performance.background_service_reason,
        Some(RuntimeDeferredServiceReason::SafeMode)
    );

}
