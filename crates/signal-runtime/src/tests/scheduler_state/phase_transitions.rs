use super::super::*;

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
