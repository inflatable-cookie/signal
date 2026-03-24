#[path = "support/public_host_edge_continuity.rs"]
mod public_host_edge_continuity_support;

use public_host_edge_continuity_support::apply_public_render_graph;
use signal_host_server::ServerRuntimeHost;
use signal_runtime::{
    RestartRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeInterruptionClass,
    RuntimeLifecycleApi, RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderRequest,
    SignalRuntime, StopReason,
};

#[test]
fn server_shared_host_edge_exports_restartable_and_terminal_offline_render_session_truth() {
    let mut restartable_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    restartable_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    restartable_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(
        &mut restartable_runtime,
        "graph:host-server:render-restartable",
    );
    restartable_runtime.start().unwrap();
    restartable_runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-server:restartable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    restartable_runtime
        .advance_offline_render_execution("render:host-server:restartable")
        .unwrap();
    restartable_runtime
        .stop(StopReason::DeviceReconfigure)
        .unwrap();
    restartable_runtime
        .restart(RestartRequest { reconfigure: None })
        .unwrap();

    let restartable_host = ServerRuntimeHost::new(restartable_runtime);
    let restartable_report = restartable_host.supervisor_report();
    assert_eq!(
        restartable_report
            .observation
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Restartable)
    );

    let mut terminal_runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    terminal_runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    terminal_runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(&mut terminal_runtime, "graph:host-server:render-terminal");
    terminal_runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-server:terminal".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: Some("/dev/null/signal-host-server-render-terminal".into()),
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    let mut terminal_error_observed = false;
    for _ in 0..16 {
        match terminal_runtime.advance_offline_render_execution("render:host-server:terminal") {
            Ok(_) => continue,
            Err(_) => {
                terminal_error_observed = true;
                break;
            }
        }
    }
    assert!(terminal_error_observed);

    let terminal_host = ServerRuntimeHost::new(terminal_runtime);
    let terminal_report = terminal_host.supervisor_report();
    assert_eq!(
        terminal_report
            .observation
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.state),
        Some(RuntimeOfflineRenderExecutionState::Failed)
    );
    assert_eq!(
        terminal_report
            .observation
            .offline_render_session_snapshot
            .last_session
            .as_ref()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Terminal)
    );
    let rendered = terminal_report.render_json();
    assert!(rendered.contains("\"offline_render_session_snapshot\":{"));
    assert!(rendered.contains("\"state\":\"Failed\""));
    assert!(rendered.contains("\"interruption_class\":\"Terminal\""));
}
