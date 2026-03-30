use super::super::*;

#[test]
fn runtime_primes_prework_window_from_forecast_policy() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-prework".into(),
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

    let policy = RuntimePreworkForecastPolicy {
        target_window_blocks: 2,
        prepare_budget_per_cycle: 2,
        buffer_seed_offset: 17,
        transport_playing: true,
        transport_tempo_bpm: 122.0,
        transport_loop_length_blocks: 24,
        parameter_target: "engine.server.balance".into(),
        parameter_cycle_length: 6,
    };

    let current_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
        .expect("prime forecast window");
    assert_eq!(admitted, 2);

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 2);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![1, 2]
    );
    assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
    assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(0));

    let transport = runtime.forecast_transport_projection_for_block(2, &policy);
    assert_eq!(transport.tempo_bpm, 122.0);
    assert_eq!(transport.timeline_position_samples, 512);

    let batch = runtime.forecast_parameter_batch_for_block(2, &policy);
    assert_eq!(batch.epoch, 4);
    assert_eq!(batch.events.len(), 1);
    assert_eq!(batch.events[0].target, "engine.server.balance");
    assert!((batch.events[0].normalized_value - 0.4).abs() < 1.0e-6);
}

#[test]
fn runtime_forecast_policy_limits_prework_window_depth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-window-limit".into(),
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

    let policy = RuntimePreworkForecastPolicy {
        target_window_blocks: 1,
        prepare_budget_per_cycle: 1,
        buffer_seed_offset: 0,
        transport_playing: true,
        transport_tempo_bpm: 126.0,
        transport_loop_length_blocks: 16,
        parameter_target: "engine.local.drive".into(),
        parameter_cycle_length: 8,
    };

    let current_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
        .expect("prime limited forecast window");
    assert_eq!(admitted, 1);

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 1);
    assert_eq!(snapshot.prework_pending_target_count, 0);
    assert_eq!(snapshot.prework_cache_window_target_count, 1);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![1]
    );
}
