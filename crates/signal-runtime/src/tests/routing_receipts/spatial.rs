use super::super::*;

#[path = "spatial/observation_topology.rs"]
mod observation_topology;
#[path = "spatial/preview_supervisor.rs"]
mod preview_supervisor;

#[test]
fn runtime_observation_and_render_preview_surface_spatial_execution_receipts() {
    let runtime = prepare_spatial_runtime();
    let observation = RuntimeObservationReport::capture(&runtime, &RuntimeEventRecorder::default());

    observation_topology::assert_spatial_execution_topology(&observation);
    preview_supervisor::assert_spatial_preview_and_supervisor_receipts(&runtime, &observation);
}
