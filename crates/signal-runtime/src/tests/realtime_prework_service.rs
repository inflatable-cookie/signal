use super::*;

#[test]
fn runtime_degraded_bound_plugin_session_gates_prework_lane() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 6,
            prepare_budget_per_cycle: 2,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 32,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set plugin-bound forecast policy");
    runtime
        .apply_graph_projection(GraphProjection {
            graph_id: "graph:runtime:plugin-bound-gate".into(),
            node_count: 3,
            nodes: vec![
                GraphNodeProjection {
                    node_id: "inline".into(),
                    execution_class: GraphNodeExecutionClass::PureTransform,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::Gain { linear: 0.9 }],
                },
                GraphNodeProjection {
                    node_id: "plugin".into(),
                    execution_class: GraphNodeExecutionClass::PluginBacked,
                    latency_samples: 0,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.8 }],
                },
                GraphNodeProjection {
                    node_id: "latency".into(),
                    execution_class: GraphNodeExecutionClass::LatencyBearing,
                    latency_samples: 96,
                    stages: vec![GraphStageSpec::HardClip { threshold: 0.6 }],
                },
            ],
        })
        .unwrap();
    runtime
        .apply_plugin_backed_node_bindings(PluginBackedNodeBindingProjection {
            graph_id: "graph:runtime:plugin-bound-gate".into(),
            bindings: vec![PluginBackedNodeBinding {
                node_id: "plugin".into(),
                sandbox_id: "sandbox-a".into(),
            }],
        })
        .expect("apply plugin-backed bindings");
    runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-a",
            "region-a",
            TransportAttachIntent::SteadyState,
        )
        .expect("begin transport session");
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::Attached,
        Some(1),
        None,
    );
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated prework pressure");
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::DetachFault,
        Some(1),
        Some("late detach fault".into()),
    );

    runtime
        .service_prework_lane(1, 3)
        .expect("service elevated prework lane");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::PluginConstrained
    );
    assert_eq!(snapshot.prework_service_bound_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_active_bound_plugin_sandboxes, 0);
    assert_eq!(snapshot.prework_service_degraded_bound_plugin_sandboxes, 1);
    assert_eq!(snapshot.prework_service_missing_bound_plugin_sandboxes, 0);
    assert!(snapshot.prework_service_plugin_gate_active);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Yielding
    );
    assert!(snapshot.prework_service_yield_count >= 1);

    let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    let profiling = supervisor.profiling_receipt();
    let soak = supervisor.soak_receipt();
    assert!(profiling.plugin_gate_active);
    assert_eq!(profiling.degraded_bound_plugin_sandboxes, 1);
    assert_eq!(profiling.missing_bound_plugin_sandboxes, 0);
    assert_eq!(profiling.plugin_chain_stage_count, 1);
    assert!(profiling
        .render_json()
        .contains("\"plugin_gate_active\":true"));
    assert!(profiling
        .render_json()
        .contains("\"degraded_bound_plugin_sandboxes\":1"));
    assert_eq!(soak.plugin_fault_count, 0);
    assert_eq!(soak.plugin_quarantined_sandbox_count, 0);
}

#[test]
fn runtime_realtime_block_services_prework_window_under_normal_pressure() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 4,
            prepare_budget_per_cycle: 4,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set realtime-driven forecast policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-normal");
    runtime.start().expect("start runtime");

    let before = runtime.get_engine_block_snapshot();
    let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    apply_current_forecast_block_state(&mut runtime, 1);
    let first = runtime
        .process_engine_block(1, 1, first_block)
        .expect("process first realtime block");
    assert_eq!(
        first.snapshot.prework_cache_window_target_block_sequences,
        vec![2, 3, 4]
    );

    let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
    apply_current_forecast_block_state(&mut runtime, 2);
    let snapshot = runtime
        .process_engine_block(2, 2, second_block)
        .expect("process second realtime block")
        .snapshot;

    assert!(snapshot.prework_service_cycle_count > before.prework_service_cycle_count);
    assert_eq!(snapshot.last_prework_service_processing_epoch, Some(2));
    assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
    assert_eq!(snapshot.last_prework_service_cycle_count, 1);
    assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(4));
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(4)
    );
    assert!(snapshot.last_prework_service_prepared_targets >= 1);
    assert!(snapshot
        .last_prework_serviced_target_block_sequence
        .is_some_and(|block_sequence| block_sequence >= 5));
    assert_eq!(
        snapshot.last_prework_serviced_backlog_class,
        Some(RuntimePreworkBacklogClass::Deferred)
    );
    assert_eq!(snapshot.prework_pending_target_count, 0);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Idle
    );
    assert!(snapshot
        .prework_cache_window_target_block_sequences
        .contains(&5));
}

#[test]
fn runtime_compatible_schedule_projection_widens_normal_prework_service_scope() {
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
        .expect("set widened multicore realtime policy");
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:realtime-scheduler-widened-budget",
        &["track:drums", "track:bass"],
        false,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:widened-budget".into(),
            stream_count: 3,
        })
        .expect("apply widened compatible schedule");
    runtime.start().expect("start runtime");

    let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    apply_current_forecast_block_state(&mut runtime, 1);
    let first = runtime
        .process_engine_block(1, 1, first_block)
        .expect("process widened realtime block");

    assert_eq!(
        first.snapshot.scheduler_topology.schedule_stream_count,
        Some(3)
    );
    assert!(first.snapshot.scheduler_topology.compatible);
    assert_eq!(first.snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(first.snapshot.last_prework_service_effective_cycles, 3);
    assert_eq!(first.snapshot.last_prework_service_cycle_count, 3);
    assert_eq!(
        first.snapshot.last_prework_service_budget_per_cycle,
        Some(1)
    );
    assert_eq!(
        first
            .snapshot
            .last_prework_service_effective_budget_per_cycle,
        Some(3)
    );
    assert!(first.snapshot.last_prework_service_prepared_targets >= 7);
    assert!(first.snapshot.prework_service_prepared_targets >= 7);
    assert_eq!(first.snapshot.prework_pending_target_count, 0);

    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());
    assert_eq!(
        observation
            .engine_block_snapshot
            .scheduler_topology
            .schedule_stream_count,
        Some(3)
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .last_prework_service_requested_cycles,
        3
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .last_prework_service_effective_budget_per_cycle,
        Some(3)
    );
    assert!(observation
        .render_json()
        .contains("\"last_prework_service_requested_cycles\":3"));
    assert!(observation
        .render_json()
        .contains("\"last_prework_service_effective_budget_per_cycle\":3"));
}

#[test]
fn runtime_missing_schedule_projection_does_not_widen_prework_service_budget() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 4,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set single-budget realtime policy");
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:realtime-scheduler-no-schedule",
        &["track:drums", "track:bass"],
        false,
    );
    runtime.start().expect("start runtime");

    let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    apply_current_forecast_block_state(&mut runtime, 1);
    let first = runtime
        .process_engine_block(1, 1, first_block)
        .expect("process no-schedule realtime block");

    assert_eq!(
        first.snapshot.scheduler_topology.schedule_stream_count,
        None
    );
    assert!(!first.snapshot.scheduler_topology.compatible);
    assert_eq!(first.snapshot.last_prework_service_requested_cycles, 1);
    assert_eq!(
        first.snapshot.last_prework_service_budget_per_cycle,
        Some(1)
    );
    assert_eq!(
        first
            .snapshot
            .last_prework_service_effective_budget_per_cycle,
        Some(1)
    );
    assert!(first.snapshot.last_prework_service_prepared_targets <= 1);
}
