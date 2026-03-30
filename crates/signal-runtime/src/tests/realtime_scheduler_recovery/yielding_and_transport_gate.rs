use super::super::*;

#[test]
fn runtime_lingering_transport_enters_yielding_scheduler_state_under_elevated_pressure() {
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
        .expect("set lingering realtime policy");
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:realtime-scheduler-lingering");
    runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-a",
            "region-a",
            TransportAttachIntent::SteadyState,
        )
        .expect("begin steady session");
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::DetachRequested,
        Some(1),
        None,
    );
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated pressure");
    seed_pending_prework_targets(&mut runtime, 1, &[2, 3, 4]);
    runtime.refresh_prework_service_policy_and_state(None);
    let snapshot = runtime.get_engine_block_snapshot();

    assert_eq!(
        snapshot.prework_service_pressure,
        RuntimePreworkServicePressure::Elevated
    );
    assert_eq!(snapshot.prework_service_recovery_overlap_sessions, 0);
    assert_eq!(snapshot.prework_service_lingering_sessions, 1);
    assert_eq!(snapshot.prework_service_detach_faulted_sessions, 0);
    assert!(snapshot.prework_service_transport_gate_active);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Yielding
    );
    assert!(snapshot.prework_pending_target_count > 0);
}

#[test]
fn runtime_schedule_widened_transport_gate_yields_without_servicing() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 256));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    let policy = RuntimePreworkForecastPolicy {
        target_window_blocks: 6,
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
        .expect("set widened transport policy");
    apply_latency_runtime_graph(
        &mut runtime,
        "graph:runtime:transport-gate-schedule-widened",
    );
    runtime
        .begin_transport_session(
            "sandbox-a",
            "lease-a",
            "region-a",
            TransportAttachIntent::SteadyState,
        )
        .expect("begin steady session");
    runtime.record_plugin_sandbox_transport(
        "sandbox-a",
        "lease-a",
        "region-a",
        PluginSandboxTransportStage::DetachRequested,
        Some(1),
        None,
    );
    runtime.start().expect("start runtime");
    runtime
        .set_prework_service_pressure(RuntimePreworkServicePressure::Elevated)
        .expect("set elevated pressure");
    runtime
        .apply_schedule_projection(ScheduleProjection {
            schedule_id: "sched:runtime:transport-gate-widened".into(),
            stream_count: 3,
        })
        .expect("apply widened schedule projection");
    let current_sequence = runtime.allocate_block_sequence();

    let admitted = runtime
        .prime_engine_prework_window_with_forecast(1, current_sequence, &policy)
        .expect("prime widened transport-gated window");

    let snapshot = runtime.get_engine_block_snapshot();
    assert_eq!(admitted, 0);
    assert_eq!(snapshot.last_prework_service_requested_cycles, 3);
    assert_eq!(snapshot.last_prework_service_effective_cycles, 0);
    assert_eq!(
        snapshot.last_prework_service_effective_budget_per_cycle,
        Some(0)
    );
    assert!(snapshot.prework_service_transport_gate_active);
    assert_eq!(
        snapshot.prework_service_state,
        RuntimePreworkServiceState::Yielding
    );
    assert!(snapshot.prework_pending_target_count > 0);
    assert!(snapshot.prework_service_yield_count >= 1);
}
