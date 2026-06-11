use signal_runtime::{
    RuntimeDeploymentClass, RuntimeFoldDownPolicy, RuntimeImmersiveExportClass,
    RuntimeImmersiveExportOutcome, RuntimeImmersiveRoomOutcome, RuntimeMonitoringOutcome,
    RuntimeMonitoringSceneClass, RuntimeObservationReport, RuntimeOfflineRenderContractPreview,
    RuntimeRendererCapabilityNegotiationPosture, RuntimeRoomPolicyClass, RuntimeSpatialBedClass,
    RuntimeSpatialExecutionMode, RuntimeSpatialExpandedFallbackOutcome,
    RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy, RuntimeSpatialRenderScope,
    RuntimeSupervisorReport,
};

pub fn assert_public_spatial_preview(preview: &RuntimeOfflineRenderContractPreview) {
    assert_eq!(preview.chain_contract.spatial_stage_count, 2);
    assert_eq!(preview.chain_contract.active_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.bypassed_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.fallback_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.surround_bed_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.object_aware_spatial_stage_count, 0);
    assert_eq!(
        preview.chain_contract.expanded_fallback_spatial_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.immersive_spatial_stage_count, 1);
    assert_eq!(
        preview.chain_contract.room_policy_aware_spatial_stage_count,
        0
    );
    assert_eq!(
        preview
            .chain_contract
            .fallback_room_policy_spatial_stage_count,
        1
    );
    assert_eq!(preview.chain_contract.deployment_spatial_stage_count, 1);
    assert_eq!(preview.chain_contract.folded_down_spatial_stage_count, 1);
    assert_eq!(
        preview
            .chain_contract
            .fallback_monitoring_scene_spatial_stage_count,
        1
    );
    assert_eq!(
        preview
            .chain_contract
            .renderer_capability_spatial_stage_count,
        1
    );
    assert_eq!(
        preview
            .chain_contract
            .negotiated_renderer_spatial_stage_count,
        0
    );
    assert_eq!(
        preview.chain_contract.immersive_export_spatial_stage_count,
        1
    );
    assert_eq!(
        preview
            .chain_contract
            .fallback_immersive_export_spatial_stage_count,
        1
    );
    assert!(preview.chain_contract.spatial_stages.iter().any(|stage| {
        stage.node_id == "spatial-stereo"
            && stage.spatial.execution_mode == RuntimeSpatialExecutionMode::BalanceGroups
            && stage.spatial.bed_class == RuntimeSpatialBedClass::StereoBed
            && stage.spatial.mix_policy == RuntimeSpatialMixPolicy::BedOnly
    }));
    assert!(preview.chain_contract.spatial_stages.iter().any(|stage| {
        stage.node_id == "spatial-surround"
            && stage.spatial.fallback_outcome
                == Some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
            && stage.spatial.expanded_fallback_outcome
                == Some(RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial)
            && stage.spatial.render_scope == RuntimeSpatialRenderScope::BedRender
            && stage
                .spatial
                .immersive_room_policy
                .as_ref()
                .is_some_and(|immersive| {
                    immersive.room_policy_class == RuntimeRoomPolicyClass::FallbackRoom
                        && immersive.room_outcome == RuntimeImmersiveRoomOutcome::BypassRoomPolicy
                })
            && stage
                .spatial
                .deployment_monitoring
                .as_ref()
                .is_some_and(|monitoring| {
                    monitoring.deployment_class == RuntimeDeploymentClass::FallbackDeployment
                        && monitoring.fold_down_policy
                            == RuntimeFoldDownPolicy::FoldDownToReferenceBed
                        && monitoring.monitoring_scene_class
                            == RuntimeMonitoringSceneClass::FallbackScene
                        && monitoring.monitoring_outcome
                            == RuntimeMonitoringOutcome::BypassMonitoringScene
                })
            && stage
                .spatial
                .renderer_export
                .as_ref()
                .is_some_and(|renderer| {
                    renderer.renderer_capability_posture
                        == RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
                        && renderer.immersive_export_class
                            == RuntimeImmersiveExportClass::FallbackExport
                        && renderer.export_outcome
                            == RuntimeImmersiveExportOutcome::BypassImmersiveExport
                })
    }));
}

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
