use super::super::super::super::*;
use crate::host::host_support;

#[test]
fn local_host_shared_report_surfaces_duplex_cross_clock_mismatch() {
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
    let _ = host.host_supervisor_report();
    host.active_output_stream = Some(HardwareStreamConfig {
        device: AudioDeviceDescriptor {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio",
            device_id: "coreaudio:duplex-cross-clock".into(),
            name: "CoreAudio Duplex Cross Clock".into(),
            default_input: true,
            default_output: true,
            max_input_channels: 2,
            max_output_channels: 2,
            nominal_sample_rate: SampleRate(48_000),
            preferred_buffer_sizes: vec![256],
        },
        direction: AudioStreamDirection::Duplex,
        sample_rate: SampleRate(48_000),
        buffer_size: 256,
        input_channels: 2,
        output_channels: 2,
        sample_format: AudioSampleFormat::F32,
        interleaved: true,
        clock_source: HardwareClockSource::Internal,
        clock_topology: HardwareClockTopology::SingleEndpoint,
        lifecycle: HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::HostMustRestart,
        },
        latency: HardwareLatencyProfile {
            input_latency_samples: Some(128),
            output_latency_samples: 256,
            round_trip_latency_samples: Some(384),
        },
        simulated: false,
    });

    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::Duplex
    );
    assert_eq!(
        report.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::CrossClockDiverged
    );
    assert_eq!(
        report.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::CrossClockManaged
    );
    assert_eq!(
        report.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Reconfigured
    );
    assert!(!report.observation.host_io.clocking.partial_availability);
    assert_eq!(
        report.observation.observation.external_io_snapshot.primary_role,
        signal_runtime::RuntimeExternalIoPrimaryRole::ProgramDuplex
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.monitoring_state,
        signal_runtime::RuntimeExternalIoMonitoringState::Guarded
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.loopback_state,
        signal_runtime::RuntimeExternalIoLoopbackState::Guarded
    );
}

#[test]
fn local_host_shared_report_surfaces_duplex_partial_availability() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    let mut host = LocalRuntimeHost::new(runtime);
    host.active_output_stream = Some(HardwareStreamConfig {
        device: AudioDeviceDescriptor {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio",
            device_id: "coreaudio:duplex-partial".into(),
            name: "CoreAudio Duplex Partial".into(),
            default_input: true,
            default_output: true,
            max_input_channels: 2,
            max_output_channels: 2,
            nominal_sample_rate: SampleRate(48_000),
            preferred_buffer_sizes: vec![256],
        },
        direction: AudioStreamDirection::Duplex,
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

    let report = host.host_supervisor_report();

    assert_eq!(
        report.observation.host_io.clocking.endpoint_topology,
        RuntimeHostEndpointTopology::Duplex
    );
    assert_eq!(
        report.observation.host_io.clocking.duplex_mismatch_state,
        RuntimeHostDuplexMismatchState::PartialAvailability
    );
    assert!(report.observation.host_io.clocking.partial_availability);
    assert_eq!(
        report.observation.host_io.clocking.drift_state,
        RuntimeHostClockDriftState::Stable
    );
    assert_eq!(
        report.observation.host_io.clocking.discontinuity_state,
        RuntimeHostClockDiscontinuityState::Continuous
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.monitoring_state,
        signal_runtime::RuntimeExternalIoMonitoringState::Guarded
    );
    assert_eq!(
        report.observation.observation.external_io_snapshot.loopback_state,
        signal_runtime::RuntimeExternalIoLoopbackState::Guarded
    );
}

#[test]
fn host_audio_transfer_bounds_channels_and_zero_fills_unwritten_output() {
    let runtime_output = AudioBuffer::from_interleaved(
        SampleRate(48_000),
        ChannelLayout::Count(ChannelCount(4)),
        vec![0.5, 0.4, 0.3, 0.2, 0.6, 0.5, 0.4, 0.3, 0.7, 0.6, 0.5, 0.4],
    );
    let stream = HardwareStreamConfig {
        device: AudioDeviceDescriptor {
            backend_identity: HardwareBackendIdentity::CoreAudio,
            backend_name: "coreaudio",
            device_id: "coreaudio:default-output".into(),
            name: "CoreAudio Default Output".into(),
            default_input: false,
            default_output: true,
            max_input_channels: 0,
            max_output_channels: 2,
            nominal_sample_rate: SampleRate(48_000),
            preferred_buffer_sizes: vec![3],
        },
        direction: AudioStreamDirection::Output,
        sample_rate: SampleRate(48_000),
        buffer_size: 4,
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
        latency: HardwareLatencyProfile::output_only(4),
        simulated: false,
    };
    let policy = LocalAudioTransferPolicy {
        max_callback_frames: 4,
        max_transfer_channels: 2,
        zero_fill_unwritten_output: true,
    };

    let transfer = host_support::transfer_runtime_output_to_host_buffer(
        &runtime_output,
        &stream,
        policy.into(),
    );

    assert_eq!(
        transfer.outcome,
        host_support::LocalAudioTransferOutcome {
            copied_samples: 6,
            zero_filled_samples: 2,
            dropped_samples: 6,
        }
    );
    assert!(transfer.output_peak >= 0.7);
}
