#[path = "support/public_host_edge_continuity.rs"]
mod public_host_edge_continuity_support;

use public_host_edge_continuity_support::apply_public_render_graph;
use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeOfflineRenderRequest, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_resumable_offline_render_session_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-edge-render".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .unwrap();
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 512))
        .unwrap();
    apply_public_render_graph(&mut runtime, "graph:host-local:render");
    runtime
        .begin_offline_render_execution(RuntimeOfflineRenderRequest {
            request_id: "render:host-local:resumable".into(),
            timeline_start_samples: 0,
            duration_samples: 512,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        })
        .unwrap();
    runtime
        .advance_offline_render_execution("render:host-local:resumable")
        .unwrap();
    runtime
        .pause_offline_render_execution("render:host-local:resumable")
        .unwrap();

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    assert_eq!(
        report
            .observation
            .offline_render_session_snapshot
            .active_sessions
            .first()
            .map(|session| session.interruption_class),
        Some(RuntimeInterruptionClass::Resumable)
    );
    let rendered = report.render_json();
    assert!(rendered.contains("\"offline_render_session_snapshot\":{"));
    assert!(rendered.contains("\"interruption_class\":\"Resumable\""));
}
