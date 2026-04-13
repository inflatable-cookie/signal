use crate::host::host_test_support::prepare_server_host_with_lifecycle;
use signal_runtime::{RecoveryRestartIntent, RuntimeObservationApi};

#[test]
fn server_host_overlap_recovery_keeps_bound_plugin_dispatch_truth() {
    let (mut host, protocol, mut lifecycle, mut run) = prepare_server_host_with_lifecycle();

    host.execute_block(&protocol, &mut run, 1, &mut lifecycle, false)
        .expect("initial realtime block");
    let mut recovered = host
        .recover_sandbox(
            &protocol,
            "server-default-sandbox",
            &mut lifecycle,
            &run,
            RecoveryRestartIntent::WatchdogRecovery,
            None,
        )
        .expect("overlap recovery");
    let block_sequence = host.runtime.allocate_block_sequence();
    host.execute_block(
        &protocol,
        &mut recovered,
        block_sequence,
        &mut lifecycle,
        false,
    )
    .expect("replacement realtime block");

    let snapshot = host.runtime.get_engine_block_snapshot();
    let concurrency = host.runtime.get_transport_concurrency_snapshot();

    assert_eq!(recovered.processing_epoch, 2);
    assert_eq!(
        recovered.last_engine_graph_id.as_deref(),
        Some("signal.host.server.demo")
    );
    let plugin_state = recovered
        .last_plugin_state
        .as_ref()
        .expect("replacement lifecycle should retain plugin state");
    assert_eq!(plugin_state.plugin_type_id, "plugin:clap:server");
    assert_eq!(plugin_state.instance_id, "instance:server:default");
    assert_eq!(plugin_state.lifecycle_state, "Active");
    assert_eq!(plugin_state.readiness_state, "Ready");
    assert!(plugin_state.active);
    assert!(snapshot.planned_nodes.iter().any(|node| {
        node.node_id == "drive"
            && node.plugin_sandbox_id.as_deref() == Some("server-default-sandbox")
    }));
    assert_eq!(
        snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.projection_epoch),
        Some(1)
    );
    assert_eq!(
        snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.timeline_position_samples),
        Some(((block_sequence as i64) * 512).rem_euclid(16 * 512))
    );
    assert_eq!(concurrency.current_attached_sessions, 1);
    assert_eq!(concurrency.current_recovery_overlap_sessions, 0);
    assert_eq!(concurrency.peak_attached_sessions, 2);
}
