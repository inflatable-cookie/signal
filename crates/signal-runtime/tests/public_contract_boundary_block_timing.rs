#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;

use public_contract_boundary_graph_foundation_support::apply_public_capture_graph;
use signal_graph::synthetic_stereo_block;
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    HandshakeRequest, RuntimeBlockDeadlinePressure, RuntimeConfig, RuntimeConfigRequest,
    RuntimeEventRecorder, RuntimeLifecycleApi, RuntimeObservationReport, RuntimeSupervisorReport,
    SignalRuntime,
};

#[test]
fn public_runtime_block_timing_boundary_reports_bounded_runtime_measurements() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-block-timing".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public block timing handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public block timing configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:public:block-timing");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 47),
        )
        .expect("public block timing block should process");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let performance = observation.performance_snapshot();
    let trace = RuntimeObservationReport::build_performance_trace_receipt(&[observation.clone()]);

    assert_eq!(
        observation.engine_block_snapshot.last_block_sequence,
        Some(1)
    );
    assert_eq!(
        observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns,
        Some(1_000_000)
    );
    assert!(
        observation
            .engine_block_snapshot
            .last_block_execution_time_ns
            .expect("public block timing should expose latest execution time")
            > 0
    );
    assert_eq!(
        performance.last_block_execution_time_ns,
        observation
            .engine_block_snapshot
            .last_block_execution_time_ns
    );
    assert_eq!(
        performance.last_block_deadline_budget_ns,
        observation
            .engine_block_snapshot
            .last_block_deadline_budget_ns
    );
    assert_eq!(
        performance.last_block_deadline_pressure,
        observation
            .engine_block_snapshot
            .last_block_deadline_pressure
    );
    assert!(matches!(
        performance.last_block_deadline_pressure,
        RuntimeBlockDeadlinePressure::Normal
            | RuntimeBlockDeadlinePressure::Elevated
            | RuntimeBlockDeadlinePressure::Critical
            | RuntimeBlockDeadlinePressure::Overrun
    ));
    assert_eq!(
        supervisor
            .performance_snapshot()
            .last_block_execution_time_ns,
        performance.last_block_execution_time_ns
    );
    assert_eq!(trace.observation_count, 1);
    assert_eq!(
        trace.peak_block_execution_time_ns,
        performance
            .last_block_execution_time_ns
            .expect("trace should preserve the public latest block timing")
    );
    assert_eq!(
        trace.peak_block_budget_utilization_percent,
        performance
            .last_block_budget_utilization_percent
            .expect("trace should preserve public budget utilization")
    );

    let observation_json = observation.render_json();
    assert!(observation_json.contains("\"engine_block_snapshot\":{"));
    assert!(observation_json.contains("\"last_block_execution_time_ns\":"));
    assert!(observation_json.contains("\"last_block_deadline_pressure\":"));

    let supervisor_json = supervisor.render_json();
    assert!(supervisor_json.contains("\"engine_block_snapshot\":{"));
    assert!(supervisor_json.contains("\"last_block_deadline_budget_ns\":1000000"));

    let performance_json = performance.render_json();
    assert!(performance_json.contains("\"last_block_execution_time_ns\":"));
    assert!(performance_json.contains("\"last_block_deadline_pressure\":"));

    let trace_json = trace.render_json();
    assert!(trace_json.contains("\"peak_block_execution_time_ns\":"));
    assert!(trace_json.contains("\"peak_block_budget_utilization_percent\":"));
}
