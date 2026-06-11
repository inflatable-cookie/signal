#[path = "support/public_contract_boundary_graph_foundation.rs"]
mod public_contract_boundary_graph_foundation_support;

use public_contract_boundary_graph_foundation_support::apply_public_capture_graph;
use signal_graph::{synthetic_stereo_block, GraphExecutionLane};
use signal_primitives::{FrameCount, SampleRate};
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts() {
    let recorder = RuntimeEventRecorder::default();
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 48));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-critical-path".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public critical-path handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 48))
        .expect("public critical-path configure should succeed");
    apply_public_capture_graph(&mut runtime, "graph:public:critical-path");
    runtime
        .process_engine_block(
            1,
            1,
            synthetic_stereo_block(SampleRate(48_000), FrameCount(48), 31),
        )
        .expect("public critical-path block should process");

    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    let performance = observation.performance_snapshot();
    let trace = RuntimeObservationReport::build_performance_trace_receipt(std::slice::from_ref(
        &observation,
    ));

    assert!(performance.hot_latency_node_id.is_some());
    assert!(performance.hot_latency_group_node_count > 0);
    assert!(matches!(
        performance.critical_path_lane.as_deref(),
        Some("Realtime") | Some("Anticipative")
    ));
    assert!(!performance.worker_lane_summaries.is_empty());

    let critical_lane_summary = performance
        .worker_lane_summaries
        .iter()
        .find(|summary| {
            Some(match summary.lane {
                GraphExecutionLane::Realtime => "Realtime",
                GraphExecutionLane::Anticipative => "Anticipative",
            }) == performance.critical_path_lane.as_deref()
        })
        .expect("public critical-path lane should resolve to a typed worker-lane summary");
    assert_eq!(
        performance.critical_path_lane_node_count,
        critical_lane_summary.node_count
    );
    assert_eq!(
        performance.critical_path_lane_plugin_backed_node_count,
        critical_lane_summary.plugin_backed_node_count
    );
    assert_eq!(
        performance.critical_path_lane_total_latency_samples,
        critical_lane_summary.total_latency_samples
    );
    assert_eq!(
        supervisor.performance_snapshot().critical_path_lane,
        performance.critical_path_lane
    );
    assert_eq!(
        trace.peak_hot_latency_group_node_count,
        performance.hot_latency_group_node_count
    );
    assert_eq!(
        trace.peak_critical_path_lane,
        performance.critical_path_lane
    );
    assert_eq!(
        trace.peak_critical_path_lane_total_latency_samples,
        performance.critical_path_lane_total_latency_samples
    );
}
