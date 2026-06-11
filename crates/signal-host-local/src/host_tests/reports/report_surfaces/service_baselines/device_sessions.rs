use super::super::super::super::*;

#[test]
fn local_host_shared_report_surfaces_runtime_stretch_engine_baseline() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    host.boot_default().expect("default local host boot");
    let report = host.host_supervisor_report();

    assert_eq!(report.observation.observation.stretch_engine_snapshot.clip_count, 0);
    assert_eq!(
        report.observation.observation.stretch_engine_snapshot.ready_clip_count,
        0
    );
    assert!(report
        .observation
        .observation
        .stretch_engine_snapshot
        .clips
        .is_empty());

}
