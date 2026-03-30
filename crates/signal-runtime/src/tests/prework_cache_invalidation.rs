use super::*;

#[test]
fn runtime_prework_cache_expires_by_block_sequence_window() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:block-expiry".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .unwrap();

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 31);
    let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
    let second = runtime.process_engine_block(1, 2, block.clone()).unwrap();
    let third = runtime.process_engine_block(1, 3, block.clone()).unwrap();
    let fourth = runtime.process_engine_block(1, 4, block).unwrap();

    assert_eq!(first.snapshot.prework_cache_misses, 1);
    assert_eq!(first.snapshot.prework_cache_consumptions, 1);
    assert_eq!(second.snapshot.prework_cache_hits, 1);
    assert_eq!(third.snapshot.prework_cache_hits, 2);
    assert_eq!(third.snapshot.prework_cache_consumptions, 3);
    assert_eq!(
        third.snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Exhausted
    );
    assert_eq!(third.snapshot.prework_cache_remaining_valid_blocks, Some(0));
    assert_eq!(
        fourth.snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::BlockSequenceExpired)
    );
    assert_eq!(
        fourth.snapshot.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::BlockSequenceExpired)
    );
    assert_eq!(fourth.snapshot.last_prework_retired_unconsumed, Some(false));
    assert_eq!(fourth.snapshot.prework_cache_retirement_count, 1);
    assert_eq!(fourth.snapshot.prework_cache_consumed_retirement_count, 1);
    assert_eq!(fourth.snapshot.prework_cache_unconsumed_retirement_count, 0);
    assert_eq!(fourth.snapshot.prework_cache_misses, 2);
    assert_eq!(
        fourth.snapshot.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(fourth.snapshot.prework_cache_consumptions, 4);
    assert_eq!(
        fourth.snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Fresh
    );
    assert_eq!(
        fourth.snapshot.prework_cache_valid_until_block_sequence,
        Some(6)
    );
    assert_eq!(fourth.snapshot.last_prework_source_block_sequence, Some(4));
}

#[test]
fn runtime_invalidates_prework_cache_on_parameter_and_transport_changes() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:invalidate");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 21);
    let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
    assert_eq!(
        first.snapshot.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(first.snapshot.prework_cache_admissions, 1);
    assert_eq!(first.snapshot.prework_cache_consumptions, 1);
    assert_eq!(
        first.snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Fresh
    );

    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(1, 2, 1, block.clone(), None, None)
        .unwrap());

    runtime
        .apply_parameter_batch(ParameterBatch {
            epoch: runtime.projection_epoch().saturating_add(1),
            events: vec![ParameterEvent {
                target: "invalidate.param".into(),
                sample_offset: 0,
                normalized_value: 0.25,
            }],
        })
        .unwrap();
    let after_parameter = runtime.get_engine_block_snapshot();
    assert_eq!(
        after_parameter.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(after_parameter.last_prework_invalidation_reason, None);

    let second = runtime.process_engine_block(2, 2, block.clone()).unwrap();
    assert_eq!(second.snapshot.prework_cache_misses, 2);
    assert!(!second.snapshot.last_prework_cache_hit);
    assert_eq!(
        second.snapshot.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(second.snapshot.prework_cache_admissions, 2);
    assert_eq!(second.snapshot.prework_cache_consumptions, 2);
    assert_eq!(
        second.snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Fresh
    );

    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(2, 3, 2, block.clone(), None, None)
        .unwrap());

    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 512,
            tempo_bpm: 130.0,
            loop_state: None,
        })
        .unwrap();
    let after_transport = runtime.get_engine_block_snapshot();
    assert_eq!(
        after_transport.prework_cache_state,
        RuntimePreworkCacheState::Invalidated
    );
    assert_eq!(
        after_transport.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::TransportStarted)
    );
    assert_eq!(after_transport.prework_cache_invalidation_count, 2);
    assert_eq!(after_transport.prework_cache_retirement_count, 2);
    assert_eq!(
        after_transport.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::TransportStarted)
    );
    assert_eq!(after_transport.last_prework_retired_unconsumed, Some(false));
    assert_eq!(after_transport.prework_cache_unconsumed_retirement_count, 0);
    assert_eq!(after_transport.prework_cache_consumed_retirement_count, 2);
    assert_eq!(
        after_transport.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Invalidated
    );
    assert_eq!(
        after_transport.prework_cache_valid_until_processing_epoch,
        None
    );
}

#[test]
fn runtime_invalidation_heavy_transition_stress_preserves_widened_scheduler_receipts() {
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
        .expect("set transition stress policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:transition-stress");
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:transition-stress".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");
    runtime.start().expect("start runtime");

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 91);
    let transitions = vec![
        TransportProjection {
            playing: true,
            timeline_position_samples: 64,
            tempo_bpm: 120.0,
            loop_state: None,
        },
        TransportProjection {
            playing: true,
            timeline_position_samples: 512,
            tempo_bpm: 120.0,
            loop_state: None,
        },
        TransportProjection {
            playing: true,
            timeline_position_samples: 520,
            tempo_bpm: 130.0,
            loop_state: None,
        },
        TransportProjection {
            playing: true,
            timeline_position_samples: 528,
            tempo_bpm: 130.0,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 256,
                end_samples: 1024,
            }),
        },
        TransportProjection {
            playing: false,
            timeline_position_samples: 536,
            tempo_bpm: 130.0,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 256,
                end_samples: 1024,
            }),
        },
    ];

    for (index, projection) in transitions.into_iter().enumerate() {
        runtime
            .apply_parameter_batch(ParameterBatch {
                epoch: runtime.projection_epoch().saturating_add(50 + index as u64),
                events: vec![ParameterEvent {
                    target: format!("stress.param.{index}"),
                    sample_offset: 0,
                    normalized_value: (index as f32) * 0.1,
                }],
            })
            .expect("apply stress parameter batch");
        runtime
            .apply_transport_projection(projection)
            .expect("apply stress transport projection");

        let result = runtime
            .process_engine_block((index + 1) as u64, (index + 1) as u64, block.clone())
            .expect("process stress transition block");

        assert_eq!(
            result.snapshot.scheduler_topology.schedule_stream_count,
            Some(3)
        );
        assert_eq!(result.snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(result.snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(
            result
                .snapshot
                .last_prework_service_effective_budget_per_cycle,
            Some(3)
        );
    }

    let snapshot = runtime.get_engine_block_snapshot();
    assert!(snapshot.prework_cache_invalidation_count >= 5);
    assert!(snapshot.prework_cache_retirement_count >= 5);
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
    assert_eq!(
        runtime.get_timeline_snapshot().last_transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Stopped)
    );
}
