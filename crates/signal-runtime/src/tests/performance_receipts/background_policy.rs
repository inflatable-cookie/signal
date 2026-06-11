use super::super::*;

#[test]
fn runtime_performance_snapshot_captures_scheduler_pressure_and_background_policy() {
    let (mut runtime, imported_path) = prepare_offline_render_engine_runtime();
    runtime.set_cpu_load_percent(11.5);
    runtime.set_graph_latency_ms(4.25);
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .expect("enable safe mode");

    let deferred = runtime
        .render_offline_queue(vec![RuntimeOfflineRenderRequest {
            request_id: "render:queue:performance:0001".into(),
            timeline_start_samples: 0,
            duration_samples: 64,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        }])
        .expect("safe mode should defer offline render queue");

    assert_eq!(
        deferred.orchestration.decision,
        RuntimeDeferredServiceDecision::Defer
    );

    let report = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());
    let performance = report.performance_snapshot();

    assert_eq!(performance.sample_rate_hz, 48_000);
    assert_eq!(performance.block_size, 256);
    assert!((performance.cpu_load_percent - 11.5).abs() < 1.0e-6);
    assert!((performance.graph_latency_ms - 4.25).abs() < 1.0e-6);
    let engine_snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        performance.prework_service_state,
        engine_snapshot.prework_service_state
    );
    assert_eq!(
        performance.prework_service_pressure,
        engine_snapshot.prework_service_pressure
    );
    assert_eq!(
        performance.scheduler_prepared_dispatch_count,
        engine_snapshot.prepared_dispatch_count
    );
    assert_eq!(
        performance.scheduler_realtime_dispatch_count,
        engine_snapshot.realtime_dispatch_count
    );
    assert_eq!(
        performance.scheduler_dispatch_handoff_count,
        engine_snapshot.dispatch_handoff_count
    );
    assert_eq!(
        performance.scheduler_topology_compatible,
        engine_snapshot.scheduler_topology.compatible
    );
    assert_eq!(
        performance.scheduler_topology_requires_host_reinterpretation,
        engine_snapshot
            .scheduler_topology
            .requires_host_reinterpretation
    );
    assert_eq!(
        performance.scheduler_topology_issue_count,
        engine_snapshot.scheduler_topology.issues.len()
    );
    assert_eq!(
        performance.prework_service_starvation_count,
        engine_snapshot.prework_service_starvation_count
    );
    assert_eq!(
        performance.prework_service_throttle_count,
        engine_snapshot.prework_service_throttle_count
    );
    assert_eq!(
        performance.prework_service_yield_count,
        engine_snapshot.prework_service_yield_count
    );
    assert_eq!(
        performance.last_prework_service_effective_cycles,
        engine_snapshot.last_prework_service_effective_cycles
    );
    assert_eq!(
        performance.last_prework_service_budget_per_cycle,
        engine_snapshot.last_prework_service_budget_per_cycle
    );
    assert_eq!(
        performance.last_prework_service_effective_budget_per_cycle,
        engine_snapshot.last_prework_service_effective_budget_per_cycle
    );
    assert_eq!(
        performance.last_prework_serviced_backlog_class,
        engine_snapshot
            .last_prework_serviced_backlog_class
            .map(|class| format!("{class:?}"))
    );
    let expected_hot_node = engine_snapshot
        .planned_nodes
        .iter()
        .max_by_key(|node| node.latency_samples)
        .filter(|node| node.latency_samples > 0)
        .expect("prepared runtime should expose a latency-bearing hot node");
    assert_eq!(
        performance.hot_latency_node_id.as_deref(),
        Some(expected_hot_node.node_id.as_str())
    );
    assert_eq!(
        performance.hot_latency_node_group.as_deref(),
        Some(match expected_hot_node.group {
            GraphNodePlanningGroup::InlineRealtime => "InlineRealtime",
            GraphNodePlanningGroup::StatefulRealtime => "StatefulRealtime",
            GraphNodePlanningGroup::AnticipativeEligible => "AnticipativeEligible",
        })
    );
    assert_eq!(
        performance.hot_latency_node_topology_role.as_deref(),
        Some(match expected_hot_node.topology_role {
            GraphNodeTopologyRole::Utility => "Utility",
            GraphNodeTopologyRole::TrackLane => "TrackLane",
            GraphNodeTopologyRole::Bus => "Bus",
            GraphNodeTopologyRole::Send => "Send",
            GraphNodeTopologyRole::Return => "Return",
            GraphNodeTopologyRole::ConsoleNode => "ConsoleNode",
        })
    );
    assert_eq!(
        performance.hot_latency_node_samples,
        expected_hot_node.latency_samples
    );
    let expected_group_total = engine_snapshot
        .planned_nodes
        .iter()
        .filter(|node| node.group == expected_hot_node.group)
        .map(|node| node.latency_samples)
        .sum::<u32>();
    assert_eq!(
        performance.hot_latency_group.as_deref(),
        performance.hot_latency_node_group.as_deref()
    );
    assert_eq!(
        performance.hot_latency_group_node_count,
        engine_snapshot
            .planned_nodes
            .iter()
            .filter(|node| node.group == expected_hot_node.group)
            .count()
    );
    assert_eq!(
        performance.hot_latency_group_total_samples,
        expected_group_total
    );
    let expected_lane = performance
        .worker_lane_summaries
        .iter()
        .max_by_key(|summary| summary.total_latency_samples)
        .expect("prepared runtime should export at least one worker-lane summary");
    assert_eq!(
        performance.critical_path_lane.as_deref(),
        Some(match expected_lane.lane {
            GraphExecutionLane::Realtime => "Realtime",
            GraphExecutionLane::Anticipative => "Anticipative",
        })
    );
    assert_eq!(
        performance.critical_path_lane_node_count,
        expected_lane.node_count
    );
    assert_eq!(
        performance.critical_path_lane_plugin_backed_node_count,
        expected_lane.plugin_backed_node_count
    );
    assert_eq!(
        performance.critical_path_lane_planning_group_count,
        expected_lane.planning_group_count
    );
    assert_eq!(
        performance.critical_path_lane_total_latency_samples,
        expected_lane.total_latency_samples
    );
    assert_eq!(
        performance.worker_lane_summaries.len(),
        engine_snapshot.lane_order.len()
    );
    assert!(performance.worker_lane_summaries.iter().all(|summary| {
        summary.node_count > 0 && summary.total_latency_samples >= summary.max_node_latency_samples
    }));
    assert_eq!(
        performance.background_service_class,
        Some(RuntimeDeferredServiceClass::OfflineRenderQueue)
    );
    assert_eq!(
        performance.background_service_decision,
        Some(RuntimeDeferredServiceDecision::Defer)
    );
    assert_eq!(
        performance.background_service_reason,
        Some(RuntimeDeferredServiceReason::SafeMode)
    );
    assert_eq!(
        performance.background_service_priority_band,
        Some(RuntimeDeferredServicePriorityBand::UserVisible)
    );
    assert_eq!(
        performance.background_service_blocking_priority_band,
        Some(RuntimeDeferredServicePriorityBand::RecoveryCritical)
    );
    assert_eq!(
        performance.background_service_backpressure_source,
        Some(RuntimeDeferredServiceBackpressureSource::SafeMode)
    );
    assert!(performance.background_service_starvation_risk);
    assert_eq!(performance.background_service_starved_work_item_count, 1);
    assert_eq!(performance.background_service_cancellation_cause, None);
    assert_eq!(performance.background_service_cancelled_work_item_count, 0);
    assert_eq!(performance.background_queued_work_item_count, 1);
    assert_eq!(performance.background_deferred_work_item_count, 1);

    runtime
        .set_safe_mode(SafeModeRequest { enabled: false })
        .expect("disable safe mode");

    let _ = fs::remove_file(imported_path);
    if let Some(path) = runtime
        .get_media_pipeline_snapshot()
        .assets
        .first()
        .and_then(|asset| asset.cache_path.as_deref())
    {
        let _ = fs::remove_file(path);
    }
}
