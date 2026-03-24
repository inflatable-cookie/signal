#[path = "support/public_contract_boundary_host_io_linux.rs"]
mod public_contract_boundary_host_io_linux_support;

use public_contract_boundary_host_io_linux_support::sample_public_linux_backend_host_io;
use signal_hardware::{BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeHostAudioStreamState, RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain,
    RuntimeHostClockDriftState, RuntimeHostClockFallbackState, RuntimeHostClockTransitionState,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology, RuntimeHostLifecycleOwnership,
    RuntimeHostRestartPolicy, RuntimeLifecycleApi, RuntimeObservationReport,
    RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_linux_live_ownership_boundary_reports_runtime_owned_session_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-linux-live-ownership".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public linux live ownership handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public linux live ownership configure should succeed");

    let baseline = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_eq!(
        baseline.linux_backend_session_snapshot.backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Unavailable
    );
    assert_eq!(
        baseline.linux_backend_session_snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::Unavailable
    );

    let mut alsa = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Alsa),
        "alsa",
        "alsa:default-output",
        "ALSA Default Output",
        false,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    alsa.clocking.ownership = RuntimeHostLifecycleOwnership::HostDrivenCallback;
    alsa.clocking.restart_policy = RuntimeHostRestartPolicy::HostMustRestart;
    alsa.clocking.clock_domain = RuntimeHostClockDomain::SameClock;
    alsa.clocking.fallback_state = RuntimeHostClockFallbackState::Direct;
    alsa.clocking.transition_state = RuntimeHostClockTransitionState::Stable;
    alsa.clocking.drift_state = RuntimeHostClockDriftState::Stable;
    alsa.clocking.discontinuity_state = RuntimeHostClockDiscontinuityState::Continuous;
    alsa.clocking.duplex_mismatch_state = RuntimeHostDuplexMismatchState::Aligned;
    alsa.clocking.endpoint_topology = RuntimeHostEndpointTopology::Duplex;

    let jack = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        "jack",
        "jack:graph-main",
        "JACK Graph Main",
        true,
        BackendHealth::Healthy,
        0,
        0,
        0,
    );
    let mut pipewire = sample_public_linux_backend_host_io(
        HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        "pipewire",
        "pipewire:default-graph",
        "PipeWire Default Graph",
        true,
        BackendHealth::Recovering,
        1,
        1,
        1,
    );
    pipewire.audio_pump.stream_state = RuntimeHostAudioStreamState::Faulted;

    let alsa_observation = baseline.clone().with_linux_backend_session_snapshot(&alsa);
    let jack_observation = baseline.clone().with_linux_backend_session_snapshot(&jack);
    let pipewire_observation = baseline.with_linux_backend_session_snapshot(&pipewire);

    assert_eq!(
        alsa_observation
            .linux_backend_session_snapshot
            .backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Alsa
    );
    assert_eq!(
        alsa_observation.linux_backend_session_snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::HostBrokeredCallback
    );
    assert_eq!(
        alsa_observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::DirectClaim
    );
    assert_eq!(
        jack_observation
            .linux_backend_session_snapshot
            .backend_identity,
        signal_runtime::RuntimeLinuxAudioBackendIdentity::Jack
    );
    assert_eq!(
        jack_observation.linux_backend_session_snapshot.ownership,
        signal_runtime::RuntimeLinuxBackendSessionOwnership::BackendManagedGraph
    );
    assert_eq!(
        jack_observation
            .linux_backend_session_snapshot
            .ownership_fallback,
        signal_runtime::RuntimeLinuxBackendOwnershipFallbackState::BackendManagedGuarded
    );
    assert_eq!(
        pipewire_observation
            .linux_backend_session_snapshot
            .lifecycle_state,
        signal_runtime::RuntimeLinuxBackendSessionLifecycleState::Recovering
    );
    assert_eq!(
        pipewire_observation
            .linux_backend_session_snapshot
            .device_claim_posture,
        signal_runtime::RuntimeLinuxBackendDeviceClaimPosture::Lost
    );
    assert_eq!(
        pipewire_observation
            .linux_backend_session_snapshot
            .session_role,
        signal_runtime::RuntimeLinuxBackendSessionRole::FallbackContinuation
    );

    let observation_json = pipewire_observation.render_json();
    assert!(observation_json.contains("\"linux_backend_session_snapshot\":{"));
    assert!(observation_json.contains("\"backend_identity\":\"PipeWire\""));
    assert!(observation_json.contains("\"lifecycle_state\":\"Recovering\""));
    assert!(observation_json.contains("\"device_claim_posture\":\"Lost\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_linux_backend_session_snapshot(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"linux_backend_session_snapshot\":{"));
    assert!(supervisor_json.contains("\"backend_identity\":\"Alsa\""));
    assert!(supervisor_json.contains("\"ownership\":\"HostBrokeredCallback\""));
}
