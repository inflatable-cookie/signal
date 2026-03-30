use super::*;

#[test]
fn scheduler_snapshot_tracks_state_and_phase_transitions() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);

    let configured = runtime.get_scheduler_snapshot();
    assert_eq!(configured.state, RuntimeSchedulerState::Configured);
    assert_eq!(configured.phase, RuntimeExecutionPhase::Idle);
    assert!(!configured.graph_applied);

    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:scheduler".into(),
            node_count: 2,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "track".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.85 }],
                },
                GraphNodeProjection {
                    node_id: "master".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 16,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.9 }],
                },
            ],
        })
        .unwrap();
    runtime.start().unwrap();

    let primed = runtime.get_scheduler_snapshot();
    assert_eq!(primed.state, RuntimeSchedulerState::Anticipative);
    assert_eq!(primed.phase, RuntimeExecutionPhase::Prework);
    assert!(primed.graph_applied);

    runtime
        .set_prework_forecast_profile(RuntimePreworkForecastProfileSelection {
            profile: RuntimePreworkForecastProfile::Local,
            target_window_blocks_override: Some(2),
        })
        .unwrap();
    seed_pending_prework_targets(&mut runtime, 1, &[2, 3]);

    runtime
        .apply_transport_projection(TransportProjection {
            playing: true,
            timeline_position_samples: 0,
            tempo_bpm: 120.0,
            loop_state: None,
        })
        .unwrap();
    runtime.service_prework_lane(1, 1).unwrap();

    let prework = runtime.get_scheduler_snapshot();
    assert_eq!(prework.state, RuntimeSchedulerState::Anticipative);
    assert_eq!(prework.phase, RuntimeExecutionPhase::Prework);
    assert!(prework.transport_projected);

    runtime
        .process_engine_block(
            2,
            1,
            AudioBuffer::new(SampleRate(48_000), ChannelLayout::Stereo, FrameCount(256)),
        )
        .unwrap();

    let realtime = runtime.get_scheduler_snapshot();
    assert_eq!(realtime.state, RuntimeSchedulerState::Anticipative);
    assert_eq!(realtime.phase, RuntimeExecutionPhase::Realtime);
    assert_eq!(realtime.processed_block_count, 1);
}

#[test]
fn scheduler_snapshot_surfaces_realtime_only_and_degraded_runtime_states() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_disabled_forecast(&mut runtime, false);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:realtime-only".into(),
            node_count: 1,
            nodes: vec![GraphNodeProjection {
                node_id: "track".into(),
                execution_class: GraphNodeExecutionClass::LatencyBearing,
                latency_samples: 32,
                stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
            }],
        })
        .unwrap();
    runtime.start().unwrap();

    let realtime_only = runtime.get_scheduler_snapshot();
    assert_eq!(realtime_only.state, RuntimeSchedulerState::RealtimeOnly);
    assert_eq!(realtime_only.phase, RuntimeExecutionPhase::Priming);

    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .unwrap();

    let degraded = runtime.get_scheduler_snapshot();
    assert_eq!(degraded.state, RuntimeSchedulerState::Degraded);
    assert_eq!(degraded.phase, RuntimeExecutionPhase::Degraded);
}

