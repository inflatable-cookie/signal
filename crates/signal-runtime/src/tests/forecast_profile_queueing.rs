use super::*;

#[test]
fn runtime_selects_forecast_profile_with_target_window_override() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-profile-override".into(),
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
            target_window_blocks_override: Some(4),
        })
        .expect("set prework forecast profile");

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
        Some(4)
    );
    assert_eq!(
        snapshot.prework_forecast_policy_target_window_blocks,
        Some(4)
    );

    let block_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .apply_forecast_state_for_block(1, block_sequence)
        .expect("apply forecast state");
    assert_eq!(admitted, 4);
}

#[test]
fn runtime_configure_seeds_default_forecast_profile_from_runtime_role() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

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
        Some(RuntimePreworkForecastProfile::Server)
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
}

#[test]
fn runtime_can_disable_and_restore_role_default_forecast_mode() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-mode-toggle".into(),
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
        .set_prework_forecast_mode(RuntimePreworkForecastMode::Disabled)
        .expect("disable prework forecast mode");
    let disabled_snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        disabled_snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::Disabled
    );
    assert_eq!(
        disabled_snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::Disabled
    );
    assert!(disabled_snapshot.prework_forecast_policy_configured);
    assert_eq!(
        disabled_snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::PlanningDisabled)
    );

    let block_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .apply_forecast_state_for_block(1, block_sequence)
        .expect("apply forecast state while disabled");
    assert_eq!(admitted, 0);

    runtime
        .set_prework_forecast_mode(RuntimePreworkForecastMode::RuntimeRoleDefault)
        .expect("restore role-default forecast mode");
    let restored_snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        restored_snapshot.prework_forecast_requested_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        restored_snapshot.prework_forecast_mode,
        RuntimePreworkForecastMode::RuntimeRoleDefault
    );
    assert_eq!(
        restored_snapshot.prework_forecast_profile,
        Some(RuntimePreworkForecastProfile::Local)
    );
    assert_eq!(
        restored_snapshot.prework_forecast_profile_source,
        Some(RuntimePreworkForecastProfileSource::RuntimeRoleDefault)
    );
    assert_eq!(
        restored_snapshot.prework_forecast_policy_target_window_blocks,
        Some(2)
    );

    let next_block_sequence = runtime.allocate_block_sequence();
    let restored_admitted = runtime
        .apply_forecast_state_for_block(2, next_block_sequence)
        .expect("apply forecast state after restore");
    assert_eq!(restored_admitted, 2);
}

#[test]
fn runtime_retires_queued_prework_when_forecast_profile_changes() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-plan-change".into(),
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
    assert_eq!(
        runtime
            .get_engine_block_snapshot()
            .prework_cache_queue_depth,
        2
    );

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Server,
            target_window_blocks_override: Some(3),
        })
        .expect("switch explicit profile");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 2);
    assert_eq!(snapshot.prework_pending_target_count, 1);
    assert_eq!(snapshot.prework_cache_window_target_count, 3);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![1, 2, 3]
    );
    assert_eq!(
        snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::ForecastPlanChanged)
    );
    assert_eq!(
        snapshot.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::ForecastPlanChanged)
    );
    assert_eq!(snapshot.prework_cache_queued_admissions, 4);
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
fn runtime_rebuilds_missing_queued_prework_when_forecast_window_expands() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-window-expand".into(),
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
            target_window_blocks_override: Some(3),
        })
        .expect("expand local forecast window");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 3);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![1, 2, 3]
    );
    assert_eq!(snapshot.prework_cache_invalidation_count, 0);
    assert_eq!(snapshot.prework_cache_retirement_count, 0);
}
