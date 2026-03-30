use super::*;

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

#[test]
fn runtime_constrained_anticipative_window_caps_widened_service_realization() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 1,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set constrained widened forecast policy");
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:constrained-window-widened",
        &["track:drums", "track:bass"],
        false,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:constrained-window-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened constrained schedule");
    runtime.start().expect("start runtime");

    for block_sequence in 1..=3u64 {
        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), block_sequence);
        apply_current_forecast_block_state(&mut runtime, block_sequence);
        let snapshot = runtime
            .process_engine_block(block_sequence, block_sequence, block)
            .expect("process constrained widened block")
            .snapshot;

        assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
        assert!(snapshot.scheduler_topology.compatible);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_prepared_targets, 1);
        assert!(snapshot.prework_cache_window_target_count <= 2);
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert!(snapshot.prework_cache_peak_queue_depth <= 2);
    }
}

#[test]
fn runtime_forecast_runner_leaves_pending_targets_when_budget_is_smaller_than_window() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
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
        .expect("set bounded raw forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-runner-budget".into(),
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
    assert_eq!(runtime.engine.prework_queue.len(), 1);
    assert!(runtime.engine.pending_prework_targets.len() > 1);

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 1);
    assert!(snapshot.prework_pending_target_count > 1);
    assert_eq!(snapshot.prework_cache_window_target_count, 8);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![1, 2, 3, 4, 5, 6, 7, 8]
    );

    runtime.start().expect("start runtime");
    let started = runtime.get_engine_block_snapshot();
    assert_eq!(started.prework_cache_queue_depth, 2);
    assert!(started.prework_pending_target_count > 0);

    let serviced_once = runtime
        .service_prework_lane(1, 1)
        .expect("service pending prework once");
    assert_eq!(serviced_once, 1);
    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 3);
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_service_cycle_count >= 1);
    assert!(snapshot.prework_service_prepared_targets >= 1);
    assert_eq!(snapshot.last_prework_service_processing_epoch, Some(1));
    assert_eq!(snapshot.last_prework_service_cycle_count, 1);
    assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
    assert!(snapshot.last_prework_service_prepared_targets >= 1);

    let serviced_again = runtime
        .service_prework_lane(1, 2)
        .expect("service pending prework until idle");
    assert!(serviced_again >= 1);
    let snapshot = runtime.get_engine_block_snapshot();
    assert!(snapshot.prework_cache_queue_depth >= 3);
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_service_cycle_count >= 2);
    assert!(snapshot.prework_service_prepared_targets >= 3);
    assert_eq!(snapshot.last_prework_service_cycle_count, 2);
    assert_eq!(snapshot.last_prework_service_prepared_targets, 2);
}

#[test]
fn runtime_prework_service_lane_enters_starved_state_when_budget_is_zero() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 8,
            prepare_budget_per_cycle: 0,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set zero-budget forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-runner-starved".into(),
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

    let paused = runtime.get_engine_block_snapshot();
    assert_eq!(
        paused.prework_service_state,
        RuntimePreworkServiceState::Paused
    );
    assert!(paused.prework_pending_target_count > 0);

    runtime.start().expect("start runtime");
    runtime
        .service_prework_lane(1, 1)
        .expect("service prework lane with zero effective budget");
    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Starved
    );
    assert_eq!(snapshot.prework_cache_queue_depth, 0);
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_service_starvation_count >= 1);
}

#[test]
fn runtime_prework_service_lane_resumes_after_start() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 3,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set bounded forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-runner-resume".into(),
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

    let paused = runtime.get_engine_block_snapshot();
    assert_eq!(
        paused.prework_service_state,
        RuntimePreworkServiceState::Paused
    );
    assert!(paused.prework_pending_target_count > 0);

    runtime.start().expect("start runtime");

    let resumed = runtime.get_engine_block_snapshot();
    assert!(matches!(
        resumed.prework_service_state,
        RuntimePreworkServiceState::Pending | RuntimePreworkServiceState::Idle
    ));
    assert!(resumed.prework_service_pause_count >= 1);
    assert!(resumed.prework_service_resume_count >= 1);
    assert!(resumed.prework_service_prepared_targets >= 1);
}

#[test]
fn runtime_prework_service_lane_yields_under_critical_pressure() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 3,
            prepare_budget_per_cycle: 2,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set bounded forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-runner-critical".into(),
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
        .set_prework_service_pressure(RuntimePreworkServicePressure::Critical)
        .expect("set critical prework pressure");
    runtime
        .service_prework_lane(1, 3)
        .expect("service prework lane under critical pressure");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Yielding
    );
    assert_eq!(
        snapshot.prework_service_pressure,
        RuntimePreworkServicePressure::Critical
    );
    assert!(snapshot.prework_pending_target_count > 0);
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(0)
    );
    assert!(snapshot.prework_service_yield_count >= 1);
}

#[test]
fn runtime_prework_service_lane_throttles_under_elevated_pressure() {
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
        .expect("set bounded forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:forecast-runner-elevated".into(),
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
        .expect("service prework lane under elevated pressure");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_service_pressure,
        RuntimePreworkServicePressure::Elevated
    );
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert!(snapshot.last_prework_service_effective_cycles <= 1);
    assert!(matches!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(0 | 1)
    ));
    assert!(snapshot.prework_service_throttle_count >= 1);
    assert!(
        snapshot.prework_service_prepared_targets >= 1 || snapshot.prework_service_yield_count >= 1
    );
}
