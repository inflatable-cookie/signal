#[path = "support/public_contract_boundary_graph_plugin_surface.rs"]
mod public_contract_boundary_graph_plugin_surface_support;
#[path = "support/public_contract_boundary_spatial_preview_render.rs"]
mod public_contract_boundary_spatial_preview_render_support;
#[path = "support/public_contract_boundary_spatial_topology.rs"]
mod public_contract_boundary_spatial_topology_support;

use public_contract_boundary_graph_plugin_surface_support::apply_public_spatial_graph;
use public_contract_boundary_spatial_preview_render_support::assert_public_spatial_rendering;
use public_contract_boundary_spatial_topology_support::assert_public_spatial_topology;
use signal_runtime::{
    HandshakeRequest, RuntimeConfig, RuntimeConfigRequest, RuntimeEventRecorder,
    RuntimeLifecycleApi, RuntimeObservationReport, RuntimeSupervisorReport, SignalRuntime,
};

#[test]
fn public_runtime_spatial_boundary_reports_runtime_owned_execution_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(HandshakeRequest {
            client_version: "public-runtime-spatial-boundary".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public runtime spatial handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public runtime spatial configure should succeed");
    apply_public_spatial_graph(&mut runtime, "graph:public:spatial");

    let recorder = RuntimeEventRecorder::default();
    let observation = RuntimeObservationReport::capture(&runtime, &recorder);
    assert_public_spatial_topology(&observation);

    let supervisor = RuntimeSupervisorReport::capture(&runtime, &recorder);
    assert_public_spatial_rendering(&observation, &supervisor);
}
