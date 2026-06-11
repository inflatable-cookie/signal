use super::super::*;

#[test]
fn runtime_supervisor_report_derives_profiling_and_soak_receipts() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:profiling-receipt");
    runtime.set_cpu_load_percent(7.25);
    runtime.set_graph_latency_ms(3.5);
    runtime.start().expect("start runtime");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1),
        )
        .expect("process profiling block");

    let mut recorder = RuntimeEventRecorder::default();
    RuntimeEventSink::push(
        &mut recorder,
        RuntimeEvent::RecoveryCycle {
            sandbox_id: "sandbox-profile".into(),
            intent: RecoveryRestartIntent::WatchdogRecovery,
            stop_reason: StopReason::DegradedModeRecovery,
            processing_epoch: Some(1),
        },
    );

    let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(&runtime, &recorder);
    let profiling = supervisor.profiling_receipt();
    let soak = supervisor.soak_receipt();
    let diagnostics = &supervisor.observation.diagnostics_snapshot;

    assert_eq!(profiling.sample_rate_hz, 48_000);
    assert_eq!(profiling.block_size, 256);
    assert_eq!(profiling.engine_processed_blocks, 1);
    assert_eq!(profiling.engine_node_count, 2);
    assert_eq!(profiling.engine_stage_count, 2);
    assert!(!profiling.readiness_degraded);
    assert!(!profiling.transport_gate_active);
    assert!(!profiling.plugin_gate_active);
    assert_eq!(profiling.plugin_chain_stage_count, 0);
    assert_eq!(profiling.plugin_chain_degraded_stage_count, 0);
    assert_eq!(
        profiling.runtime_cpu_load_percent,
        diagnostics.cpu_load_percent
    );
    assert_eq!(
        profiling.runtime_graph_latency_ms,
        diagnostics.graph_latency_ms
    );
    assert_eq!(profiling.host_callback_count, None);

    assert_eq!(soak.event_stream_count, 1);
    assert!(!soak.readiness_degraded);
    assert_eq!(soak.recovery_event_count, 1);
    assert_eq!(soak.plugin_quarantined_sandbox_count, 0);
    assert_eq!(soak.recall_stage_count, 0);
    assert_eq!(
        soak.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(soak.last_stop_reason, None);
}
