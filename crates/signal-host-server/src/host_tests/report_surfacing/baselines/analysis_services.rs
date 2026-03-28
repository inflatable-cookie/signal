use super::super::super::*;

#[test]
fn server_host_shared_report_surfaces_runtime_stretch_engine_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.stretch_engine_snapshot.clip_count, 0);
    assert_eq!(report.observation.stretch_engine_snapshot.ready_clip_count, 0);
    assert!(report.observation.stretch_engine_snapshot.clips.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"stretch_engine_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"sample_domain_clip_count\":0"));
}

#[test]
fn server_host_shared_report_surfaces_runtime_marker_analysis_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.marker_analysis_snapshot.clip_count, 0);
    assert_eq!(report.observation.marker_analysis_snapshot.ready_clip_count, 0);
    assert_eq!(
        report.observation.marker_analysis_snapshot.tempo_assist_ready_clip_count,
        0
    );
    assert!(report.observation.marker_analysis_snapshot.clips.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"marker_analysis_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"tempo_assist_ready_clip_count\":0"));
}

#[test]
fn server_host_shared_report_surfaces_runtime_transform_artifact_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.transform_artifact_snapshot.clip_count, 0);
    assert_eq!(report.observation.transform_artifact_snapshot.ready_clip_count, 0);
    assert_eq!(report.observation.transform_artifact_snapshot.reusable_clip_count, 0);
    assert!(report.observation.transform_artifact_snapshot.clips.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"transform_artifact_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"reusable_clip_count\":0"));
}

#[test]
fn server_host_shared_report_surfaces_runtime_preview_transform_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let host = ServerRuntimeHost::new(runtime);
    let report = host.supervisor_report();

    assert_eq!(report.observation.preview_transform_snapshot.clip_count, 0);
    assert_eq!(report.observation.preview_transform_snapshot.active_audition_clip_count, 0);
    assert_eq!(report.observation.preview_transform_snapshot.ready_clip_count, 0);
    assert_eq!(
        report.observation.preview_transform_snapshot.artifact_backed_clip_count,
        0
    );
    assert!(report.observation.preview_transform_snapshot.clips.is_empty());

    let rendered = report.render_json();
    assert!(rendered.contains("\"preview_transform_snapshot\":{"));
    assert!(rendered.contains("\"clip_count\":0"));
    assert!(rendered.contains("\"artifact_backed_clip_count\":0"));
}
