use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeJackClientRole, RuntimeJackGraphCoordinationState,
    RuntimeJackGuardedCoordinationState, RuntimeJackTransportPosture, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_jack_coordination_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default()
        .expect("public local jack coordination default boot should succeed");
    let report = host.supervisor_report();

    assert_eq!(
        report
            .observation
            .jack_coordination_snapshot
            .transport_posture,
        RuntimeJackTransportPosture::NotJack
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.graph_state,
        RuntimeJackGraphCoordinationState::NotJack
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.client_role,
        RuntimeJackClientRole::NotJack
    );
    assert_eq!(
        report.observation.jack_coordination_snapshot.guarded_state,
        RuntimeJackGuardedCoordinationState::NotJack
    );

    let rendered = report.render_json();
    assert!(rendered.contains("\"jack_coordination_snapshot\":{"));
    assert!(rendered.contains("\"transport_posture\":\"NotJack\""));
    assert!(rendered.contains("\"graph_state\":\"NotJack\""));
    assert!(rendered.contains("\"client_role\":\"NotJack\""));
    assert!(rendered.contains("\"guarded_state\":\"NotJack\""));
}
