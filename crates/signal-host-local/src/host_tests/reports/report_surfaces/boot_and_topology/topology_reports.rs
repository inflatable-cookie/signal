use super::super::super::super::*;

#[test]
fn local_host_executes_track_bus_output_topology_through_audio_pump() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = host.boot_default().expect("default local host boot");
    let supervisor = host.supervisor_report();
    let topology = &supervisor.observation.execution_topology_summary;

    assert_eq!(summary.audio_pump.stream_state, LocalAudioStreamState::Running);
    assert_eq!(summary.audio_pump.callback_count, 8);
    assert_local_plugin_topology(&summary);
    assert_eq!(summary.topology, *topology);
    assert!(supervisor
        .render_multiline()
        .contains("execution_topology_summary_node_3=output-main"));
}

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
    assert_eq!(
        report.observation.host_io.hardware.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::NotLinux
    );
    assert_eq!(
        report.observation.host_io.hardware.linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
    assert_eq!(report.observation.host_io.hardware.device_id, "coreaudio:default-output");
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
    assert_eq!(
        report.observation.host_io.clocking.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.host_io.clocking.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report.observation.host_io.clocking.linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
    assert!(!report.observation.host_io.clocking.partial_availability);
    assert_eq!(
        report.observation.observation.external_io_snapshot.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::NotLinux
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.linux_clocking_parity,
        signal_runtime::RuntimeLinuxAudioBackendClockingParityBand::Unsupported
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.linux_duplex_parity,
        signal_runtime::RuntimeLinuxAudioBackendDuplexParityState::Unsupported
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.linux_endpoint_topology_parity,
        signal_runtime::RuntimeLinuxAudioBackendEndpointTopologyParityState::Unsupported
    );
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
    assert_eq!(report.observation.observation.execution_topology_summary.node_count, 4);
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .track_lane_node_count,
        2
    );
    assert_eq!(
        report
            .observation
            .observation
            .execution_topology_summary
            .bus_node_count,
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
    assert_eq!(
        report.observation.observation.plugin_discovery_snapshot.scan_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .format_filtered_scan_count,
        1
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .discovered_type_count,
        2
    );
    assert_eq!(
        report
            .observation
            .observation
            .plugin_discovery_snapshot
            .last_scan
            .as_ref()
            .map(|scan| scan.discovered_type_count),
        Some(2)
    );
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:clap:default"
            && plugin.format == PluginFormat::Clap
            && plugin.state_contract.supports_snapshot));
    assert!(report
        .observation
        .observation
        .plugin_discovery_snapshot
        .discovered_types
        .iter()
        .any(|plugin| plugin.plugin_type_id == "plugin:clap:sandbox"
            && plugin.features.contains(&signal_plugin::PluginFeature::Utility)
            && plugin.processing_contract.produces_midi));
    assert_eq!(
        report
            .observation
            .observation
            .plugin_lifecycle_snapshot
            .sandboxes
            .first()
            .and_then(|sandbox| sandbox.plugin_format),
        Some(PluginFormat::Clap)
    );
    assert!(report.render_json().contains("\"node_id\":\"plugin-insert\""));
    assert!(report
        .render_json()
        .contains("\"plugin_sandbox_id\":\"local-default-sandbox\""));
    assert!(report.render_json().contains("\"input_bus_id\":\"bus:track:lead\""));
    assert!(report.render_json().contains("\"output_bus_id\":\"bus:mix:tracks\""));
    assert!(report
        .render_compact()
        .contains("host_audio_graph_matches_runtime=true"));
    assert!(report.render_compact().contains("metering_snapshot_routes=1/2/0/1"));
    assert!(report.render_multiline().contains("host_backend=coreaudio"));
    assert!(report.render_json().contains("\"device_loss_count\":0"));
    assert!(report.render_json().contains("\"clock_source\":\"Internal\""));
    assert!(report.render_json().contains("\"clock_domain\":\"SameClock\""));
    assert!(report.render_json().contains("\"fallback_state\":\"Direct\""));
    assert!(report.render_json().contains("\"transition_state\":\"Stable\""));
    assert!(report.render_json().contains("\"drift_state\":\"Stable\""));
    assert!(report.render_json().contains("\"discontinuity_state\":\"Continuous\""));
    assert!(report.render_json().contains("\"endpoint_topology\":\"OutputOnly\""));
    assert!(report
        .render_json()
        .contains("\"estimated_output_latency_samples\":536"));
    assert!(report
        .render_json()
        .contains("\"metering_snapshot\":{\"meter_count\":"));
}
