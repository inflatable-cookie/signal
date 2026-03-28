use super::*;
use crate::{
    AudioSampleFormat, AudioStreamDirection, HardwareConfigRequest, HardwareDiagnosticEvent,
    HardwareDiagnosticKind, HardwareDiagnosticSeverity, HardwareLifecycleOwnership,
    HardwareRestartPolicy, LinuxAudioBackendKind,
};
use signal_primitives::SampleRate;

#[test]
fn simulated_backend_negotiates_default_output_stream_and_runtime_request() {
    let backend = SimulatedHardwareBackend::default_stereo_output(
        HardwareBackendIdentity::Unsupported,
        "simulated",
        "sim:default-output",
        "Simulated Output",
    )
    .with_lifecycle(HardwareLifecycleContract {
        ownership: HardwareLifecycleOwnership::HostDrivenCallback,
        restart_policy: HardwareRestartPolicy::BackendMayRestart,
    });

    let device = backend
        .default_output_device()
        .expect("default output device available");
    let request = HardwareStreamRequest::new_output(device.device_id.clone(), 48_000, 256)
        .with_output_channels(2);
    let stream = backend
        .negotiate_stream(&request)
        .expect("negotiate simulated output stream");

    assert_eq!(stream.device.device_id, "sim:default-output");
    assert_eq!(
        stream.device.backend_identity,
        HardwareBackendIdentity::Unsupported
    );
    assert_eq!(stream.direction, AudioStreamDirection::Output);
    assert_eq!(stream.sample_rate, SampleRate(48_000));
    assert_eq!(stream.buffer_size, 256);
    assert_eq!(stream.output_channels, 2);
    assert_eq!(stream.sample_format, AudioSampleFormat::F32);
    assert!(stream.interleaved);
    assert_eq!(stream.clock_source, HardwareClockSource::Virtual);
    assert_eq!(stream.clock_topology, HardwareClockTopology::SingleEndpoint);
    assert_eq!(
        stream.lifecycle,
        HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::BackendMayRestart,
        }
    );
    assert_eq!(stream.latency, HardwareLatencyProfile::output_only(256));
    assert!(stream.simulated);

    let runtime_request = HardwareConfigRequest::from_stream(&stream, backend.policy_record().tier);
    assert_eq!(runtime_request.sample_rate, SampleRate(48_000));
    assert_eq!(runtime_request.buffer_size, 256);
    assert_eq!(runtime_request.output_channels, 2);
    assert_eq!(
        runtime_request.backend_policy,
        BackendPolicyTier::Tier0InHost
    );
    assert_eq!(
        backend.policy_record().backend_identity,
        HardwareBackendIdentity::Unsupported
    );
}

#[test]
fn simulated_backend_surfaces_diagnostics_contract() {
    let diagnostics = HardwareDiagnosticsSnapshot {
        health: BackendHealth::Degraded,
        xrun_count: 3,
        callback_overrun_count: 1,
        device_loss_count: 1,
        restart_attempt_count: 2,
        restart_failure_count: 1,
        last_event: Some(HardwareDiagnosticEvent {
            kind: HardwareDiagnosticKind::RestartFailed,
            severity: HardwareDiagnosticSeverity::Critical,
            device_id: Some("sim:default-output".into()),
            callback_index: Some(42),
            detail: "simulated restart failure".into(),
        }),
    };
    let backend = SimulatedHardwareBackend::default_stereo_output(
        HardwareBackendIdentity::Unsupported,
        "simulated",
        "sim:default-output",
        "Simulated Output",
    )
    .with_diagnostics(diagnostics.clone());

    assert_eq!(backend.health(), BackendHealth::Degraded);
    assert_eq!(backend.diagnostics(), diagnostics);
}

#[test]
fn simulated_linux_backend_baselines_surface_typed_identity_and_contracts() {
    let alsa = SimulatedHardwareBackend::linux_alsa_default_output();
    let jack = SimulatedHardwareBackend::linux_jack_duplex();
    let pipewire = SimulatedHardwareBackend::linux_pipewire_duplex();

    assert_eq!(
        alsa.policy_record().backend_identity,
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa)
    );
    assert_eq!(
        jack.policy_record().backend_identity,
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
    );
    assert_eq!(
        pipewire.policy_record().backend_identity,
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire)
    );

    let alsa_stream = alsa
        .negotiate_stream(&HardwareStreamRequest::new_output(
            "alsa:default-output",
            48_000,
            256,
        ))
        .expect("alsa baseline should negotiate");
    assert_eq!(
        alsa_stream.device.backend_identity,
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa)
    );
    assert_eq!(alsa_stream.clock_source, HardwareClockSource::Internal);
    assert_eq!(
        alsa_stream.clock_topology,
        HardwareClockTopology::SingleEndpoint
    );
    assert_eq!(
        alsa_stream.lifecycle,
        HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::HostDrivenCallback,
            restart_policy: HardwareRestartPolicy::HostMustRestart,
        }
    );

    let jack_stream = jack
        .negotiate_stream(
            &HardwareStreamRequest::new_output("jack:graph-main", 48_000, 128)
                .with_input_channels(2)
                .with_output_channels(2),
        )
        .expect("jack baseline should negotiate");
    assert_eq!(
        jack_stream.device.backend_identity,
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack)
    );
    assert_eq!(
        jack_stream.clock_source,
        HardwareClockSource::ExternalWordClock
    );
    assert_eq!(jack_stream.clock_topology, HardwareClockTopology::Aggregate);
    assert_eq!(
        jack_stream.lifecycle,
        HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::BackendManagedCallback,
            restart_policy: HardwareRestartPolicy::BackendMayRestart,
        }
    );

    let pipewire_stream = pipewire
        .negotiate_stream(
            &HardwareStreamRequest::new_output("pipewire:default-graph", 48_000, 512)
                .with_input_channels(2)
                .with_output_channels(2),
        )
        .expect("pipewire baseline should negotiate");
    assert_eq!(
        pipewire_stream.device.backend_identity,
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire)
    );
    assert_eq!(pipewire_stream.clock_source, HardwareClockSource::Virtual);
    assert_eq!(
        pipewire_stream.clock_topology,
        HardwareClockTopology::Aggregate
    );
    assert_eq!(
        pipewire_stream.lifecycle,
        HardwareLifecycleContract {
            ownership: HardwareLifecycleOwnership::BackendManagedCallback,
            restart_policy: HardwareRestartPolicy::BackendMayRestart,
        }
    );
}
