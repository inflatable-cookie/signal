use signal_runtime::{
    RuntimeDeploymentClass, RuntimeFoldDownPolicy, RuntimeImmersiveExportAuthority,
    RuntimeImmersiveExportClass, RuntimeImmersiveExportOutcome,
    RuntimeImmersiveObjectRenderingPosture, RuntimeImmersiveRoomOutcome, RuntimeMonitoringOutcome,
    RuntimeMonitoringSceneAuthority, RuntimeMonitoringSceneClass, RuntimeObservationReport,
    RuntimeRendererCapabilityAuthority, RuntimeRendererCapabilityNegotiationPosture,
    RuntimeRoomPolicyAuthority, RuntimeRoomPolicyClass, RuntimeSpatialAdapterClass,
    RuntimeSpatialBedClass, RuntimeSpatialExecutionMode, RuntimeSpatialExpandedFallbackOutcome,
    RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy, RuntimeSpatialRenderScope,
    RuntimeSpatialTargetEnvironment,
};

pub fn assert_public_spatial_topology(observation: &RuntimeObservationReport) {
    let topology = &observation.execution_topology_summary;
    assert_eq!(topology.spatial_node_count, 2);
    assert_eq!(topology.active_spatial_node_count, 1);
    assert_eq!(topology.bypassed_spatial_node_count, 1);
    assert_eq!(topology.fallback_spatial_node_count, 1);
    assert_eq!(topology.surround_bed_spatial_node_count, 1);
    assert_eq!(topology.object_aware_spatial_node_count, 0);
    assert_eq!(topology.expanded_fallback_spatial_node_count, 1);
    assert_eq!(topology.immersive_spatial_node_count, 1);
    assert_eq!(topology.room_policy_aware_spatial_node_count, 0);
    assert_eq!(topology.fallback_room_policy_spatial_node_count, 1);
    assert_eq!(topology.deployment_spatial_node_count, 1);
    assert_eq!(topology.folded_down_spatial_node_count, 1);
    assert_eq!(topology.fallback_monitoring_scene_spatial_node_count, 1);
    assert_eq!(topology.renderer_capability_spatial_node_count, 1);
    assert_eq!(topology.negotiated_renderer_spatial_node_count, 0);
    assert_eq!(topology.immersive_export_spatial_node_count, 1);
    assert_eq!(topology.fallback_immersive_export_spatial_node_count, 1);

    let stereo = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "spatial-stereo")
        .and_then(|node| node.spatial_execution.as_ref())
        .expect("public stereo node should carry spatial execution");
    assert_eq!(stereo.adapter_class, RuntimeSpatialAdapterClass::Balance);
    assert_eq!(
        stereo.execution_mode,
        RuntimeSpatialExecutionMode::BalanceGroups
    );
    assert_eq!(
        stereo.target_environment,
        RuntimeSpatialTargetEnvironment::SourceLayout
    );
    assert_eq!(stereo.fallback_outcome, None);
    assert_eq!(stereo.bed_class, RuntimeSpatialBedClass::StereoBed);
    assert_eq!(stereo.object_role, None);
    assert_eq!(stereo.object_count, 0);
    assert_eq!(stereo.mix_policy, RuntimeSpatialMixPolicy::BedOnly);
    assert_eq!(stereo.render_scope, RuntimeSpatialRenderScope::BedRender);
    assert_eq!(stereo.expanded_fallback_outcome, None);
    assert_eq!(stereo.balance.as_deref(), Some("-0.200"));

    let surround = topology
        .nodes
        .iter()
        .find(|node| node.node_id == "spatial-surround")
        .and_then(|node| node.spatial_execution.as_ref())
        .expect("public surround node should carry spatial execution");
    assert_eq!(surround.adapter_class, RuntimeSpatialAdapterClass::Balance);
    assert_eq!(
        surround.execution_mode,
        RuntimeSpatialExecutionMode::Bypassed
    );
    assert_eq!(
        surround.fallback_outcome,
        Some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
    );
    assert_eq!(
        surround.bed_class,
        RuntimeSpatialBedClass::CanonicalSurroundBed
    );
    assert_eq!(surround.object_role, None);
    assert_eq!(surround.object_count, 0);
    assert_eq!(
        surround.mix_policy,
        RuntimeSpatialMixPolicy::CollapseToBaselineSpatial
    );
    assert_eq!(surround.render_scope, RuntimeSpatialRenderScope::BedRender);
    assert_eq!(
        surround.expanded_fallback_outcome,
        Some(RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial)
    );
    let surround_immersive = surround
        .immersive_room_policy
        .as_ref()
        .expect("public surround node should carry immersive room policy");
    assert_eq!(
        surround_immersive.object_rendering_posture,
        RuntimeImmersiveObjectRenderingPosture::NotRequested
    );
    assert_eq!(
        surround_immersive.room_policy_class,
        RuntimeRoomPolicyClass::FallbackRoom
    );
    assert_eq!(
        surround_immersive.room_policy_authority,
        RuntimeRoomPolicyAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_immersive.room_outcome,
        RuntimeImmersiveRoomOutcome::BypassRoomPolicy
    );
    let surround_monitoring = surround
        .deployment_monitoring
        .as_ref()
        .expect("public surround node should carry deployment and monitoring summary");
    assert_eq!(
        surround_monitoring.deployment_class,
        RuntimeDeploymentClass::FallbackDeployment
    );
    assert_eq!(
        surround_monitoring.fold_down_policy,
        RuntimeFoldDownPolicy::FoldDownToReferenceBed
    );
    assert_eq!(
        surround_monitoring.monitoring_scene_class,
        RuntimeMonitoringSceneClass::FallbackScene
    );
    assert_eq!(
        surround_monitoring.monitoring_scene_authority,
        RuntimeMonitoringSceneAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_monitoring.monitoring_outcome,
        RuntimeMonitoringOutcome::BypassMonitoringScene
    );
    let surround_export = surround
        .renderer_export
        .as_ref()
        .expect("public surround node should carry renderer and export summary");
    assert_eq!(
        surround_export.renderer_capability_posture,
        RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
    );
    assert_eq!(
        surround_export.capability_authority,
        RuntimeRendererCapabilityAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_export.immersive_export_class,
        RuntimeImmersiveExportClass::FallbackExport
    );
    assert_eq!(
        surround_export.export_authority,
        RuntimeImmersiveExportAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_export.export_outcome,
        RuntimeImmersiveExportOutcome::BypassImmersiveExport
    );
    assert_eq!(surround.balance.as_deref(), Some("0.350"));

    let plugin_chain = &observation.plugin_chain_snapshot;
    assert_eq!(plugin_chain.stage_count, 2);
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| {
            stage.node_id == "spatial-stereo"
                && stage.spatial_execution.as_ref().is_some_and(|spatial| {
                    spatial.execution_mode == RuntimeSpatialExecutionMode::BalanceGroups
                        && spatial.bed_class == RuntimeSpatialBedClass::StereoBed
                        && spatial.mix_policy == RuntimeSpatialMixPolicy::BedOnly
                })
        }));
    assert!(plugin_chain
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .any(|stage| {
            stage.node_id == "spatial-surround"
                && stage.spatial_execution.as_ref().is_some_and(|spatial| {
                    spatial.fallback_outcome
                        == Some(RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
                        && spatial.bed_class == RuntimeSpatialBedClass::CanonicalSurroundBed
                        && spatial.expanded_fallback_outcome
                            == Some(
                                RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial,
                            )
                })
        }));
}
