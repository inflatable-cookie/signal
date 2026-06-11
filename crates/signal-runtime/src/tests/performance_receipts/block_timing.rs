use super::super::*;

#[test]
fn runtime_process_engine_block_records_bounded_timing_and_budget_fields() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:block-timing");

    let result = runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 21),
        )
        .expect("process runtime block with timing instrumentation");
    let snapshot = runtime.get_engine_block_snapshot();
    let diagnostics = runtime.get_diagnostics_snapshot();

    assert_eq!(snapshot.last_block_sequence, Some(1));
    assert_eq!(snapshot.last_block_deadline_budget_ns, Some(1_000_000));
    assert_eq!(
        result.snapshot.last_block_deadline_budget_ns,
        Some(1_000_000)
    );
    assert_eq!(
        snapshot.last_block_execution_time_ns,
        result.snapshot.last_block_execution_time_ns
    );
    let execution_time_ns = snapshot
        .last_block_execution_time_ns
        .expect("runtime should capture a block execution time");
    assert!(execution_time_ns > 0);
    assert_eq!(
        snapshot.last_block_budget_overrun_ns.is_some(),
        snapshot.last_block_deadline_pressure == RuntimeBlockDeadlinePressure::Overrun
    );
    assert!(snapshot.peak_block_execution_time_ns >= execution_time_ns);
    assert!((diagnostics.graph_latency_ms - (execution_time_ns as f32 / 1_000_000.0)).abs() < 0.01);
    assert_eq!(
        diagnostics.cpu_load_percent,
        snapshot
            .last_block_budget_utilization_percent
            .expect("timing instrumentation should derive utilization")
    );
}

#[test]
fn runtime_block_timing_pressure_rolls_into_performance_snapshot_and_trace_receipt() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    handshake_and_configure_with_anticipative(&mut runtime, true);
    apply_latency_runtime_graph(&mut runtime, "graph:runtime:block-timing-trace");

    runtime.record_block_execution_timing_ns(48, 500_000);
    let normal = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

    runtime.record_block_execution_timing_ns(48, 800_000);
    let elevated = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

    runtime.record_block_execution_timing_ns(48, 950_000);
    let critical = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

    runtime.record_block_execution_timing_ns(48, 1_250_000);
    let overrun = RuntimeSupervisorReport::capture(&runtime, &RuntimeEventRecorder::default());

    let performance = overrun.performance_snapshot();
    assert_eq!(performance.last_block_deadline_budget_ns, Some(1_000_000));
    assert_eq!(performance.last_block_execution_time_ns, Some(1_250_000));
    assert_eq!(
        performance.last_block_deadline_pressure,
        RuntimeBlockDeadlinePressure::Overrun
    );
    assert_eq!(performance.last_block_budget_overrun_ns, Some(250_000));
    assert_eq!(performance.budget_overrun_count, 1);
    assert_eq!(performance.peak_block_execution_time_ns, 1_250_000);
    assert_eq!(performance.peak_block_budget_overrun_ns, 250_000);

    let trace = RuntimeSupervisorReport::build_performance_trace_receipt(&[
        normal.clone(),
        elevated,
        critical,
        overrun.clone(),
    ]);
    assert_eq!(trace.elevated_deadline_pressure_observation_count, 1);
    assert_eq!(trace.critical_deadline_pressure_observation_count, 1);
    assert_eq!(trace.overrun_deadline_pressure_observation_count, 1);
    assert_eq!(trace.budget_overrun_count_delta, 1);
    assert_eq!(trace.peak_block_execution_time_ns, 1_250_000);
    assert_eq!(trace.peak_block_budget_overrun_ns, 250_000);
}
