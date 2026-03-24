use signal_host_server::ServerRuntimeHost;
use signal_runtime::{RuntimeConfig, RuntimeControlSurfaceGraphState, SignalRuntime};

#[test]
fn server_shared_host_edge_exports_runtime_control_surface_truth() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(
        report.observation.control_surface_snapshot.discovery_state,
        signal_runtime::RuntimeExternalMidiDiscoveryState::Idle
    );
    assert_eq!(
        report.observation.control_surface_snapshot.graph_state,
        RuntimeControlSurfaceGraphState::Empty
    );
    assert_eq!(
        report.observation.control_surface_snapshot.provider_name,
        "signal-host-server"
    );
    assert_eq!(report.observation.control_surface_snapshot.device_count, 0);
    assert!(report
        .observation
        .control_surface_snapshot
        .devices
        .is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"control_surface_snapshot\":{"));
    assert!(rendered.contains("\"graph_state\":\"Empty\""));
    assert!(rendered.contains("\"provider_name\":\"signal-host-server\""));
}
