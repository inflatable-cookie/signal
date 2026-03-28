use super::super::super::*;

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
    assert!(profiling.render_json().contains("\"host_callback_count\":14"));
    assert!(profiling
        .render_json()
        .contains("\"fault_diagnostic_receipt\":{"));

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
    assert!(soak.render_json().contains("\"watchdog_restart_count\":3"));
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
    assert!(report.render_json().contains("\"node_id\":\"plugin-insert\""));
    assert!(report
        .render_json()
        .contains("\"plugin_sandbox_id\":\"local-default-sandbox\""));
    assert!(report.render_json().contains("\"track_lane_id\":\"track:lead\""));
    assert!(report.render_json().contains("\"bus_group_id\":\"mix:tracks\""));
    assert!(report.render_compact().contains("xruns=1"));
    assert!(report
        .render_json()
        .contains("\"runtime_graph_id_matches_pump\":true"));
}

#[test]
fn local_host_shared_report_tracks_device_loss_recovery() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host
        .boot_with_device_loss_recovery()
        .expect("device loss recovery local host boot");
    let supervisor = host.supervisor_report();
    let report = host.host_supervisor_report();

    assert_eq!(
        summary.execution.last_stop_reason,
        Some(StopReason::DeviceReconfigure)
    );
    assert_eq!(
        report.observation.host_io.audio_pump.stream_state,
        RuntimeHostAudioStreamState::Running
    );
    assert_eq!(
        report.observation.host_io.hardware.backend_health,
        BackendHealth::Healthy
    );
    assert_eq!(report.observation.host_io.hardware.device_loss_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_attempt_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_failure_count, 0);
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.state,
        signal_runtime::RuntimeDeviceSupervisionState::Stable
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.restart_state,
        signal_runtime::RuntimeDeviceRestartState::Recovered
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.fault_boundary,
        signal_runtime::RuntimeDeviceFaultBoundaryState::Clear
    );
    assert_eq!(
        report
            .observation
            .observation
            .device_supervision_snapshot
            .restart_attempt_count,
        Some(1)
    );
    assert_eq!(report.observation.host_io.latency.output_latency_samples, 512);
    assert!(report.observation.host_io.runtime_graph_id_matches_pump);
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .track_lane_node_count,
        2
    );
    assert!(report
        .render_compact()
        .contains("host_backend_device_losses=1"));
    assert!(report.render_json().contains("\"restart_attempt_count\":1"));
    assert!(report
        .render_json()
        .contains("\"device_supervision_snapshot\":{"));
    assert!(report.render_json().contains("\"restart_state\":\"Recovered\""));
}

#[test]
fn local_host_shared_report_tracks_device_loss_restart_failure() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let error = host
        .boot_with_device_loss_restart_failure()
        .expect_err("device loss restart should fail");
    let supervisor = host.supervisor_report();
    let report = host.host_supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::HardwareFailure);
    assert_eq!(
        report.observation.host_io.audio_pump.stream_state,
        RuntimeHostAudioStreamState::Faulted
    );
    assert_eq!(
        report.observation.host_io.hardware.backend_health,
        BackendHealth::Degraded
    );
    assert_eq!(report.observation.host_io.hardware.device_loss_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_attempt_count, 1);
    assert_eq!(report.observation.host_io.hardware.restart_failure_count, 1);
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.state,
        signal_runtime::RuntimeDeviceSupervisionState::Exhausted
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.restart_state,
        signal_runtime::RuntimeDeviceRestartState::Exhausted
    );
    assert_eq!(
        supervisor.observation.device_supervision_snapshot.fault_boundary,
        signal_runtime::RuntimeDeviceFaultBoundaryState::Exhausted
    );
    assert_eq!(
        report.observation.host_io.clocking.clock_source,
        RuntimeHostClockSource::Internal
    );
    assert_eq!(
        report.observation.host_io.clocking.clock_domain,
        RuntimeHostClockDomain::Degraded
    );
    assert_eq!(
        report.observation.host_io.clocking.fallback_state,
        RuntimeHostClockFallbackState::RecoveryConstrained
    );
    assert_eq!(
        report.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::Stable
    );
    assert_eq!(
        report.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Resyncing
    );
    assert_eq!(
        report.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Faulted
    );
    assert_eq!(
        report.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::NotApplicable
    );
    assert_eq!(
        report.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::OutputOnly
    );
    assert!(!report.observation.host_io.clocking.partial_availability);
    assert_eq!(
        report.observation.observation.external_io_snapshot.monitoring_state,
        signal_runtime::RuntimeExternalIoMonitoringState::Faulted
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.loopback_state,
        signal_runtime::RuntimeExternalIoLoopbackState::Faulted
    );
    assert!(!report.observation.host_io.clocking.crossing_required);
    assert!(!report.observation.host_io.runtime_graph_id_matches_pump);
    assert_eq!(
        report.observation.observation.control_snapshot.last_stop_reason,
        Some(StopReason::DeviceReconfigure)
    );
    assert!(report
        .render_compact()
        .contains("host_backend_restart_failures=1"));
    assert!(report.render_json().contains("\"device_loss_count\":1"));
    assert!(report
        .render_json()
        .contains("\"device_supervision_snapshot\":{"));
    assert!(report.render_json().contains("\"fault_boundary\":\"Exhausted\""));
    assert!(report.render_json().contains("\"clock_domain\":\"Degraded\""));
    assert!(report
        .render_json()
        .contains("\"fallback_state\":\"RecoveryConstrained\""));
    assert!(report.render_json().contains("\"transition_state\":\"Stable\""));
    assert!(report.render_json().contains("\"drift_state\":\"Resyncing\""));
    assert!(report.render_json().contains("\"discontinuity_state\":\"Faulted\""));
}
