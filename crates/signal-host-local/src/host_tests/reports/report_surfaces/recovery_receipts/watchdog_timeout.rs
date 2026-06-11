use super::super::super::super::*;

#[test]
fn local_host_shared_report_derives_profiling_and_soak_receipts() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_with_mixed_watchdog_soak()
        .expect("mixed watchdog soak boot");
    let report = host.host_supervisor_report();
    let profiling = report.profiling_receipt();
    let soak = report.soak_receipt();

    assert_eq!(profiling.sample_rate_hz, 48_000);
    assert_eq!(profiling.block_size, 512);
    assert_eq!(profiling.host_callback_count, Some(14));
    assert_eq!(profiling.runtime_xrun_count, 1);
    assert_eq!(profiling.host_backend_xrun_count, Some(0));
    assert_eq!(profiling.host_device_loss_count, Some(0));
    assert!(profiling.host_graph_latency_ms.unwrap_or_default() > 0.4);
    assert!(profiling.runtime_graph_latency_ms > 0.0);
    assert_eq!(
        profiling.fault_diagnostic_receipt.primary_family,
        Some(signal_runtime::RuntimeFaultDiagnosticFamily::DeferredWorkPressure)
    );
    assert!(profiling
        .fault_diagnostic_receipt
        .contributions
        .iter()
        .any(|entry| {
            entry.family == signal_runtime::RuntimeFaultDiagnosticFamily::CallbackPressure
                && entry.authority
                    == signal_runtime::RuntimeFaultDiagnosticAuthority::HostAdvisory
        }));

    assert_eq!(soak.watchdog_restart_count, 3);
    assert!(soak.safe_mode_enabled);
    assert_eq!(
        soak.last_recovery_intent,
        Some(RecoveryRestartIntent::WatchdogRecovery)
    );
    assert_eq!(
        soak.last_stop_reason,
        Some(StopReason::DegradedModeRecovery)
    );
    assert_eq!(soak.event_stream_count, report.events.len());
    assert!(soak.recovery_event_count >= 3);
    assert!(soak.heartbeat_event_count >= 4);
}

#[test]
fn local_host_shared_report_tracks_timeout_recovery_without_losing_topology() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_with_timeout_recovery()
        .expect("timeout recovery local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.host_io.audio_pump.stream_state,
        RuntimeHostAudioStreamState::Running
    );
    assert!(report.observation.host_io.runtime_graph_id_matches_pump);
    assert_eq!(report.observation.observation.degradation_summary.xrun_count, 1);
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .track_lane_node_count,
        2
    );
    assert_eq!(
        report.observation.observation.execution_topology_summary.bus_node_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .console_node_count,
        1
    );
}
