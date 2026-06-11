use super::*;

#[test]
fn runtime_emits_events_to_subscribers() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let sink = Box::new(TestSink::default());
    runtime.subscribe(sink);

    runtime
        .handshake(HandshakeRequest {
            client_version: "runtime-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .unwrap();
    runtime.start().unwrap();
    runtime.set_active_output_device("coreaudio:default");
    runtime.set_active_plugin_sandboxes(2);

    let readiness = runtime.get_readiness();
    assert_eq!(readiness, RuntimeReadiness::Ready);
    assert_eq!(
        runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
        2
    );
}

#[test]
fn runtime_records_plugin_fault_events() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        crate::interfaces::PluginFaultKind::ProtocolViolation,
        "epoch mismatch",
        Some(3),
    );

    assert_eq!(
        runtime.get_diagnostics_snapshot().active_plugin_sandboxes,
        0
    );
}

#[test]
fn runtime_tracks_plugin_lifecycle_recovery_and_quarantine_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    let recorder = RuntimeEventRecorder::default();
    runtime.subscribe(Box::new(recorder.clone()));
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:lifecycle-receipts".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "plugin-a".into(),
                execution_class: GraphNodeExecutionClass::PluginBacked,
                latency_samples: 24,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
            }],
        })
        .expect("apply lifecycle receipt graph");
    runtime
        .apply_graph_contract_projection(GraphContractProjection {
            graph_id: "graph:runtime:lifecycle-receipts".into(),
            contract_count: 1,
            nodes: vec![GraphNodeContractProjection {
                node_id: "plugin-a".into(),
                buffer_contract: GraphNodeBufferContractProjection::default(),
                topology: GraphNodeTopologyProjection {
                    role: Some(GraphNodeTopologyRole::TrackLane),
                    track_lane_id: Some("track:lead".into()),
                    bus_group_id: Some("mix:tracks".into()),
                    console_group_id: None,
                    send_return_id: None,
                },
            }],
        })
        .expect("apply lifecycle receipt contracts");
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:lifecycle-receipts".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin-a".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .expect("apply lifecycle receipt binding");

    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::SandboxEnsured,
        None,
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(1),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );
    runtime.set_active_plugin_sandboxes(1);

    let ready = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(ready.active_sandbox_count, 1);
    assert_eq!(ready.ready_sandbox_count, 1);
    assert_eq!(ready.sandboxes[0].state, RuntimePluginLifecycleState::Ready);
    assert_eq!(
        ready.sandboxes[0].active_lease_id.as_deref(),
        Some("lease-a")
    );

    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        crate::interfaces::PluginFaultKind::Crash,
        "sandbox crashed during process block",
        Some(2),
    );
    runtime.set_active_plugin_sandboxes(0);

    let faulted = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(faulted.faulted_sandbox_count, 1);
    assert_eq!(
        faulted.sandboxes[0].state,
        RuntimePluginLifecycleState::Faulted
    );
    assert_eq!(
        faulted.sandboxes[0].last_fault_detail.as_deref(),
        Some("sandbox crashed during process block")
    );

    runtime.record_recovery_cycle(
        "sandbox-a",
        RecoveryRestartIntent::CrashRecovery,
        StopReason::DegradedModeRecovery,
        Some(3),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::SandboxRestarted,
        Some(3),
    );
    runtime.record_plugin_sandbox_lifecycle(
        "sandbox-a",
        PluginSandboxLifecycleStage::InstancePrepared,
        Some(4),
    );
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-b",
        "region-b",
        PluginSandboxTransportStage::Attached,
        Some(4),
        None,
    );
    runtime.set_active_plugin_sandboxes(1);

    let recovered = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(recovered.ready_sandbox_count, 1);
    assert_eq!(
        recovered.sandboxes[0].state,
        RuntimePluginLifecycleState::Ready
    );
    assert_eq!(recovered.sandboxes[0].restart_count, 1);
    assert_eq!(recovered.sandboxes[0].recovery_count, 1);
    assert_eq!(
        recovered.sandboxes[0].active_lease_id.as_deref(),
        Some("lease-b")
    );

    runtime.record_plugin_sandbox_fault(
        "sandbox-a",
        crate::interfaces::PluginFaultKind::Timeout,
        "sandbox missed heartbeat twice",
        Some(5),
    );

    let quarantined = runtime.get_plugin_lifecycle_snapshot();
    assert_eq!(quarantined.quarantined_sandbox_count, 1);
    assert_eq!(
        quarantined.sandboxes[0].state,
        RuntimePluginLifecycleState::Quarantined
    );
    assert_eq!(quarantined.sandboxes[0].fault_count, 2);

    let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(&runtime, &recorder);
    let profiling = supervisor.profiling_receipt();
    let soak = supervisor.soak_receipt();
    assert_eq!(profiling.plugin_chain_stage_count, 1);
    assert_eq!(profiling.plugin_chain_degraded_stage_count, 1);
    assert_eq!(soak.plugin_fault_count, 2);
    assert_eq!(soak.recovery_event_count, 1);
    assert_eq!(soak.plugin_quarantined_sandbox_count, 1);
    assert_eq!(soak.recall_stage_count, 1);
    assert_eq!(soak.recovered_recall_stage_count, 0);
    assert_eq!(soak.unavailable_recall_stage_count, 1);
    assert_eq!(
        soak.last_recovery_intent,
        Some(RecoveryRestartIntent::CrashRecovery)
    );
}
