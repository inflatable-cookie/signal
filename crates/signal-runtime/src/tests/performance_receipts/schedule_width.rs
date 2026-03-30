use super::super::*;

#[test]
fn runtime_schedule_width_survives_restart_and_reconfigure_transitions() {
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
        .expect("set widened restart policy");
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:restart-reconfigure-schedule-widened",
        &["track:drums", "track:bass"],
        false,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:restart-reconfigure-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");
    runtime.start().expect("start runtime");

    let started = runtime.get_engine_block_snapshot();
    assert_eq!(started.scheduler_topology.schedule_stream_count, Some(3));
    assert!(started.scheduler_topology.compatible);
    assert_eq!(started.last_prework_service_requested_cycles, 3);
    assert_eq!(started.last_prework_service_effective_cycles, 3);

    runtime
        .restart(RestartRequest { reconfigure: None })
        .expect("restart runtime");
    let restarted = runtime.get_engine_block_snapshot();
    assert_eq!(restarted.scheduler_topology.schedule_stream_count, Some(3));
    assert!(restarted.scheduler_topology.compatible);
    assert_eq!(restarted.last_prework_service_requested_cycles, 3);
    assert_eq!(restarted.last_prework_service_effective_cycles, 3);
    assert_eq!(runtime.get_control_snapshot().restart_count, 1);

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("reconfigure runtime");
    let reconfigured = runtime.get_engine_block_snapshot();
    assert_eq!(
        reconfigured.scheduler_topology.schedule_stream_count,
        Some(3)
    );
    assert!(reconfigured.scheduler_topology.compatible);
    assert_eq!(
        reconfigured.prework_service_state,
        RuntimePreworkServiceState::Paused
    );

    runtime.start().expect("restart after reconfigure");
    let restarted_after_reconfigure = runtime.get_engine_block_snapshot();
    assert_eq!(
        restarted_after_reconfigure
            .scheduler_topology
            .schedule_stream_count,
        Some(3)
    );
    assert!(restarted_after_reconfigure.scheduler_topology.compatible);
    assert_eq!(
        restarted_after_reconfigure.last_prework_service_requested_cycles,
        3
    );
    assert_eq!(
        restarted_after_reconfigure.last_prework_service_effective_cycles,
        3
    );
    assert_eq!(
        restarted_after_reconfigure.last_prework_service_effective_budget_per_cycle,
        Some(3)
    );
}
