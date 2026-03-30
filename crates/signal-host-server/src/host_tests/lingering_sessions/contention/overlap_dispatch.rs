use super::super::super::host_test_support::prepare_server_host_with_lifecycle;
use crate::host_support::{LOCAL_DEMO_GRAPH_ID, LOCAL_DEMO_PLUGIN_NODE_ID};

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
        recovered
            .last_plugin_render_context
            .as_ref()
            .map(|context| context.tempo_bpm),
        Some(126.0)
    );
    assert_eq!(
        recovered
            .last_plugin_render_context
            .as_ref()
            .map(|context| context.timeline_position_samples),
        Some(((block_sequence as i64) * 512).rem_euclid(16 * 512))
    );
    assert_eq!(
        recovered.last_plugin_automation_value,
        Some(((block_sequence % 8) as f32) / 7.0)
    );
    assert_eq!(recovered.plugin_render_bypass_count, 0);
    assert!(!recovered.last_plugin_render_bypassed);
    assert_eq!(recovered.last_engine_graph_id.as_deref(), Some(LOCAL_DEMO_GRAPH_ID));
    assert!(snapshot.planned_nodes.iter().any(|node| {
        node.node_id == LOCAL_DEMO_PLUGIN_NODE_ID
            && node.plugin_sandbox_id.as_deref() == Some("server-default-sandbox")
    }));
    assert_eq!(
        snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.projection_epoch),
        Some(2)
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
