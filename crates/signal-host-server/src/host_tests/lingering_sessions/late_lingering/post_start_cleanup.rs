use crate::host::host_test_support::prepare_server_host_without_lifecycle;
use signal_plugin_clap::ClapSandboxLifecycleHarness;
use signal_runtime::{
    BrokerFailureStage, PluginSandboxTransportStage, RuntimeReadiness, TransportAttachIntent,
};
use std::path::Path;

#[test]
fn server_host_reconciles_late_lingering_completion_without_disturbing_active_replacement() {
    let (mut host, protocol) = prepare_server_host_without_lifecycle();
    let late_region = host
        .broker
        .create_region("server-late-lingering", 256)
        .expect("late lingering region");
    let late_transport = late_region.metadata().clone();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-late-origin",
            late_transport.region_id.as_str(),
            TransportAttachIntent::SteadyState,
            Some(late_transport.backing_path.clone()),
            Some(late_transport.total_bytes),
        )
        .expect("late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-late-origin",
        late_transport.region_id.as_str(),
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("late origin teardown completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered = host
        .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");

    host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

    let supervisor = host.supervisor_report();
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
            .active_sessions
            .len(),
        1
    );
    assert_eq!(
        supervisor
            .observation
            .transport_concurrency_snapshot
            .active_sessions[0]
            .lease_id,
        recovered.shared_memory_lease_id
    );
    assert!(!Path::new(&late_transport.backing_path).exists());
}

#[test]
fn server_host_keeps_active_replacement_running_when_late_lingering_cleanup_fails() {
    let (mut host, protocol) = prepare_server_host_without_lifecycle();
    host.runtime
        .begin_transport_session_with_metadata(
            "server-default-sandbox",
            "lease-late-origin",
            "region-late-origin-failure",
            TransportAttachIntent::SteadyState,
            None,
            None,
        )
        .expect("late lingering session");
    host.runtime.record_plugin_sandbox_transport(
        "server-default-sandbox",
        "lease-late-origin",
        "region-late-origin-failure",
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("late origin teardown completion".into()),
    );
    let mut lifecycle = ClapSandboxLifecycleHarness::default();
    let recovered = host
        .run_lifecycle(&protocol, "server-default-sandbox", 2, &mut lifecycle)
        .expect("replacement lifecycle");

    host.reconcile_late_lingering_sessions_after_start("server-default-sandbox", &recovered);

    let supervisor = host.supervisor_report();
    assert!(supervisor.observation.control_snapshot.running);
    assert_eq!(supervisor.observation.readiness, RuntimeReadiness::Ready);
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
        .any(|session| session.lease_id == recovered.shared_memory_lease_id));
    assert!(supervisor
        .observation
        .transport_concurrency_snapshot
        .active_sessions
        .iter()
        .any(|session| session.lease_id == "lease-late-origin"));
    assert!(supervisor
        .observation
        .observation
        .broker_failure_events
        .iter()
        .any(|failure| {
            failure.stage == BrokerFailureStage::TransportTeardown
                && failure.detail.contains("missing backing_path metadata")
        }));
}
