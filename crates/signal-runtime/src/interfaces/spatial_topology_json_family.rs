use super::*;

mod spatial;
mod topology;

pub(super) fn json_runtime_spatial_execution_summary(
    summary: &RuntimeSpatialExecutionSummary,
) -> String {
    spatial::json_runtime_spatial_execution_summary(summary)
}

pub(super) fn json_runtime_execution_topology_summary(
    summary: &RuntimeExecutionTopologySummary,
) -> String {
    topology::json_runtime_execution_topology_summary(summary)
}
