use super::super::*;

#[test]
fn runtime_elevated_pressure_clamps_schedule_widened_prework_cycles() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    let policy = RuntimePreworkForecastPolicy {
        target_window_blocks: 8,
        prepare_budget_per_cycle: 1,
        buffer_seed_offset: 0,
        transport_playing: true,
        transport_tempo_bpm: 126.0,
        transport_loop_length_blocks: 16,
        parameter_target: "engine.local.drive".into(),
        parameter_cycle_length: 8,
    };
    runtime
        .set_prework_forecast_policy(policy.clone())
        .expect("set elevated widened realtime policy");
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:realtime-scheduler-elevated-widened-cycles",
        &["track:drums", "track:bass"],
        false,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:elevated-widened-cycles".into(),
            stream_count: 3,
        })
        .expect("apply elevated compatible schedule");
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated pressure");
    let current_sequence = runtime.allocate_block_sequence();
    let admitted = runtime
        .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
        .expect("prime elevated widened forecast window");
    assert_eq!(admitted, 1);

    let snapshot = runtime.get_engine_block_snapshot();

    assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
    assert!(snapshot.scheduler_topology.compatible);
    assert_eq!(
        snapshot.prework_service_pressure,
        RuntimePreworkServicePressure::Elevated
    );
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
    assert_eq!(snapshot.last_prework_service_cycle_count, 1);
    assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(1));
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(1)
    );
    assert!(snapshot.last_prework_service_prepared_targets <= 1);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Pending
    );
    assert!(snapshot.prework_service_throttle_count >= 1);
    assert!(snapshot.prework_pending_target_count > 0);
}

#[test]
fn runtime_realtime_block_respects_elevated_pressure_backlog_limits() {
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
        .expect("set elevated realtime-driven forecast policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-elevated");
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated pressure");

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

    assert_eq!(
        snapshot.prework_service_pressure,
        RuntimePreworkServicePressure::Elevated
    );
    assert_eq!(
        snapshot.prework_service_semantic_policy,
        RuntimePreworkServiceSemanticPolicy::Balanced
    );
    assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Pending
    );
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_pending_deferred_target_count > 0);
    assert_eq!(snapshot.prework_pending_immediate_target_count, 0);
    assert!(snapshot
        .prework_next_pending_target_block_sequence
        .is_some());
    assert!(snapshot.prework_service_throttle_count >= 1);
}

#[test]
fn runtime_recovery_overlap_throttles_realtime_scheduler_under_normal_pressure() {
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
        .expect("set recovery-overlap realtime policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-overlap");
    runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-a",
            "region-a",
            TransportAttachIntent::SteadyState,
        )
        .expect("begin steady session");
    runtime
        .begin_transport_session(
            "sandbox-b",
            "lease-b",
            "region-b",
            TransportAttachIntent::RecoveryOverlap,
        )
        .expect("begin overlap session");
    runtime.start().expect("start runtime");

    let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    apply_current_forecast_block_state(&mut runtime, 1);
    runtime
        .process_engine_block(1, 1, first_block)
        .expect("process first realtime block");

    let second_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
    apply_current_forecast_block_state(&mut runtime, 2);
    let snapshot = runtime
        .process_engine_block(2, 2, second_block)
        .expect("process second realtime block")
        .snapshot;

    assert_eq!(
        snapshot.prework_service_pressure,
        RuntimePreworkServicePressure::Normal
    );
    assert_eq!(snapshot.prework_service_recovery_overlap_sessions, 1);
    assert_eq!(snapshot.prework_service_lingering_sessions, 0);
    assert_eq!(snapshot.prework_service_detach_faulted_sessions, 0);
    assert!(!snapshot.prework_service_transport_gate_active);
    assert_eq!(snapshot.last_prework_service_requested_cycles, 1);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 1);
    assert_eq!(snapshot.last_prework_service_budget_per_cycle, Some(4));
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(1)
    );
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Pending
    );
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_pending_deferred_target_count > 0);
    assert!(snapshot.prework_service_throttle_count >= 1);

    let report = crate::interfaces::RuntimeObservationReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    assert_eq!(report.degradation_summary.recovery_overlap_sessions, 1);
    assert_eq!(report.degradation_summary.lingering_sessions, 0);
    assert!(!report.degradation_summary.transport_gate_active);
    assert_eq!(
        report.scheduler_summary.prework_pending_target_count,
        snapshot.prework_pending_target_count
    );
    assert!(report
        .render_compact()
        .contains("degradation_summary_sessions=1/0/0/0"));

    let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    assert!(supervisor
        .render_multiline()
        .contains("degradation_summary_sessions=1/0/0/0/0"));
    let json = supervisor.render_json();
    assert!(json.contains("\"degradation_summary\":{"));
    assert!(json.contains("\"recovery_overlap_sessions\":1"));
    assert!(json.contains("\"lingering_sessions\":0"));
}
