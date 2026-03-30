use super::super::super::super::super::*;

#[test]
fn local_host_shared_report_surfaces_aggregate_clock_domain() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    runtime
        .handshake(HandshakeRequest {
            client_version: "host-local-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })
        .expect("handshake");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("configure");
    let mut host = LocalRuntimeHost::new(runtime);
    let initial = host.host_supervisor_report();
    assert_eq!(
        initial.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::InitialObservation
    );
    host.active_output_stream = Some(HardwareStreamConfig {
        device: AudioDeviceDescriptor {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio",
            device_id: "coreaudio:aggregate-output".into(),
            name: "CoreAudio Aggregate Output".into(),
            default_input: false,
            default_output: true,
            max_input_channels: 0,
            max_output_channels: 2,
            nominal_sample_rate: SampleRate(48_000),
            preferred_buffer_sizes: vec![256],
        },
        direction: AudioStreamDirection::Output,
        sample_rate: SampleRate(48_000),
        buffer_size: 256,
        input_channels: 0,
        output_channels: 2,
        sample_format: AudioSampleFormat::F32,
        interleaved: true,
        clock_source: HardwareClockSource::DigitalInput,
        clock_topology: HardwareClockTopology::Aggregate,
        lifecycle: HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::HostMustRestart,
        },
        latency: HardwareLatencyProfile::output_only(256),
        simulated: false,
    });

    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.host_io.clocking.clock_domain,
        RuntimeHostClockDomain::Aggregate
    );
    assert_eq!(
        report.observation.host_io.clocking.fallback_state,
        RuntimeHostClockFallbackState::Direct
    );
    assert_eq!(
        report.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::EnteredAggregateClock
    );
    assert_eq!(
        report.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::AggregateManaged
    );
    assert_eq!(
        report.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Reconfigured
    );
    assert_eq!(
        report.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::Aggregate
    );
    assert!(!report.observation.host_io.clocking.partial_availability);
    assert!(report.observation.host_io.clocking.crossing_required);
    assert!(report.render_json().contains("\"clock_domain\":\"Aggregate\""));
    assert!(report
        .render_json()
        .contains("\"transition_state\":\"EnteredAggregateClock\""));
    assert!(report
        .render_json()
        .contains("\"drift_state\":\"AggregateManaged\""));
    assert!(report.render_json().contains("\"endpoint_topology\":\"Aggregate\""));
}

#[test]
fn local_host_shared_report_tracks_return_to_direct_after_cross_clock_fallback() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    runtime
        .handshake(HandshakeRequest {
            client_version: "host-local-test".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(192_000),
        })
        .expect("handshake");
    runtime
        .configure(RuntimeConfigRequest::new(44_100, 256))
        .expect("configure");
    let mut host = LocalRuntimeHost::new(runtime);
    let initial = host.host_supervisor_report();
    assert_eq!(
        initial.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::InitialObservation
    );
    host.active_output_stream = Some(HardwareStreamConfig {
        device: AudioDeviceDescriptor {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio",
            device_id: "coreaudio:cross-clock-output".into(),
            name: "CoreAudio Cross Clock Output".into(),
            default_input: false,
            default_output: true,
            max_input_channels: 0,
            max_output_channels: 2,
            nominal_sample_rate: SampleRate(48_000),
            preferred_buffer_sizes: vec![256],
        },
        direction: AudioStreamDirection::Output,
        sample_rate: SampleRate(48_000),
        buffer_size: 256,
        input_channels: 0,
        output_channels: 2,
        sample_format: AudioSampleFormat::F32,
        interleaved: true,
        clock_source: HardwareClockSource::Internal,
        clock_topology: HardwareClockTopology::SingleEndpoint,
        lifecycle: HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::HostMustRestart,
        },
        latency: HardwareLatencyProfile::output_only(256),
        simulated: false,
    });

    let cross_clock = host.host_supervisor_report();
    assert_eq!(
        cross_clock.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::EnteredCrossClockFallback
    );

    host.active_output_stream = Some(HardwareStreamConfig {
        sample_rate: SampleRate(44_100),
        ..host
            .active_output_stream
            .clone()
            .expect("cross-clock stream should exist")
    });

    let recovered = host.host_supervisor_report();
    assert_eq!(
        recovered.observation.host_io.clocking.clock_domain,
        RuntimeHostClockDomain::SameClock
    );
    assert_eq!(
        recovered.observation.host_io.clocking.fallback_state,
        RuntimeHostClockFallbackState::Direct
    );
    assert_eq!(
        recovered.observation.host_io.clocking.transition_state,
        RuntimeHostClockTransitionState::ReturnedToDirect
    );
    assert_eq!(
        recovered.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Stable
    );
    assert_eq!(
        recovered.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Reconfigured
    );
    assert!(recovered
        .render_json()
        .contains("\"transition_state\":\"ReturnedToDirect\""));
    assert!(recovered
        .render_json()
        .contains("\"discontinuity_state\":\"Reconfigured\""));
}
