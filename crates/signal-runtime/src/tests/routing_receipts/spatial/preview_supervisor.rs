use super::super::super::*;

pub(super) fn assert_spatial_preview_and_supervisor_receipts(
    runtime: &SignalRuntime,
    observation: &RuntimeObservationReport,
) {
    let _supervisor = RuntimeSupervisorReport::capture(runtime, &RuntimeEventRecorder::default());

    assert_eq!(observation.execution_topology_summary.spatial_node_count, 2);
}
