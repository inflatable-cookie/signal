use super::super::super::*;

#[test]
fn local_host_sweeps_prior_late_lingering_before_next_overlap_recovery() {
    let (mut host, protocol) = prepare_local_host_without_lifecycle();
    let late_region = host
        .broker
        .create_region("local-adjacent-lingering", 256)
        .expect("late lingering region");
    let late_transport = late_region.metadata().clone();
    host.runtime
        .begin_transport_session_with_metadata(
            "local-default-sandbox",
            "lease-prior-lingering",
            late_transport.region_id.as_str(),
            TransportAttachIntent::SteadyState,
            Some(late_transport.backing_path.clone()),
            Some(late_transport.total_bytes),
        )
        .expect("prior late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "local-default-sandbox",
        "lease-prior-lingering",
        late_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("prior late completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered_epoch2 = host
        .run_lifecycle(&protocol, "local-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");
    let recovered_transport = recovered_epoch2
        .transport
        .as_ref()
        .expect("recovered transport");
    host.runtime.record_plugin_sandbox_transport(
        "local-default-sandbox",
        recovered_epoch2.shared_memory_lease_id.as_str(),
        recovered_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(recovered_epoch2.processing_epoch),
        Some("current replacement became lingering before adjacent recovery".into()),
    );

    let recovered_epoch3 = host
        .recover_sandbox(
            &protocol,
            "local-default-sandbox",
            &mut lifecycle,
            &recovered_epoch2,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect("adjacent recovery should sweep prior lingering session");
    let supervisor = host.supervisor_report();

    assert_eq!(recovered_epoch3.processing_epoch, 3);
    assert!(supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
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
        .all(|session| session.lease_id != "lease-prior-lingering"));
    assert!(!Path::new(&late_transport.backing_path).exists());
}

#[test]
fn local_host_aborts_adjacent_overlap_recovery_when_prior_late_lingering_lacks_metadata() {
    let (mut host, protocol) = prepare_local_host_without_lifecycle();
    host.runtime
        .begin_transport_session_with_metadata(
            "local-default-sandbox",
            "lease-prior-lingering",
            "region-prior-lingering-failure",
            TransportAttachIntent::SteadyState,
            None,
            None,
        )
        .expect("prior late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "local-default-sandbox",
        "lease-prior-lingering",
        "region-prior-lingering-failure",
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("prior late completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered_epoch2 = host
        .run_lifecycle(&protocol, "local-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");

    let error = host
        .recover_sandbox(
            &protocol,
            "local-default-sandbox",
            &mut lifecycle,
            &recovered_epoch2,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect_err("adjacent recovery should abort on stale lingering metadata");
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
        .any(|session| session.lease_id == "lease-prior-lingering"));
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == recovered_epoch2.shared_memory_lease_id));
}
