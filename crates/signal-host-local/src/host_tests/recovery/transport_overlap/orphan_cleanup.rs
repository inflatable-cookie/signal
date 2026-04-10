use super::super::*;
use std::path::Path;

#[test]
fn local_host_sweeps_orphan_lingering_sessions_before_overlap_recovery() {
    let (mut host, protocol, mut lifecycle, run) = prepare_local_host_with_lifecycle();
    let orphan_region = host
        .broker
        .create_region("local-orphan-lingering", 256)
        .expect("orphan region");
    let orphan_transport = orphan_region.metadata().clone();
    host.runtime
        .begin_transport_session_with_metadata(
            "local-default-sandbox",
            "lease-orphan",
            orphan_transport.region_id.as_str(),
            TransportAttachIntent::RecoveryOverlap,
            Some(orphan_transport.backing_path.clone()),
            Some(orphan_transport.total_bytes),
        )
        .expect("orphan transport session");
    host.runtime.record_plugin_sandbox_transport(
        "local-default-sandbox",
        "lease-orphan",
        orphan_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("replacement rollback linger".into()),
    );

    let recovered = host
        .recover_sandbox(
            &protocol,
            "local-default-sandbox",
            &mut lifecycle,
            &run,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect("orphan lingering sweep recovery");
    let supervisor = host.supervisor_report();

    assert_eq!(recovered.processing_epoch, 2);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
    assert!(supervisor.observation.control_snapshot.running);
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .peak_attached_sessions,
        2
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .all(|session| session.lease_id != "lease-orphan"));
    assert!(!Path::new(&orphan_transport.backing_path).exists());
}

#[test]
fn local_host_aborts_when_orphan_lingering_cleanup_fails_before_overlap_recovery() {
    let (mut host, protocol, mut lifecycle, run) = prepare_local_host_with_lifecycle();
    host.runtime
        .begin_transport_session_with_metadata(
            "local-default-sandbox",
            "lease-orphan",
            "region-orphan-failure",
            TransportAttachIntent::RecoveryOverlap,
            None,
            None,
        )
        .expect("orphan transport session");
    host.runtime.record_plugin_sandbox_transport(
        "local-default-sandbox",
        "lease-orphan",
        "region-orphan-failure",
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("replacement rollback linger".into()),
    );

    let error = host
        .recover_sandbox(
            &protocol,
            "local-default-sandbox",
            &mut lifecycle,
            &run,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect_err("orphan lingering cleanup failure should abort recovery");
    let supervisor = host.supervisor_report();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceUnavailable);
    assert!(error.message.contains("missing backing_path metadata"));
    assert!(!supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Stopped);
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        2
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        1
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == "lease-orphan"));
}

#[test]
fn local_host_cleans_multiple_orphan_lingering_sessions_for_same_sandbox() {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let orphan_region_a = host
        .broker
        .create_region("local-orphan-a", 256)
        .expect("orphan region a");
    let orphan_transport_a = orphan_region_a.metadata().clone();
    let orphan_region_b = host
        .broker
        .create_region("local-orphan-b", 256)
        .expect("orphan region b");
    let orphan_transport_b = orphan_region_b.metadata().clone();

    host.runtime
        .begin_transport_session_with_metadata(
            "local-default-sandbox",
            "lease-orphan-a",
            orphan_transport_a.region_id.as_str(),
            TransportAttachIntent::SteadyState,
            Some(orphan_transport_a.backing_path.clone()),
            Some(orphan_transport_a.total_bytes),
        )
        .expect("orphan session a");
    host.runtime.record_plugin_sandbox_transport(
        "local-default-sandbox",
        "lease-orphan-a",
        orphan_transport_a.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("orphan a lingering".into()),
    );
    host.runtime
        .begin_transport_session_with_metadata(
            "local-default-sandbox",
            "lease-orphan-b",
            orphan_transport_b.region_id.as_str(),
            TransportAttachIntent::RecoveryOverlap,
            Some(orphan_transport_b.backing_path.clone()),
            Some(orphan_transport_b.total_bytes),
        )
        .expect("orphan session b");
    host.runtime.record_plugin_sandbox_transport(
        "local-default-sandbox",
        "lease-orphan-b",
        orphan_transport_b.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("orphan b lingering".into()),
    );

    host.cleanup_orphan_lingering_sessions_for_sandbox(
        "local-default-sandbox",
        1,
        None,
        None,
        LingeringCleanupMode::StrictPreAttach,
    )
    .expect("multiple orphan cleanup");

    let supervisor = host.supervisor_report();
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_attached_sessions,
        0
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .current_lingering_sessions,
        0
    );
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .is_empty());
    assert!(!Path::new(&orphan_transport_a.backing_path).exists());
    assert!(!Path::new(&orphan_transport_b.backing_path).exists());
}
