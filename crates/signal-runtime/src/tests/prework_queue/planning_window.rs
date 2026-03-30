use super::super::*;

#[test]
fn runtime_planning_window_retires_future_entries_not_in_revised_window() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:prework-window-revision".into(),
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

    let targets = vec![
        RuntimePreworkWindowTarget {
            target_block_sequence: 2,
            admitted_from_block_sequence: 1,
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 52),
            parameter_epoch_override: Some(9),
            transport_override: Some(TransportProjection {
                playing: true,
                timeline_position_samples: 96,
                tempo_bpm: 120.0,
                loop_state: None,
            }),
        },
        RuntimePreworkWindowTarget {
            target_block_sequence: 3,
            admitted_from_block_sequence: 1,
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
            parameter_epoch_override: Some(10),
            transport_override: Some(TransportProjection {
                playing: true,
                timeline_position_samples: 104,
                tempo_bpm: 121.0,
                loop_state: None,
            }),
        },
    ];
    assert_eq!(
        runtime
            .prepare_engine_prework_window(1, targets)
            .expect("initial planning window"),
        2
    );

    let revised_targets = vec![RuntimePreworkWindowTarget {
        target_block_sequence: 3,
        admitted_from_block_sequence: 2,
        buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 53),
        parameter_epoch_override: Some(10),
        transport_override: Some(TransportProjection {
            playing: true,
            timeline_position_samples: 104,
            tempo_bpm: 121.0,
            loop_state: None,
        }),
    }];
    assert_eq!(
        runtime
            .prepare_engine_prework_window(2, revised_targets)
            .expect("revised planning window"),
        1
    );

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 1);
    assert_eq!(snapshot.prework_cache_window_target_count, 1);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![3]
    );
    assert_eq!(snapshot.prework_cache_invalidation_count, 1);
    assert_eq!(
        snapshot.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::PlanningWindowRevised)
    );
    assert_eq!(
        snapshot.last_prework_retirement_reason,
        Some(RuntimePreworkRetirementReason::PlanningWindowRevised)
    );
    assert_eq!(snapshot.last_prework_retired_unconsumed, Some(true));
}

#[test]
fn runtime_planning_window_reuses_existing_future_sequences_and_allocates_missing() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:prework-window-sequences".into(),
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

    let current_sequence = runtime.allocate_block_sequence();
    let first_future_sequence = runtime.allocate_block_sequence();
    let second_future_sequence = runtime.allocate_block_sequence();

    let initial_targets = vec![
        RuntimePreworkWindowTarget {
            target_block_sequence: first_future_sequence,
            admitted_from_block_sequence: current_sequence,
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 61),
            parameter_epoch_override: Some(9),
            transport_override: Some(TransportProjection {
                playing: true,
                timeline_position_samples: 96,
                tempo_bpm: 120.0,
                loop_state: None,
            }),
        },
        RuntimePreworkWindowTarget {
            target_block_sequence: second_future_sequence,
            admitted_from_block_sequence: current_sequence,
            buffer: synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 62),
            parameter_epoch_override: Some(10),
            transport_override: Some(TransportProjection {
                playing: true,
                timeline_position_samples: 104,
                tempo_bpm: 121.0,
                loop_state: None,
            }),
        },
    ];
    runtime
        .prepare_engine_prework_window(1, initial_targets)
        .expect("initial planning window");

    let revised_sequences = runtime.plan_prework_window_block_sequences(first_future_sequence, 2);
    assert_eq!(
        revised_sequences,
        vec![
            second_future_sequence,
            second_future_sequence.saturating_add(1)
        ]
    );
    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(snapshot.prework_cache_queue_depth, 2);
    assert_eq!(snapshot.prework_cache_window_target_count, 2);
    assert_eq!(
        snapshot.prework_cache_window_target_block_sequences,
        vec![first_future_sequence, second_future_sequence]
    );
}
