use super::super::*;

#[test]
fn runtime_restart_and_reconfigure_keep_realtime_scheduler_window_coherent() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 3,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set restart forecast policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-restart");
    runtime.start().expect("start runtime");

    let first_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 1);
    apply_current_forecast_block_state(&mut runtime, 1);
    let first = runtime
        .process_engine_block(1, 1, first_block)
        .expect("process first realtime block");
    assert!(first
        .snapshot
        .prework_cache_window_target_block_sequences
        .contains(&4));

    runtime
        .restart(RestartRequest { reconfigure: None })
        .expect("restart runtime");
    let restarted = runtime.get_engine_block_snapshot();
    assert_eq!(
        restarted.prework_cache_window_target_block_sequences,
        vec![2, 3, 4]
    );

    let restart_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 2);
    apply_current_forecast_block_state(&mut runtime, 2);
    let after_restart = runtime
        .process_engine_block(2, 2, restart_block)
        .expect("process realtime block after restart");
    assert!(after_restart
        .snapshot
        .prework_cache_window_target_block_sequences
        .contains(&5));
    assert_eq!(
        after_restart.snapshot.last_prework_service_processing_epoch,
        Some(2)
    );

    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("reconfigure runtime");
    let reconfigured = runtime.get_engine_block_snapshot();
    assert_eq!(
        reconfigured.prework_cache_window_target_block_sequences,
        vec![3, 4, 5]
    );
    assert_eq!(
        reconfigured.prework_service_state,
        RuntimePreworkServiceState::Paused
    );

    let reconfigured_block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), 3);
    runtime.start().expect("restart after reconfigure");
    apply_current_forecast_block_state(&mut runtime, 3);
    let after_reconfigure = runtime
        .process_engine_block(3, 3, reconfigured_block)
        .expect("process realtime block after reconfigure");
    assert!(after_reconfigure
        .snapshot
        .prework_cache_window_target_block_sequences
        .contains(&6));
    assert_eq!(
        after_reconfigure
            .snapshot
            .last_prework_service_processing_epoch,
        Some(3)
    );

    let report = crate::interfaces::RuntimeObservationReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    assert_eq!(report.control_snapshot.restart_count, 1);
    assert!(report.scheduler_summary.prework_pending_target_count > 0);
    assert!(report.render_compact().contains("restarts=1"));

    let supervisor = crate::interfaces::RuntimeSupervisorReport::capture(
        &runtime,
        &RuntimeEventRecorder::default(),
    );
    assert!(supervisor.render_multiline().contains("restart_count=1"));
    assert!(supervisor
        .render_multiline()
        .contains("scheduler_summary_pending_targets="));
    let json = supervisor.render_json();
    assert!(json.contains("\"restart_count\":1"));
    assert!(json.contains("\"scheduler_summary\":{"));
}
