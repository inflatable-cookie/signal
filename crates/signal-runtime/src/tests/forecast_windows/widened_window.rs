use super::super::*;

#[test]
fn runtime_constrained_anticipative_window_caps_widened_service_realization() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    runtime
        .set_prework_forecast_policy(RuntimePreworkForecastPolicy {
            target_window_blocks: 1,
            prepare_budget_per_cycle: 1,
            buffer_seed_offset: 0,
            transport_playing: true,
            transport_tempo_bpm: 126.0,
            transport_loop_length_blocks: 16,
            parameter_target: "engine.local.drive".into(),
            parameter_cycle_length: 8,
        })
        .expect("set constrained widened forecast policy");
    install_scheduler_topology_runtime_graph(
        &mut runtime,
        "graph:runtime:constrained-window-widened",
        &["track:drums", "track:bass"],
        false,
    );
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:constrained-window-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened constrained schedule");
    runtime.start().expect("start runtime");

    for block_sequence in 1..=3u64 {
        let block = synthetic_stereo_block(SampleRate(48_000), FrameCount(256), block_sequence);
        apply_current_forecast_block_state(&mut runtime, block_sequence);
        let snapshot = runtime
            .process_engine_block(block_sequence, block_sequence, block)
            .expect("process constrained widened block")
            .snapshot;

        assert_eq!(snapshot.scheduler_topology.schedule_stream_count, Some(3));
        assert!(snapshot.scheduler_topology.compatible);
        assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
        assert_eq!(snapshot.last_prework_service_effective_cycles, 3);
        assert_eq!(snapshot.last_prework_service_cycle_count, 1);
        assert_eq!(snapshot.last_prework_service_prepared_targets, 1);
        assert!(snapshot.prework_cache_window_target_count <= 2);
        assert_eq!(snapshot.prework_pending_target_count, 0);
        assert!(snapshot.prework_cache_peak_queue_depth <= 2);
    }
}
