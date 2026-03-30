use super::super::*;

#[test]
fn runtime_executes_applied_graph_block_and_updates_snapshot() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:test".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "input".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![
                        GraphStageSpec::Gain { linear: 0.5 },
                        GraphStageSpec::Bias { amount: 0.2 },
                    ],
                },
                GraphNodeProjection {
                    node_id: "output".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.7 }],
                },
            ],
        })
        .unwrap();
    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 96,
            tempo_bpm: 120.0,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 64,
                end_samples: 128,
            }),
        })
        .unwrap();
    runtime
        .apply_parameter_batch(ParameterBatch {
            epoch: runtime.projection_epoch(),
            events: vec![ParameterEvent {
                target: "engine.runtime.test".into(),
                sample_offset: 0,
                normalized_value: 0.5,
            }],
        })
        .unwrap();

    let result = runtime
        .process_engine_block(
            1,
            42,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(8), 3),
        )
        .unwrap();

    assert_eq!(
        result.snapshot.graph_id.as_deref(),
        Some("graph:runtime:test")
    );
    assert_eq!(result.snapshot.node_count, 2);
    assert_eq!(result.snapshot.stateful_node_count, 1);
    assert_eq!(result.snapshot.latency_node_count, 1);
    assert!(result.snapshot.anticipative_planning_enabled);
    assert_eq!(result.snapshot.inline_realtime_node_count, 1);
    assert_eq!(result.snapshot.stateful_realtime_node_count, 0);
    assert_eq!(result.snapshot.anticipative_eligible_node_count, 1);
    assert_eq!(result.snapshot.phase_count, 2);
    assert_eq!(result.snapshot.anticipative_phase_count, 1);
    assert_eq!(result.snapshot.lane_count, 2);
    assert_eq!(result.snapshot.anticipative_lane_count, 1);
    assert_eq!(
        result.snapshot.lane_order,
        vec![
            signal_graph::GraphExecutionLane::Anticipative,
            signal_graph::GraphExecutionLane::Realtime,
        ]
    );
    assert_eq!(result.snapshot.dispatch_count, 2);
    assert_eq!(result.snapshot.dispatch_boundary_count, 1);
    assert_eq!(
        result.snapshot.dispatch_order,
        vec![
            signal_graph::GraphExecutionLane::Anticipative,
            signal_graph::GraphExecutionLane::Realtime,
        ]
    );
    assert_eq!(result.snapshot.prepared_dispatch_count, 1);
    assert_eq!(result.snapshot.realtime_dispatch_count, 1);
    assert_eq!(result.snapshot.dispatch_handoff_count, 1);
    assert!(result.snapshot.prework_cache_enabled);
    assert_eq!(
        result.snapshot.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(result.snapshot.prework_cache_admissions, 1);
    assert_eq!(result.snapshot.prework_cache_consumptions, 1);
    assert_eq!(result.snapshot.prework_cache_hits, 0);
    assert_eq!(result.snapshot.prework_cache_misses, 1);
    assert_eq!(result.snapshot.prework_cache_invalidation_count, 0);
    assert_eq!(result.snapshot.prework_cache_retirement_count, 0);
    assert_eq!(
        result.snapshot.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Fresh
    );
    assert_eq!(result.snapshot.prework_cache_block_freshness_window, 2);
    assert_eq!(
        result.snapshot.prework_cache_remaining_valid_blocks,
        Some(2)
    );
    assert!(!result.snapshot.last_prework_cache_hit);
    assert_eq!(result.snapshot.last_prework_invalidation_reason, None);
    assert_eq!(
        result.snapshot.prework_cache_valid_until_processing_epoch,
        Some(2)
    );
    assert_eq!(
        result.snapshot.prework_cache_valid_until_block_sequence,
        Some(44)
    );
    assert_eq!(
        result.snapshot.last_prework_source_processing_epoch,
        Some(1)
    );
    assert_eq!(result.snapshot.last_prework_source_block_sequence, Some(42));
    assert_eq!(
        result.snapshot.last_prework_admission_processing_epoch,
        Some(1)
    );
    assert_eq!(
        result.snapshot.last_prework_admission_block_sequence,
        Some(42)
    );
    assert_eq!(
        result.snapshot.last_prework_consumption_processing_epoch,
        Some(1)
    );
    assert_eq!(
        result.snapshot.last_prework_consumption_block_sequence,
        Some(42)
    );
    assert_eq!(
        result.snapshot.phase_order,
        vec![
            signal_graph::GraphNodePlanningGroup::InlineRealtime,
            signal_graph::GraphNodePlanningGroup::AnticipativeEligible,
        ]
    );
    assert_eq!(result.snapshot.planned_nodes.len(), 2);
    assert_eq!(result.snapshot.stage_count, 3);
    assert_eq!(result.snapshot.total_latency_samples, 16);
    assert_eq!(result.snapshot.max_node_latency_samples, 16);
    assert_eq!(result.snapshot.processed_blocks, 1);
    assert_eq!(result.snapshot.last_processing_epoch, Some(1));
    assert_eq!(result.snapshot.last_block_sequence, Some(42));
    assert_eq!(result.snapshot.last_frame_count, 8);
    assert_eq!(result.snapshot.last_channel_count, 2);
    assert!(result.snapshot.last_prework_output_peak.is_some());
    assert_eq!(
        result.snapshot.last_prework_output_peak,
        result.snapshot.last_realtime_input_peak
    );
    assert!(result.snapshot.last_output_peak.unwrap_or_default() <= 0.7);
    assert!(result.snapshot.last_output_rms.unwrap_or_default() > 0.0);
    assert_eq!(
        result
            .snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.projection_epoch),
        Some(1)
    );
    assert_eq!(
        result
            .snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.parameter_epoch),
        Some(1)
    );
    assert_eq!(
        result
            .snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.anticipative_enabled),
        Some(true)
    );
    assert_eq!(
        result
            .snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.transport_playing),
        Some(true)
    );
    assert_eq!(
        result
            .snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.timeline_position_samples),
        Some(96)
    );
    assert!(!result.output.samples().is_empty());
    assert_eq!(
        runtime
            .applied_transport
            .map(|transport| transport.timeline_position_samples),
        Some(104)
    );

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        observation.engine_block_snapshot.graph_id.as_deref(),
        Some("graph:runtime:test")
    );
    assert_eq!(observation.engine_block_snapshot.node_count, 2);
    assert_eq!(observation.engine_block_snapshot.stateful_node_count, 1);
    assert!(
        observation
            .engine_block_snapshot
            .anticipative_planning_enabled
    );
    assert_eq!(
        observation.engine_block_snapshot.inline_realtime_node_count,
        1
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .stateful_realtime_node_count,
        0
    );
    assert_eq!(observation.engine_block_snapshot.phase_count, 2);
    assert_eq!(
        observation.engine_block_snapshot.anticipative_phase_count,
        1
    );
    assert_eq!(observation.engine_block_snapshot.lane_count, 2);
    assert_eq!(observation.engine_block_snapshot.anticipative_lane_count, 1);
    assert_eq!(observation.engine_block_snapshot.dispatch_count, 2);
    assert_eq!(observation.engine_block_snapshot.dispatch_boundary_count, 1);
    assert_eq!(observation.engine_block_snapshot.prepared_dispatch_count, 1);
    assert_eq!(observation.engine_block_snapshot.realtime_dispatch_count, 1);
    assert_eq!(observation.engine_block_snapshot.dispatch_handoff_count, 1);
    assert_eq!(observation.scheduler_summary.phase_count, 2);
    assert_eq!(observation.scheduler_summary.lane_count, 2);
    assert_eq!(observation.scheduler_summary.dispatch_count, 2);
    assert_eq!(
        observation.scheduler_snapshot.state,
        RuntimeSchedulerState::Configured
    );
    assert_eq!(
        observation.scheduler_snapshot.phase,
        RuntimeExecutionPhase::Idle
    );
    assert!(observation.scheduler_snapshot.graph_applied);
    assert!(!observation.scheduler_snapshot.schedule_applied);
    assert!(observation.scheduler_snapshot.transport_projected);
    assert_eq!(
        observation.scheduler_summary.prework_service_state,
        RuntimePreworkServiceState::Disabled
    );
    assert_eq!(observation.block_summary.processed_blocks, 1);
    assert_eq!(observation.block_summary.transport_epoch, 1);
    assert_eq!(
        observation.block_summary.transport_transition,
        Some(crate::interfaces::RuntimeTransportTransitionKind::Started)
    );
    assert!(!observation.degradation_summary.readiness_degraded);
    assert_eq!(observation.degradation_summary.xrun_count, 0);
    assert!(observation.engine_block_snapshot.prework_cache_enabled);
    assert_eq!(
        observation.engine_block_snapshot.prework_cache_state,
        RuntimePreworkCacheState::Consumed
    );
    assert_eq!(
        observation.engine_block_snapshot.prework_cache_admissions,
        1
    );
    assert_eq!(
        observation.engine_block_snapshot.prework_cache_consumptions,
        1
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Fresh
    );
    assert_eq!(observation.engine_block_snapshot.prework_cache_hits, 0);
    assert_eq!(observation.engine_block_snapshot.prework_cache_misses, 1);
    assert_eq!(
        observation
            .engine_block_snapshot
            .prework_cache_retirement_count,
        0
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .prework_cache_invalidation_count,
        0
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .prework_cache_valid_until_processing_epoch,
        Some(2)
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .prework_cache_valid_until_block_sequence,
        Some(44)
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .anticipative_eligible_node_count,
        1
    );
    assert_eq!(observation.engine_block_snapshot.processed_blocks, 1);
    assert_eq!(
        observation
            .engine_block_snapshot
            .last_execution_context
            .as_ref()
            .map(|context| context.transport_tempo_bpm),
        Some(120.0)
    );
}
