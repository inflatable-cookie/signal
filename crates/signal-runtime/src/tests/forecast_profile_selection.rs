use super::*;

#[test]
fn runtime_forecast_profile_change_keeps_realtime_scheduler_coherent() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 2,
            prepare_budget_per_cycle: 2,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set initial realtime policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-profile-change");
    runtime.start().expect("start runtime");

    let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    apply_current_forecast_block_state(&mut runtime, 1);
    runtime
        .process_engine_block(1, 1, first_block)
        .expect("process first realtime block");

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Server,
            target_window_blocks_override: Some(4),
        })
        .expect("switch forecast profile while running");

    let reprofiled = runtime.get_engine_block_snapshot();
    assert_eq!(
        reprofiled.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        reprofiled.prework_forecast_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        reprofiled.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Server)
    );
    assert_eq!(
        reprofiled.prework_forecast_policy_target_window_blocks,
        Some(4)
    );
    assert_eq!(
        reprofiled.prework_service_state,
        RuntimePreworkServiceState::Pending
    );
    assert!(reprofiled.prework_pending_target_count > 0);

    let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
    apply_current_forecast_block_state(&mut runtime, 2);
    let snapshot = runtime
        .process_engine_block(2, 2, second_block)
        .expect("process second realtime block after profile change")
        .snapshot;

    assert_eq!(
        snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Server)
    );
    assert_eq!(
        snapshot.prework_forecast_policy_target_window_blocks,
        Some(4)
    );
    assert!(snapshot
        .prework_cache_window_target_block_sequences
        .contains(&6));
    assert_eq!(snapshot.last_prework_service_processing_epoch, Some(2));
    assert!(matches!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Idle | RuntimePreworkServiceState::Pending
    ));
}

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

#[test]
fn runtime_apply_forecast_state_primes_window_and_applies_current_block_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-advance".into(),
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

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Server,
            target_window_blocks_override: None,
        })
        .expect("set prework forecast profile");

    let current_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .apply_forecast_state_for_block(1, current_sequence)
        .expect("apply forecast state");
    assert_eq!(admitted, 2);

    assert_eq!(
        runtime
            .applied_transport
            .as_ref()
            .map(|transport| transport.tempo_bpm),
        Some(122.0)
    );
    assert_eq!(
        runtime
            .applied_transport
            .as_ref()
            .map(|transport| transport.timeline_position_samples),
        Some(0)
    );
    assert_eq!(
        runtime.latest_parameter_epoch,
        runtime
            .forecast_parameter_batch_for_block(
                current_sequence,
                &SignalRuntime::prework_forecast_policy_for_profile(
                    RuntimePreworkForecastProfileSelection {
                        profile: RuntimePreworkForecastProfile::Server,
                        target_window_blocks_override: None,
                    },
                ),
            )
            .epoch
    );

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert!(snapshot.prework_forecast_policy_configured);
    assert_eq!(
        snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Server)
    );
    assert_eq!(
        snapshot.prework_forecast_profile_source,
        Some(RuntimePreworkForecastProfileSource::ExplicitSelection)
    );
    assert_eq!(
        snapshot.prework_forecast_profile_target_window_override,
        None
    );
    assert_eq!(
        snapshot.prework_forecast_policy_target_window_blocks,
        Some(2)
    );
    assert_eq!(snapshot.prework_cache_queue_depth, 2);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![1, 2]
    );
    assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));
}

#[test]
fn runtime_reconfigure_uses_role_default_after_requested_mode_is_reset() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Server,
            target_window_blocks_override: Some(3),
        })
        .expect("set prework forecast profile");
    assert_eq!(
        runtime.get_engine_block_snapshot().prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Server)
    );
    runtime
        .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
        .expect("reset requested mode to runtime role default");

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("reconfigure");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert!(snapshot.prework_forecast_policy_configured);
    assert_eq!(
        snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Local)
    );
    assert_eq!(
        snapshot.prework_forecast_profile_source,
        Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
    );
    assert_eq!(
        snapshot.prework_forecast_profile_target_window_override,
        None
    );
    assert_eq!(
        snapshot.prework_forecast_policy_target_window_blocks,
        Some(2)
    );

    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-role-default-after-reconfigure".into(),
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

    let block_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .apply_forecast_state_for_block(1, block_sequence)
        .expect("forecast apply should use runtime-role default");
    assert_eq!(admitted, 2);
}
