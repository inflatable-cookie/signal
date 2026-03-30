use super::*;

#[test]
fn runtime_elevated_pressure_preserves_deferred_prework_targets() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 3,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set elevated forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-runner-backlog-classes".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 24,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .unwrap();
    runtime.start().expect("start runtime");
    seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated prework pressure");

    runtime
        .service_prework_lane(1, 3)
        .expect("service elevated lane first cycle");
    runtime
        .service_prework_lane(2, 3)
        .expect("service elevated lane second cycle");
    runtime
        .service_prework_lane(3, 3)
        .expect("service elevated lane third cycle");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::Balanced
    );
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Yielding
    );
    assert_eq!(snapshot.prework_pending_immediate_target_count, 0);
    assert_eq!(snapshot.prework_pending_near_term_target_count, 0);
    assert!(snapshot.prework_pending_deferred_target_count > 0);
    assert_eq!(
        snapshot.prework_pending_target_count,
        snapshot.prework_pending_deferred_target_count
    );
    assert!(snapshot.prework_service_yield_count >= 1);
}

#[test]
fn runtime_latency_focused_graph_expands_elevated_pressure_service_scope() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 3,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set latency-focused forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:latency-focused-prework-priority".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 96,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .unwrap();
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated prework pressure");

    runtime
        .service_prework_lane(1, 3)
        .expect("service elevated lane first cycle");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::LatencyFocused
    );
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(2)
    );
    assert_eq!(snapshot.prework_pending_target_count, 0);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Idle
    );
    assert_eq!(
        snapshot.last_prework_serviced_backlog_class,
        Some(RuntimePreworkBacklogClass::Deferred)
    );
    assert!(snapshot.prework_service_throttle_count >= 1);
}

#[test]
fn runtime_plugin_backed_graph_constrains_elevated_pressure_service_scope() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 3,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set plugin-constrained forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-constrained-prework-priority".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
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
        .unwrap();
    runtime.set_active_plugin_sandboxes(1);
    runtime.start().expect("start runtime");
    seed_pending_prework_targets(&mut runtime, 1, &[7, 8]);
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated prework pressure");

    runtime
        .service_prework_lane(1, 3)
        .expect("service elevated lane first cycle");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.plugin_backed_node_count, 1);
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::PluginConstrained
    );
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_service_throttle_count >= 1);
}

#[test]
fn runtime_plugin_backed_policy_tracks_active_plugin_sandbox_count() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-policy-tracking".into(),
            node_count: 2,
            nodes: vec![
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
        .unwrap();

    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::LatencyFocused
    );
    runtime.set_active_plugin_sandboxes(1);
    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::PluginConstrained
    );
    runtime.set_active_plugin_sandboxes(0);
    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::LatencyFocused
    );
}

#[test]
fn runtime_plugin_constrained_lane_yields_when_multiple_plugin_sandboxes_are_active() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 6,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 32,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set plugin-constrained forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-gate".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
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
        .unwrap();
    runtime.set_active_plugin_sandboxes(2);
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated prework pressure");

    runtime
        .service_prework_lane(1, 3)
        .expect("service elevated lane");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::PluginConstrained
    );
    assert_eq!(snapshot.prework_service_active_plugin_sandboxes, 2);
    assert!(snapshot.prework_service_plugin_gate_active);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Yielding
    );
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_service_yield_count >= 1);
}

#[test]
fn runtime_schedule_widened_plugin_gate_yields_without_servicing() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    let policy = RuntimePreworkForecastPolicy {
        target_window_blocks: 6,
        prepare_budget_per_cycle: 1,
        buffer_seed_offset: 0,
        transport_playing: true,
        transport_tempo_bpm: 126.0,
        transport_loop_length_blocks: 32,
        parameter_target: "engine.local.drive".into(),
        parameter_cycle_length: 8,
    };
    runtime
        .set_prework_forecast_policy(policy.clone())
        .expect("set widened plugin-constrained forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-gate-schedule-widened".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
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
        .unwrap();
    runtime.set_active_plugin_sandboxes(2);
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated prework pressure");
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:plugin-gate-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");
    let current_sequence = runtime.allocate_block_sequence();

    let admitted = runtime
        .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
        .expect("prime widened plugin-gated window");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(admitted, 0);
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(0)
    );
    assert!(snapshot.prework_service_plugin_gate_active);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Yielding
    );
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_service_yield_count >= 1);
}
