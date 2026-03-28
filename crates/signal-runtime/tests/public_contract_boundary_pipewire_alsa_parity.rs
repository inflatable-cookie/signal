#[path = "support/public_contract_boundary_host_io_linux.rs"]
mod public_contract_boundary_host_io_linux_support;

use public_contract_boundary_host_io_linux_support::{
    sample_public_linux_backend_host_io, PublicLinuxBackendHostIoConfig,
};
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
fn public_runtime_pipewire_alsa_parity_boundary_reports_runtime_owned_claim_and_policy_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-pipewire-alsa-parity".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public pipewire/alsa parity handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public pipewire/alsa parity configure should succeed");

    let mut alsa = sample_public_linux_backend_host_io(PublicLinuxBackendHostIoConfig {
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
    alsa.clocking.ownership = RuntimeHostLifecycleOwnership::HostDrivenCallback;
    alsa.clocking.restart_policy = RuntimeHostRestartPolicy::HostMustRestart;
    alsa.clocking.clock_domain = RuntimeHostClockDomain::SameClock;
    alsa.clocking.fallback_state = RuntimeHostClockFallbackState::Direct;
    alsa.clocking.transition_state = RuntimeHostClockTransitionState::Stable;
    alsa.clocking.drift_state = RuntimeHostClockDriftState::Stable;
    alsa.clocking.discontinuity_state = RuntimeHostClockDiscontinuityState::Continuous;
    alsa.clocking.duplex_mismatch_state = RuntimeHostDuplexMismatchState::Aligned;
    alsa.clocking.endpoint_topology = RuntimeHostEndpointTopology::Duplex;

    let pipewire = sample_public_linux_backend_host_io(PublicLinuxBackendHostIoConfig {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
        backend_name: "pipewire",
        device_id: "pipewire:default-graph",
        device_name: "PipeWire Default Graph",
        simulated: true,
        backend_health: BackendHealth::Healthy,
        device_loss_count: 0,
        restart_attempt_count: 0,
        restart_failure_count: 0,
    });

    let mut recovering_pipewire =
        sample_public_linux_backend_host_io(PublicLinuxBackendHostIoConfig {
            backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::PipeWire),
            backend_name: "pipewire",
            device_id: "pipewire:recovering-graph",
            device_name: "PipeWire Recovering Graph",
            simulated: true,
            backend_health: BackendHealth::Recovering,
            device_loss_count: 1,
            restart_attempt_count: 1,
            restart_failure_count: 1,
        });
    recovering_pipewire.audio_pump.stream_state = RuntimeHostAudioStreamState::Faulted;

    let alsa_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_linux_backend_session_snapshot(&alsa)
        .with_pipewire_alsa_parity_snapshot(&alsa);
    assert_eq!(
        alsa_observation
            .pipewire_alsa_parity_snapshot
            .session_role_parity,
        signal_runtime::RuntimePipeWireAlsaSessionRoleParity::PrimaryAudioIo
    );
    assert_eq!(
        alsa_observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        signal_runtime::RuntimePipeWireAlsaDeviceClaimParity::DirectClaim
    );
    assert_eq!(
        alsa_observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        signal_runtime::RuntimePipeWireAlsaStreamPolicyParity::DirectHostCallback
    );
    assert_eq!(
        alsa_observation.pipewire_alsa_parity_snapshot.guarded_state,
        signal_runtime::RuntimePipeWireAlsaGuardedParityState::Direct
    );

    let pipewire_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_linux_backend_session_snapshot(&pipewire)
        .with_pipewire_alsa_parity_snapshot(&pipewire);
    assert_eq!(
        pipewire_observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        signal_runtime::RuntimePipeWireAlsaDeviceClaimParity::SharedGraph
    );
    assert_eq!(
        pipewire_observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        signal_runtime::RuntimePipeWireAlsaStreamPolicyParity::BackendManagedGraph
    );
    assert_eq!(
        pipewire_observation
            .pipewire_alsa_parity_snapshot
            .guarded_state,
        signal_runtime::RuntimePipeWireAlsaGuardedParityState::ClockGuarded
    );

    let recovering_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_linux_backend_session_snapshot(&recovering_pipewire)
        .with_pipewire_alsa_parity_snapshot(&recovering_pipewire);
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .session_role_parity,
        signal_runtime::RuntimePipeWireAlsaSessionRoleParity::FallbackContinuation
    );
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .device_claim_parity,
        signal_runtime::RuntimePipeWireAlsaDeviceClaimParity::Lost
    );
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .stream_policy_parity,
        signal_runtime::RuntimePipeWireAlsaStreamPolicyParity::Restarting
    );
    assert_eq!(
        recovering_observation
            .pipewire_alsa_parity_snapshot
            .guarded_state,
        signal_runtime::RuntimePipeWireAlsaGuardedParityState::RecoveryGuarded
    );

    let observation_json = recovering_observation.render_json();
    assert!(observation_json.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(observation_json.contains("\"session_role_parity\":\"FallbackContinuation\""));
    assert!(observation_json.contains("\"device_claim_parity\":\"Lost\""));
    assert!(observation_json.contains("\"stream_policy_parity\":\"Restarting\""));

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = supervisor
        .observation
        .clone()
        .with_linux_backend_session_snapshot(&alsa)
        .with_pipewire_alsa_parity_snapshot(&alsa);
    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"pipewire_alsa_parity_snapshot\":{"));
    assert!(supervisor_json.contains("\"stream_policy_parity\":\"DirectHostCallback\""));
}
