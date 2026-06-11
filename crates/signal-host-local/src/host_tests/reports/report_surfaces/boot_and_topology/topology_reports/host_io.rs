use super::super::super::super::super::*;

#[test]
fn local_host_shared_report_surfaces_topology_aware_host_io() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(report.observation.host_io.hardware.backend_name, "coreaudio");
    assert_eq!(
        report.observation.host_io.hardware.backend_identity,
        signal_hardware::HardwareBackendIdentity::CoreAudio
    );
    assert!(report.observation.host_io.hardware.device_id.starts_with("coreaudio:"));
    assert_eq!(report.observation.host_io.hardware.sample_rate, 48_000);
    assert_eq!(report.observation.host_io.hardware.buffer_size, 512);
    assert_eq!(report.observation.host_io.hardware.input_channels, 0);
    assert_eq!(report.observation.host_io.hardware.output_channels, 2);
    assert_eq!(
        report
            .observation
            .observation
            .external_io_snapshot
            .io_layout
            .output_layout
            .canonical_layout,
        Some(signal_runtime::RuntimeCanonicalChannelLayout::Stereo)
    );
    assert_eq!(
        report
            .observation
            .observation
            .external_io_snapshot
            .io_layout
            .output_bus_intent,
        signal_runtime::RuntimeBusIntent::HardwareOutput
    );
    assert_eq!(
        report.observation.host_io.clocking.clock_source,
        RuntimeHostClockSource::Internal
    );
    assert_eq!(
        report.observation.host_io.clocking.clock_domain,
        RuntimeHostClockDomain::SameClock
    );
    assert_eq!(
        report.observation.host_io.clocking.fallback_state,
        RuntimeHostClockFallbackState::Direct
    );
    assert_eq!(
        report.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::Stable
    );
    assert_eq!(
        report.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Stable
    );
    assert_eq!(
        report.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Continuous
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
        report.observation.observation.external_io_snapshot.primary_role,
        signal_runtime::RuntimeExternalIoPrimaryRole::ProgramOutput
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.monitoring_state,
        signal_runtime::RuntimeExternalIoMonitoringState::Direct
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.monitoring_tap_point,
        signal_runtime::RuntimeExternalIoMonitoringTapPoint::PostHardwareOutput
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.loopback_state,
        signal_runtime::RuntimeExternalIoLoopbackState::Unavailable
    );
    assert!(!report.observation.host_io.clocking.crossing_required);
    assert_eq!(report.observation.host_io.clocking.processing_sample_rate_hz, 48_000);
    assert_eq!(report.observation.host_io.clocking.hardware_sample_rate_hz, 48_000);
    assert_eq!(
        report.observation.host_io.clocking.ownership,
        signal_runtime::RuntimeHostLifecycleOwnership::HostDrivenCallback
    );
    assert_eq!(
        report.observation.host_io.clocking.restart_policy,
        signal_runtime::RuntimeHostRestartPolicy::HostMustRestart
    );
    assert!((report.observation.host_io.clocking.callback_interval_ms - 10.666667).abs() < 0.001);
    assert_eq!(report.observation.host_io.latency.output_latency_samples, 512);
    assert_eq!(report.observation.host_io.latency.graph_latency_samples, 24);
    assert_eq!(
        report.observation.host_io.latency.estimated_output_latency_samples,
        536
    );
    assert_eq!(
        report.observation.host_io.audio_pump.stream_state,
        RuntimeHostAudioStreamState::Running
    );
    assert_eq!(report.observation.host_io.audio_pump.callback_count, 8);
    assert!(report.observation.host_io.runtime_graph_id_matches_pump);
}
