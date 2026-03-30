use super::super::*;

#[test]
fn runtime_reuses_prework_cache_for_matching_adjacent_block() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:cache".into(),
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

    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 11);
    let first = runtime.process_engine_block(1, 1, block.clone()).unwrap();
    let second = runtime.process_engine_block(2, 2, block).unwrap();

    assert_eq!(first.snapshot.prework_cache_hits, 0);
    assert_eq!(first.snapshot.prework_cache_misses, 1);
    assert_eq!(
        first.snapshot.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(first.snapshot.prework_cache_admissions, 1);
    assert_eq!(first.snapshot.prework_cache_consumptions, 1);
    assert_eq!(first.snapshot.prework_cache_queued_admissions, 0);
    assert_eq!(first.snapshot.prework_cache_queued_consumptions, 0);
    assert_eq!(
        first.snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Fresh
    );
    assert_eq!(first.snapshot.prework_cache_remaining_valid_blocks, Some(2));
    assert!(!first.snapshot.last_prework_cache_hit);
    assert_eq!(
        first.snapshot.last_prework_admitted_from_block_sequence,
        Some(1)
    );
    assert_eq!(
        first.snapshot.last_prework_consumed_from_block_sequence,
        Some(1)
    );
    assert_eq!(
        first.snapshot.prework_cache_valid_until_processing_epoch,
        Some(2)
    );
    assert_eq!(
        first.snapshot.prework_cache_valid_until_block_sequence,
        Some(3)
    );
    assert_eq!(second.snapshot.prework_cache_hits, 1);
    assert_eq!(second.snapshot.prework_cache_misses, 1);
    assert_eq!(
        second.snapshot.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(second.snapshot.prework_cache_admissions, 1);
    assert_eq!(second.snapshot.prework_cache_consumptions, 2);
    assert_eq!(second.snapshot.prework_cache_queued_admissions, 0);
    assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
    assert_eq!(
        second.snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Expiring
    );
    assert_eq!(
        second.snapshot.prework_cache_remaining_valid_blocks,
        Some(1)
    );
    assert!(second.snapshot.last_prework_cache_hit);
    assert_eq!(
        second.snapshot.last_prework_source_processing_epoch,
        Some(1)
    );
    assert_eq!(second.snapshot.last_prework_source_block_sequence, Some(1));
    assert_eq!(
        second.snapshot.last_prework_admission_processing_epoch,
        Some(1)
    );
    assert_eq!(
        second.snapshot.last_prework_admission_block_sequence,
        Some(1)
    );
    assert_eq!(
        second.snapshot.last_prework_consumption_processing_epoch,
        Some(2)
    );
    assert_eq!(
        second.snapshot.last_prework_consumption_block_sequence,
        Some(2)
    );
    assert_eq!(
        second.snapshot.last_prework_admitted_from_block_sequence,
        Some(1)
    );
    assert_eq!(
        second.snapshot.last_prework_consumed_from_block_sequence,
        Some(1)
    );
    assert_eq!(
        second.snapshot.prework_cache_valid_until_processing_epoch,
        Some(2)
    );
    assert_eq!(
        second.snapshot.prework_cache_valid_until_block_sequence,
        Some(3)
    );
    assert_eq!(second.snapshot.prepared_dispatch_count, 1);
    assert_eq!(second.snapshot.realtime_dispatch_count, 1);
}

#[test]
fn runtime_consumes_primed_prework_for_the_next_block() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:queued-prework".into(),
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
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 64,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();

    let next_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
    let next_batch = ParameterBatch {
        epoch: runtime.projection_epoch().saturating_add(3),
        events: vec![ParameterEvent {
            target: "engine.local.drive".into(),
            sample_offset: 0,
            normalized_value: 0.5,
        }],
    };
    let next_transport = TransportProjection {
        playing: true,
        timeline_position_samples: 72,
        tempo_bpm: 120.0,
        loop_state: None,
    };
    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(
            1,
            2,
            1,
            next_block.clone(),
            Some(next_batch.epoch),
            Some(next_transport),
        )
        .unwrap());

    let primed = runtime.get_engine_block_snapshot();
    assert_eq!(primed.prework_cache_admissions, 1);
    assert_eq!(primed.prework_cache_queued_admissions, 1);
    assert_eq!(primed.last_prework_admission_block_sequence, Some(2));
    assert_eq!(primed.last_prework_admitted_from_block_sequence, Some(1));

    runtime.apply_parameter_batch(next_batch).unwrap();
    runtime.apply_transport_projection(next_transport).unwrap();
    let consumed = runtime.process_engine_block(1, 2, next_block).unwrap();
    assert_eq!(consumed.snapshot.prework_cache_hits, 1);
    assert_eq!(consumed.snapshot.prework_cache_admissions, 1);
    assert_eq!(consumed.snapshot.prework_cache_consumptions, 1);
    assert_eq!(consumed.snapshot.prework_cache_queued_admissions, 1);
    assert_eq!(consumed.snapshot.prework_cache_queued_consumptions, 1);
    assert!(consumed.snapshot.last_prework_cache_hit);
    assert_eq!(consumed.snapshot.last_prework_invalidation_reason, None);
    assert_eq!(
        consumed.snapshot.last_prework_admitted_from_block_sequence,
        Some(1)
    );
    assert_eq!(
        consumed.snapshot.last_prework_consumed_from_block_sequence,
        Some(1)
    );
    assert_eq!(
        consumed.snapshot.last_prework_consumption_block_sequence,
        Some(2)
    );
    assert_eq!(
        consumed
            .snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.timeline_position_samples),
        Some(72)
    );
    assert_eq!(
        consumed
            .snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.transport_tempo_bpm),
        Some(120.0)
    );
}
