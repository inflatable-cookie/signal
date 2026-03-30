use super::super::*;

#[test]
fn runtime_reuses_existing_future_queue_entry_when_target_state_matches() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:queued-prework-reuse".into(),
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
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                },
            ],
        })
        .unwrap();

    let transport = TransportProjection {
        playing: true,
        timeline_position_samples: 96,
        tempo_bpm: 120.0,
        loop_state: None,
    };
    let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 41);
    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(
            1,
            2,
            1,
            block.clone(),
            Some(9),
            Some(transport),
        )
        .unwrap());
    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(
            2,
            2,
            2,
            block,
            Some(9),
            Some(transport),
        )
        .unwrap());

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 1);
    assert_eq!(snapshot.prework_cache_admissions, 1);
    assert_eq!(snapshot.prework_cache_queued_admissions, 1);
    assert_eq!(snapshot.prework_cache_invalidation_count, 0);
    assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
    assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(1));
}

#[test]
fn runtime_replaces_future_queue_entry_when_target_state_changes() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:queued-prework-replace".into(),
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
                    latency_samples: 12,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.65 }],
                },
            ],
        })
        .unwrap();

    let first_transport = TransportProjection {
        playing: true,
        timeline_position_samples: 96,
        tempo_bpm: 120.0,
        loop_state: None,
    };
    let replacement_transport = TransportProjection {
        playing: true,
        timeline_position_samples: 104,
        tempo_bpm: 121.0,
        loop_state: None,
    };
    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(
            1,
            2,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 42),
            Some(9),
            Some(first_transport),
        )
        .unwrap());
    assert!(runtime
        .prepare_engine_prework_for_block_with_future_state(
            2,
            2,
            2,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 43),
            Some(10),
            Some(replacement_transport),
        )
        .unwrap());

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 1);
    assert_eq!(snapshot.prework_cache_admissions, 2);
    assert_eq!(snapshot.prework_cache_queued_admissions, 1);
    assert_eq!(snapshot.prework_cache_invalidation_count, 1);
    assert_eq!(
        snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::SupersededByAdmission)
    );
    assert_eq!(
        snapshot.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::SupersededByAdmission)
    );
    assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
    assert_eq!(snapshot.last_prework_admission_block_sequence, Some(2));
    assert_eq!(snapshot.last_prework_admitted_from_block_sequence, Some(2));
}
