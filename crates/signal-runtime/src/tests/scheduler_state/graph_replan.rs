use super::super::*;

#[test]
fn runtime_replans_graph_when_anticipative_mode_changes() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:planning".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "input".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "drive".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::TanhDrive { drive: 1.4 }],
                },
                GraphNodeProjection {
                    node_id: "output".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                },
            ],
        })
        .unwrap();

    let initial = runtime.get_engine_block_snapshot();
    assert!(initial.anticipative_planning_enabled);
    assert_eq!(initial.inline_realtime_node_count, 1);
    assert_eq!(initial.stateful_realtime_node_count, 1);
    assert_eq!(initial.anticipative_eligible_node_count, 1);
    assert_eq!(initial.prepared_dispatch_count, 1);
    assert_eq!(initial.realtime_dispatch_count, 1);
    assert_eq!(initial.dispatch_handoff_count, 1);
    assert!(initial.prework_cache_enabled);
    assert_eq!(initial.prework_cache_state, RuntimePreworkCacheState::Empty);
    assert_eq!(
        initial.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Empty
    );
    assert_eq!(initial.prework_cache_admissions, 0);
    assert_eq!(initial.prework_cache_consumptions, 0);
    assert_eq!(initial.prework_cache_hits, 0);
    assert_eq!(initial.prework_cache_misses, 0);
    assert_eq!(initial.prework_cache_invalidation_count, 0);
    assert_eq!(initial.prework_cache_retirement_count, 0);

    let mut request = RuntimeConfigRequest::new(48_000, 256);
    request.anticipative_enabled = false;
    runtime.configure(request).unwrap();

    let replanned = runtime.get_engine_block_snapshot();
    assert!(!replanned.anticipative_planning_enabled);
    assert_eq!(replanned.inline_realtime_node_count, 1);
    assert_eq!(replanned.stateful_realtime_node_count, 2);
    assert_eq!(replanned.anticipative_eligible_node_count, 0);
    assert_eq!(replanned.phase_count, 2);
    assert_eq!(replanned.anticipative_phase_count, 0);
    assert_eq!(replanned.lane_count, 1);
    assert_eq!(replanned.anticipative_lane_count, 0);
    assert_eq!(
        replanned.lane_order,
        vec![signal_graph::GraphExecutionLane::Realtime]
    );
    assert_eq!(replanned.dispatch_count, 1);
    assert_eq!(replanned.dispatch_boundary_count, 0);
    assert_eq!(replanned.prepared_dispatch_count, 0);
    assert_eq!(replanned.realtime_dispatch_count, 1);
    assert_eq!(replanned.dispatch_handoff_count, 0);
    assert!(!replanned.prework_cache_enabled);
    assert_eq!(
        replanned.prework_cache_state,
        RuntimePreworkCacheState::Disabled
    );
    assert_eq!(
        replanned.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Disabled
    );
    assert_eq!(replanned.prework_cache_admissions, 0);
    assert_eq!(replanned.prework_cache_consumptions, 0);
    assert_eq!(replanned.prework_cache_valid_until_processing_epoch, None);
    assert_eq!(
        replanned.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
    );
    assert_eq!(replanned.prework_cache_invalidation_count, 0);
    assert_eq!(replanned.prework_cache_retirement_count, 0);
    assert_eq!(
        replanned.dispatch_order,
        vec![signal_graph::GraphExecutionLane::Realtime]
    );
    assert_eq!(
        replanned.phase_order,
        vec![
            signal_graph::GraphNodePlanningGroup::InlineRealtime,
            signal_graph::GraphNodePlanningGroup::StatefulRealtime,
        ]
    );
    assert_eq!(replanned.planned_nodes.len(), 3);
    assert_eq!(
        replanned
            .planned_nodes
            .iter()
            .map(|node| (node.node_id.as_str(), format!("{:?}", node.group)))
            .collect::<Vec<_>>(),
        vec![
            ("input", "InlineRealtime".into()),
            ("drive", "StatefulRealtime".into()),
            ("output", "StatefulRealtime".into()),
        ]
    );
}
