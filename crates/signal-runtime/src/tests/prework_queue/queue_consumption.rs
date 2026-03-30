use super::super::*;

#[test]
fn runtime_prework_queue_consumes_multiple_future_blocks_in_order() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:queued-prework-pipeline".into(),
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

    let block2 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 12);
    let block3 = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 13);
    let batch2 = ParameterBatch {
        epoch: runtime.projection_epoch().saturating_add(3),
        events: vec![ParameterEvent {
            target: "engine.local.drive".into(),
            sample_offset: 0,
            normalized_value: 0.5,
        }],
    };
    let batch3 = ParameterBatch {
        epoch: runtime.projection_epoch().saturating_add(4),
        events: vec![ParameterEvent {
            target: "engine.local.drive".into(),
            sample_offset: 0,
            normalized_value: 0.65,
        }],
    };
    let transport2 = TransportProjection {
        playing: true,
        timeline_position_samples: 72,
        tempo_bpm: 120.0,
        loop_state: None,
    };
    let transport3 = TransportProjection {
        playing: true,
        timeline_position_samples: 80,
        tempo_bpm: 120.0,
        loop_state: None,
    };

    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(
            1,
            2,
            1,
            block2.clone(),
            Some(batch2.epoch),
            Some(transport2),
        )
        .unwrap());
    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(
            1,
            3,
            1,
            block3.clone(),
            Some(batch3.epoch),
            Some(transport3),
        )
        .unwrap());

    let primed = runtime.get_engine_block_snapshot();
    assert_eq!(primed.prework_cache_queue_capacity, 3);
    assert_eq!(primed.prework_cache_queue_depth, 2);
    assert_eq!(primed.prework_cache_peak_queue_depth, 2);
    assert_eq!(primed.prework_cache_queued_admissions, 2);
    assert_eq!(primed.last_prework_admission_block_sequence, Some(3));

    runtime.apply_parameter_batch(batch2).unwrap();
    runtime.apply_transport_projection(transport2).unwrap();
    let second = runtime.process_engine_block(1, 2, block2).unwrap();
    assert_eq!(second.snapshot.prework_cache_hits, 1);
    assert_eq!(second.snapshot.prework_cache_queued_consumptions, 1);
    assert_eq!(second.snapshot.prework_cache_queue_depth, 2);
    assert_eq!(
        second.snapshot.last_prework_consumption_block_sequence,
        Some(2)
    );
    assert_eq!(
        second.snapshot.last_prework_consumed_from_block_sequence,
        Some(1)
    );

    runtime.apply_parameter_batch(batch3).unwrap();
    runtime.apply_transport_projection(transport3).unwrap();
    let third = runtime.process_engine_block(1, 3, block3).unwrap();
    assert_eq!(third.snapshot.prework_cache_hits, 2);
    assert_eq!(third.snapshot.prework_cache_queued_consumptions, 2);
    assert_eq!(third.snapshot.prework_cache_queue_depth, 1);
    assert_eq!(
        third.snapshot.last_prework_consumption_block_sequence,
        Some(3)
    );
    assert_eq!(
        third.snapshot.last_prework_consumed_from_block_sequence,
        Some(1)
    );
}

#[test]
fn runtime_prework_queue_evicts_oldest_future_entry_when_capacity_is_exceeded() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:queued-prework-eviction".into(),
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

    for offset in 0..4 {
        let target_block_sequence = 2 + offset;
        let block = synthetic_stereo_block(
            SampleRate(48_000),
            FrameCount(8),
            12 + target_block_sequence,
        );
        let batch_epoch = runtime
            .projection_epoch()
            .saturating_add(3)
            .saturating_add(offset);
        let transport = TransportProjection {
            playing: true,
            timeline_position_samples: 72 + (offset as i64 * 8),
            tempo_bpm: 120.0,
            loop_state: None,
        };
        assert!(runtime
            .prepare_engine_prework_for_block_with_future_state(
                1,
                target_block_sequence,
                1,
                block,
                Some(batch_epoch),
                Some(transport),
            )
            .unwrap());
    }

    let primed = runtime.get_engine_block_snapshot();
    assert_eq!(primed.prework_cache_queue_capacity, 3);
    assert_eq!(primed.prework_cache_queue_depth, 3);
    assert_eq!(primed.prework_cache_peak_queue_depth, 3);
    assert_eq!(primed.prework_cache_queued_admissions, 4);
    assert_eq!(primed.prework_cache_invalidation_count, 1);
    assert_eq!(
        primed.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::QueueCapacityExceeded)
    );
    assert_eq!(
        primed.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::QueueCapacityExceeded)
    );
    assert_eq!(primed.last_prework_retired_unconsumed, Some(true));
}