#[test]
fn runtime_replans_graph_when_anticipative_mode_changes() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure_with_disabled_forecast(&mut runtime, true);
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:planning".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "input".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "drive".into(),
                    execution_class: GraphNodeExecutionClass::Stateful,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::TanhDrive { drive: 1.4 }],
                },
                GraphNodeProjection {
                    node_id: "output".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 32,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.75 }],
                },
            ],
        })
        .unwrap();

    let initial = runtime.get_engine_block_snapshot();
    assert!(initial.anticipative_planning_enabled);
    assert_eq!(initial.inline_realtime_node_count, 1);
    assert_eq!(initial.stateful_realtime_node_count, 1);
    assert_eq!(initial.anticipative_eligible_node_count, 1);
    assert_eq!(initial.prepared_dispatch_count, 1);
    assert_eq!(initial.realtime_dispatch_count, 1);
    assert_eq!(initial.dispatch_handoff_count, 1);
    assert!(initial.prework_cache_enabled);
    assert_eq!(initial.prework_cache_state, RuntimePreworkCacheState::Empty);
    assert_eq!(
        initial.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Empty
    );
    assert_eq!(initial.prework_cache_admissions, 0);
    assert_eq!(initial.prework_cache_consumptions, 0);
    assert_eq!(initial.prework_cache_hits, 0);
    assert_eq!(initial.prework_cache_misses, 0);
    assert_eq!(initial.prework_cache_invalidation_count, 0);
    assert_eq!(initial.prework_cache_retirement_count, 0);

    let mut request = RuntimeConfigRequest::new(48_000, 256);
    request.anticipative_enabled = false;
    runtime.configure(request).unwrap();

    let replanned = runtime.get_engine_block_snapshot();
    assert!(!replanned.anticipative_planning_enabled);
    assert_eq!(replanned.inline_realtime_node_count, 1);
    assert_eq!(replanned.stateful_realtime_node_count, 2);
    assert_eq!(replanned.anticipative_eligible_node_count, 0);
    assert_eq!(replanned.phase_count, 2);
    assert_eq!(replanned.anticipative_phase_count, 0);
    assert_eq!(replanned.lane_count, 1);
    assert_eq!(replanned.anticipative_lane_count, 0);
    assert_eq!(
        replanned.lane_order,
        vec![signal_graph::GraphExecutionLane::Realtime]
    );
    assert_eq!(replanned.dispatch_count, 1);
    assert_eq!(replanned.dispatch_boundary_count, 0);
    assert_eq!(replanned.prepared_dispatch_count, 0);
    assert_eq!(replanned.realtime_dispatch_count, 1);
    assert_eq!(replanned.dispatch_handoff_count, 0);
    assert!(!replanned.prework_cache_enabled);
    assert_eq!(
        replanned.prework_cache_state,
        RuntimePreworkCacheState::Disabled
    );
    assert_eq!(
        replanned.prework_cache_freshness_state,
        RuntimePreworkFreshnessState::Disabled
    );
    assert_eq!(replanned.prework_cache_admissions, 0);
    assert_eq!(replanned.prework_cache_consumptions, 0);
    assert_eq!(replanned.prework_cache_valid_until_processing_epoch, None);
    assert_eq!(
        replanned.last_prework_invalidation_reason,
        Some(RuntimePreworkInvalidationReason::RuntimeReconfigured)
    );
    assert_eq!(replanned.prework_cache_invalidation_count, 0);
    assert_eq!(replanned.prework_cache_retirement_count, 0);
    assert_eq!(
        replanned.dispatch_order,
        vec![signal_graph::GraphExecutionLane::Realtime]
    );
    assert_eq!(
        replanned.phase_order,
        vec![
            signal_graph::GraphNodePlanningGroup::InlineRealtime,
            signal_graph::GraphNodePlanningGroup::StatefulRealtime,
        ]
    );
    assert_eq!(replanned.planned_nodes.len(), 3);
    assert_eq!(
        replanned
            .planned_nodes
            .iter()
            .map(|node| (node.node_id.as_str(), format!("{:?}", node.group)))
            .collect::<Vec<_>>(),
        vec![
            ("input", "InlineRealtime".into()),
            ("drive", "StatefulRealtime".into()),
            ("output", "StatefulRealtime".into()),
        ]
    );
}

#[test]
fn safe_mode_sets_degraded_readiness() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    handshake_and_configure(&mut runtime);
    runtime.start().unwrap();
    runtime
        .set_safe_mode(SafeModeRequest { enabled: true })
        .unwrap();

    assert!(matches!(
        runtime.get_readiness(),
        RuntimeReadiness::Degraded { .. }
    ));
}

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
