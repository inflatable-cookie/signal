#[path = "support/public_contract_boundary_host_io_linux.rs"]
mod public_contract_boundary_host_io_linux_support;
#[path = "support/public_contract_boundary_transport_summary.rs"]
mod public_contract_boundary_transport_summary_support;

use public_contract_boundary_host_io_linux_support::{
    sample_public_linux_backend_host_io, PublicLinuxBackendHostIoConfig,
};
use public_contract_boundary_transport_summary_support::sample_public_transport_session_summary;
use signal_hardware::{BackendHealth, HardwareBackendIdentity, LinuxAudioBackendKind};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeHostAudioStreamState, RuntimeJackClientRole, RuntimeJackGraphCoordinationState,
    RuntimeJackGuardedCoordinationState, RuntimeJackTransportPosture, RuntimeLifecycleApi,
    RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime, TransportDispatchState,
    TransportHeartbeatFreshness, TransportSessionState,
};

#[test]
fn public_runtime_jack_coordination_boundary_reports_runtime_owned_transport_graph_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let recorder = RuntimeEventRecorder::default();
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-jack-coordination".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public jack coordination handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .expect("public jack coordination configure should succeed");

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
        device_id: "jack:main-graph",
        device_name: "JACK Main Graph",
        simulated: true,
        backend_health: BackendHealth::Healthy,
        device_loss_count: 0,
        restart_attempt_count: 0,
        restart_failure_count: 0,
    });
    let mut recovering_jack = sample_public_linux_backend_host_io(PublicLinuxBackendHostIoConfig {
        backend_identity: HardwareBackendIdentity::Linux(LinuxAudioBackendKind::Jack),
        backend_name: "jack",
        device_id: "jack:recovering-graph",
        device_name: "JACK Recovering Graph",
        simulated: true,
        backend_health: BackendHealth::Recovering,
        device_loss_count: 1,
        restart_attempt_count: 1,
        restart_failure_count: 0,
    });
    recovering_jack.audio_pump.stream_state = RuntimeHostAudioStreamState::Faulted;

    let not_jack_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_host_external_io(&alsa)
        .with_linux_backend_session_snapshot(&alsa)
        .with_jack_coordination_snapshot(&alsa);
    assert_eq!(
        not_jack_observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::NotJack
    );
    assert_eq!(
        not_jack_observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::NotJack
    );
    assert_eq!(
        not_jack_observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::NotJack
    );
    assert_eq!(
        not_jack_observation
            .jack_coordination_snapshot
            .guarded_state,
        RuntimeJackGuardedCoordinationState::NotJack
    );

    let mut following_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_host_external_io(&jack)
        .with_linux_backend_session_snapshot(&jack);
    following_observation.transport_session_summary = sample_public_transport_session_summary(
        TransportSessionState::AttachActive,
        true,
        TransportHeartbeatFreshness::Fresh,
        TransportDispatchState::Completed,
        1,
        0,
        0,
    );
    following_observation = following_observation.with_jack_coordination_snapshot(&jack);
    assert_eq!(
        following_observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::FollowingExternal
    );
    assert_eq!(
        following_observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::AttachedGuarded
    );
    assert_eq!(
        following_observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::FallbackContinuation
    );
    assert_eq!(
        following_observation
            .jack_coordination_snapshot
            .guarded_state,
        RuntimeJackGuardedCoordinationState::TransportGuarded
    );

    let mut recovering_observation = RuntimeObservationReport::capture(&runtime, &recorder)
        .with_host_external_io(&recovering_jack)
        .with_linux_backend_session_snapshot(&recovering_jack);
    recovering_observation.transport_session_summary = sample_public_transport_session_summary(
        TransportSessionState::DetachFaulted,
        true,
        TransportHeartbeatFreshness::Missed,
        TransportDispatchState::TimedOut,
        2,
        1,
        1,
    );
    recovering_observation =
        recovering_observation.with_jack_coordination_snapshot(&recovering_jack);
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::Guarded
    );
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .graph_state,
        RuntimeJackGraphCoordinationState::Recovering
    );
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .client_role,
        RuntimeJackClientRole::FallbackContinuation
    );
    assert_eq!(
        recovering_observation
            .jack_coordination_snapshot
            .guarded_state,
        RuntimeJackGuardedCoordinationState::Recovering
    );

    let mut supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    supervisor.observation = following_observation;
    let rendered = supervisor.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"transport_posture\":\"FollowingExternal\""));
    assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
    assert!(rendered.contains("\"client_role\":\"FallbackContinuation\""));
    assert!(rendered.contains("\"guarded_state\":\"TransportGuarded\""));
}
