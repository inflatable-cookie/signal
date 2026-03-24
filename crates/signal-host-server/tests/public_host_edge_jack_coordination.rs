use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeJackClientRole, RuntimeJackGraphCoordinationState,
    RuntimeJackGuardedCoordinationState, RuntimeJackTransportPosture, SignalRuntime,
};

#[test]
fn server_shared_host_edge_exports_runtime_jack_coordination_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::Detached
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::AttachedGuarded
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::PrimaryAudioIo
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.guarded_state,
        RuntimeJackGuardedCoordinationState::GraphGuarded
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"transport_posture\":\"Detached\""));
    assert!(rendered.contains("\"graph_state\":\"AttachedGuarded\""));
    assert!(rendered.contains("\"client_role\":\"PrimaryAudioIo\""));
    assert!(rendered.contains("\"guarded_state\":\"GraphGuarded\""));
}
