use signal_runtime::{RuntimeObservationReport, RuntimeSupervisorReport};

pub fn assert_public_spatial_rendering(
    observation: &RuntimeObservationReport,
    supervisor: &RuntimeSupervisorReport,
) {
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.spatial_node_count, 2);
    assert_eq!(topology.active_spatial_node_count, 1);
    assert_eq!(topology.fallback_spatial_node_count, 1);
    assert_eq!(topology.surround_bed_spatial_node_count, 1);
    assert_eq!(topology.expanded_fallback_spatial_node_count, 1);
    assert_eq!(topology.immersive_spatial_node_count, 1);
    assert_eq!(topology.fallback_room_policy_spatial_node_count, 1);
    assert_eq!(topology.deployment_spatial_node_count, 1);
    assert_eq!(topology.folded_down_spatial_node_count, 1);
    assert_eq!(topology.fallback_monitoring_scene_spatial_node_count, 1);
    assert_eq!(topology.renderer_capability_spatial_node_count, 1);
    assert_eq!(topology.negotiated_renderer_spatial_node_count, 0);
    assert_eq!(topology.immersive_export_spatial_node_count, 1);
    assert_eq!(topology.fallback_immersive_export_spatial_node_count, 1);

    let supervisor_topology = &supervisor.observation.execution_topology_summary;
    assert_eq!(supervisor_topology, topology);
}
