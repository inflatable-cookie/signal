use super::super::*;

#[test]
fn runtime_mixed_execution_class_graph_transition_reuses_schedule_widened_scope() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set widened mixed-graph policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:mixed-graph-before");
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:mixed-graph-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");
    runtime.start().expect("start runtime");

    let before = runtime.get_engine_block_snapshot();
    assert_eq!(before.last_prework_service_requested_cycles, 3);
    assert_eq!(before.last_prework_service_effective_cycles, 3);

    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:mixed-graph-after".into(),
            node_count: 4,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "state".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 96,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .expect("apply mixed execution-class graph");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
    assert!(snapshot.scheduler_topology.compatible);
    assert_eq!(snapshot.node_count, 4);
    assert_eq!(snapshot.plugin_backed_node_count, 1);
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(3)
    );
    assert!(snapshot.prework_cache_queue_depth >= before.prework_cache_queue_depth);
}

#[test]
fn runtime_mixed_execution_class_graph_churn_preserves_widened_scheduler_contract() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set mixed graph churn policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:mixed-graph-churn-a");
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:mixed-graph-churn".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");
    runtime.start().expect("start runtime");

    let projections = vec![
        GraphProjection {
            graph_id: "graph:runtime:mixed-graph-churn-b".into(),
            node_count: 4,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "state".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.7 }],
                },
                GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 48,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        },
        GraphProjection {
            graph_id: "graph:runtime:mixed-graph-churn-c".into(),
            node_count: 5,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "state-a".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "inline-a".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.95 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-a".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.85 }],
                },
                GraphNodeProjection {
                    node_id: "plugin-b".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.82 }],
                },
                GraphNodeProjection {
                    node_id: "latency-a".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 96,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        },
    ];

    let mut last_invalidation_count = runtime
        .get_engine_block_snapshot()
        .prework_cache_invalidation_count;
    for projection in projections {
        let expected_node_count = projection.node_count;
        let expected_plugin_count = projection
            .nodes
            .iter()
            .filter(|node| node.execution_class == GraphNodeExecutionClass::PluginBacked)
            .count();
        runtime
            .apply_graph_projection(projection)
            .expect("apply mixed execution-class graph projection");
        let snapshot = runtime.get_engine_block_snapshot();

        assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(
            snapshot.last_prework_service_effective_budget_per_cycle,
            Some(3)
        );
        assert_eq!(snapshot.node_count, expected_node_count);
        assert_eq!(snapshot.plugin_backed_node_count, expected_plugin_count);
        assert!(snapshot.prework_cache_invalidation_count >= last_invalidation_count);
        last_invalidation_count = snapshot.prework_cache_invalidation_count;
    }
}
