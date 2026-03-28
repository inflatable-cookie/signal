#[path = "support/public_contract_boundary_host_io_linux.rs"]
mod public_contract_boundary_host_io_linux_support;

use public_contract_boundary_host_io_linux_support::{
    sample_public_linux_backend_host_io, PublicLinuxBackendHostIoConfig,
};
use signal_hardware::{BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-linux-audio-backend".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public linux audio backend handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public linux audio backend configure should succeed");

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.external_io_snapshot.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Unavailable
    );
    assert_eq!(
        baseline.external_io_snapshot.linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Unsupported
    );

    let alsa = sample_public_linux_backend_host_io(PublicLinuxBackendHostIoConfig {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        backend_name: "alsa",
        device_id: "alsa:default-output",
        device_name: "ALSA Default Output",
        simulated: false,
        backend_health: BackendHealth::Healthy,
        device_loss_count: 0,
        restart_attempt_count: 0,
        restart_failure_count: 0,
    });
    let jack = sample_public_linux_backend_host_io(PublicLinuxBackendHostIoConfig {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        backend_name: "jack",
        device_id: "jack:graph-main",
        device_name: "JACK Graph Main",
        simulated: true,
        backend_health: BackendHealth::Recovering,
        device_loss_count: 1,
        restart_attempt_count: 1,
        restart_failure_count: 0,
    });
    let pipewire = sample_public_linux_backend_host_io(PublicLinuxBackendHostIoConfig {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        backend_name: "pipewire",
        device_id: "pipewire:default-graph",
        device_name: "PipeWire Default Graph",
        simulated: false,
        backend_health: BackendHealth::Degraded,
        device_loss_count: 0,
        restart_attempt_count: 1,
        restart_failure_count: 1,
    });

    let alsa_observation = baseline.clone().with_host_external_io(&alsa);
    let jack_observation = baseline.clone().with_host_external_io(&jack);
    let pipewire_observation = baseline.with_host_external_io(&pipewire);

    assert_eq!(
        alsa_observation.external_io_snapshot.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa_observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Portable
    );
    assert_eq!(
        jack_observation.external_io_snapshot.linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack_observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Guarded
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::PipeWire
    );
    assert_eq!(
        pipewire_observation
            .external_io_snapshot
            .linux_backend_portability,
        signal_runtime::RuntimeLinuxAudioBackendPortabilityBand::Guarded
    );

    let observation_json = pipewire_observation.render_json();
    assert!(observation_json.contains("\"linux_backend_identity\":\"PipeWire\""));
    assert!(observation_json.contains("\"linux_backend_portability\":\"Guarded\""));
    assert!(observation_json.contains("\"backend_name\":\"pipewire\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor.observation.clone().with_host_external_io(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"external_io_snapshot\":{"));
    assert!(supervisor_json.contains("\"linux_backend_identity\":\"Alsa\""));
    assert!(supervisor_json.contains("\"linux_backend_portability\":\"Portable\""));
}
