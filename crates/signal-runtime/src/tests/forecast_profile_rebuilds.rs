use super::*;

#[test]
fn runtime_forecast_plan_change_rebuild_uses_schedule_widened_service_scope() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-plan-change-schedule-widened".into(),
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
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:forecast-plan-change-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");
    runtime.start().expect("start runtime");

    let current_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .apply_forecast_state_for_block(1, current_sequence)
        .expect("prime role-default prework");
    assert!(admitted >= 2);
    let before = runtime.get_engine_block_snapshot();

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Server,
            target_window_blocks_override: Some(6),
        })
        .expect("switch widened forecast profile");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
    assert!((1..=3).contains(&snapshot.last_prework_service_cycle_count));
    assert!(snapshot.prework_cache_window_target_count > before.prework_cache_window_target_count);
    assert!(snapshot.prework_cache_invalidation_count >= 1);
    assert!(snapshot.prework_cache_retirement_count >= 1);
    assert!(snapshot.prework_cache_queue_depth >= before.prework_cache_queue_depth);
    assert!(snapshot.prework_pending_target_count <= before.prework_pending_target_count);
}

#[test]
fn runtime_preserves_compatible_queued_prework_when_forecast_mode_changes_but_plan_matches() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-plan-compatible".into(),
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

    let current_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .apply_forecast_state_for_block(1, current_sequence)
        .expect("prime local role-default prework");
    assert_eq!(admitted, 2);

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Local,
            target_window_blocks_override: None,
        })
        .expect("switch to explicit profile with matching plan");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 2);
    assert_eq!(snapshot.prework_cache_invalidation_count, 0);
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
}

#[test]
fn runtime_selectively_trims_queued_prework_when_forecast_window_shrinks() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-window-shrink".into(),
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

    let current_sequence = runtime.allocate_block_sequence();
    runtime
        .apply_forecast_state_for_block(1, current_sequence)
        .expect("prime local role-default prework");
    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_cache_queue_depth,
        2
    );

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Local,
            target_window_blocks_override: Some(1),
        })
        .expect("shrink local forecast window");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 1);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![1]
    );
    assert_eq!(
        snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
    );
    assert_eq!(
        snapshot.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
    );
}

#[test]
fn runtime_configure_with_anticipative_disabled_enters_disabled_forecast_mode() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, false);

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::Disabled
    );
    assert!(snapshot.prework_forecast_policy_configured);
    assert_eq!(
        snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Server)
    );
    assert_eq!(
        snapshot.prework_forecast_profile_source,
        Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
    );
    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Local,
            target_window_blocks_override: Some(3),
        })
        .expect("store explicit profile while anticipative planning is off");
    let explicit_snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        explicit_snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::ExplicitProfile
    );
    assert_eq!(
        explicit_snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::Disabled
    );
    assert_eq!(
        explicit_snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Local)
    );
    assert_eq!(
        explicit_snapshot.prework_forecast_profile_target_window_override,
        Some(3)
    );
}

#[test]
fn runtime_retires_queued_prework_when_effective_mode_drops_to_disabled() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:disable-retire".into(),
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

    let current_sequence = runtime.allocate_block_sequence();
    runtime
        .apply_forecast_state_for_block(1, current_sequence)
        .expect("prime role-default prework");
    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_cache_queue_depth,
        2
    );

    let mut disabled_request = RuntimeConfigRequest::new(48_000, 256);
    disabled_request.anticipative_enabled = false;
    runtime
        .configure(disabled_request)
        .expect("disable anticipative");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 0);
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::Disabled
    );
    assert_eq!(
        snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
    );
    assert_eq!(
        snapshot.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::RuntimeReconfigured)
    );
}

#[test]
fn runtime_apply_graph_projection_primes_prework_window_from_stored_forecast_state() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:auto-prime-on-graph-apply".into(),
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
        .expect("apply graph projection");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 2);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences.len(),
        2
    );
    assert!(
        snapshot.prework_cache_window_target_block_sequences[0]
            < snapshot.prework_cache_window_target_block_sequences[1]
    );
    assert_eq!(
        snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
}

#[test]
fn runtime_start_rebuilds_prework_window_after_runtime_stop() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:restart-rebuild".into(),
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
        .expect("apply graph projection");
    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_cache_queue_depth,
        2
    );

    runtime.start().expect("start runtime");
    runtime
        .stop(StopReason::UserRequested)
        .expect("stop runtime");
    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_cache_queue_depth,
        0
    );

    runtime.start().expect("restart runtime");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 2);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences.len(),
        2
    );
    assert!(
        snapshot.prework_cache_window_target_block_sequences[0]
            < snapshot.prework_cache_window_target_block_sequences[1]
    );
    assert_eq!(
        snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
}
