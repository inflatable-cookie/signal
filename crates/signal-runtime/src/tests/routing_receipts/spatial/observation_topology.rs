use super::super::super::*;

pub(super) fn assert_spatial_execution_topology(observation: &RuntimeObservationReport) {
    assert_eq!(observation.execution_topology_summary.spatial_node_count, 2);
    assert_eq!(
        observation
            .execution_topology_summary
            .active_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .bypassed_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .fallback_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .surround_bed_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .object_aware_spatial_node_count,
        0
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .expanded_fallback_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .immersive_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .room_policy_aware_spatial_node_count,
        0
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .fallback_room_policy_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .deployment_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .folded_down_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .fallback_monitoring_scene_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .renderer_capability_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .negotiated_renderer_spatial_node_count,
        0
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .immersive_export_spatial_node_count,
        1
    );
    assert_eq!(
        observation
            .execution_topology_summary
            .fallback_immersive_export_spatial_node_count,
        1
    );

    let stereo = observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "spatial-stereo")
        .and_then(|node| node.spatial_execution.as_ref())
        .expect("stereo node should carry spatial execution summary");
    assert_eq!(
        stereo.adapter_class,
        crate::RuntimeSpatialAdapterClass::Balance
    );
    assert_eq!(
        stereo.execution_mode,
        crate::RuntimeSpatialExecutionMode::BalanceGroups
    );
    assert_eq!(stereo.fallback_outcome, None);
    assert_eq!(
        stereo.target_environment,
        crate::RuntimeSpatialTargetEnvironment::SourceLayout
    );
    assert_eq!(stereo.bed_class, crate::RuntimeSpatialBedClass::StereoBed);
    assert_eq!(stereo.object_role, None);
    assert_eq!(stereo.object_count, 0);
    assert_eq!(stereo.mix_policy, crate::RuntimeSpatialMixPolicy::BedOnly);
    assert_eq!(
        stereo.render_scope,
        crate::RuntimeSpatialRenderScope::BedRender
    );
    assert_eq!(stereo.expanded_fallback_outcome, None);
    assert_eq!(stereo.balance.as_deref(), Some("-0.200"));
    assert_eq!(stereo.immersive_room_policy, None);
    assert_eq!(stereo.deployment_monitoring, None);
    assert_eq!(stereo.renderer_export, None);

    let surround = observation
        .execution_topology_summary
        .nodes
        .iter()
        .find(|node| node.node_id == "spatial-surround")
        .and_then(|node| node.spatial_execution.as_ref())
        .expect("surround node should carry spatial execution summary");
    assert_eq!(
        surround.execution_mode,
        crate::RuntimeSpatialExecutionMode::Bypassed
    );
    assert_eq!(
        surround.fallback_outcome,
        Some(crate::RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
    );
    assert_eq!(
        surround.bed_class,
        crate::RuntimeSpatialBedClass::CanonicalSurroundBed
    );
    assert_eq!(surround.object_role, None);
    assert_eq!(surround.object_count, 0);
    assert_eq!(
        surround.mix_policy,
        crate::RuntimeSpatialMixPolicy::CollapseToBaselineSpatial
    );
    assert_eq!(
        surround.render_scope,
        crate::RuntimeSpatialRenderScope::BedRender
    );
    assert_eq!(
        surround.expanded_fallback_outcome,
        Some(crate::RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial)
    );
    let surround_immersive = surround
        .immersive_room_policy
        .as_ref()
        .expect("surround node should carry immersive room policy summary");
    assert_eq!(
        surround_immersive.object_rendering_posture,
        crate::RuntimeImmersiveObjectRenderingPosture::NotRequested
    );
    assert_eq!(
        surround_immersive.room_policy_class,
        crate::RuntimeRoomPolicyClass::FallbackRoom
    );
    assert_eq!(
        surround_immersive.room_policy_authority,
        crate::RuntimeRoomPolicyAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_immersive.room_outcome,
        crate::RuntimeImmersiveRoomOutcome::BypassRoomPolicy
    );
    let surround_monitoring = surround
        .deployment_monitoring
        .as_ref()
        .expect("surround node should carry deployment and monitoring summary");
    assert_eq!(
        surround_monitoring.deployment_class,
        crate::RuntimeDeploymentClass::FallbackDeployment
    );
    assert_eq!(
        surround_monitoring.fold_down_policy,
        crate::RuntimeFoldDownPolicy::FoldDownToReferenceBed
    );
    assert_eq!(
        surround_monitoring.monitoring_scene_class,
        crate::RuntimeMonitoringSceneClass::FallbackScene
    );
    assert_eq!(
        surround_monitoring.monitoring_scene_authority,
        crate::RuntimeMonitoringSceneAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_monitoring.monitoring_outcome,
        crate::RuntimeMonitoringOutcome::BypassMonitoringScene
    );
    let surround_export = surround
        .renderer_export
        .as_ref()
        .expect("surround node should carry renderer and export summary");
    assert_eq!(
        surround_export.renderer_capability_posture,
        crate::RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
    );
    assert_eq!(
        surround_export.capability_authority,
        crate::RuntimeRendererCapabilityAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_export.immersive_export_class,
        crate::RuntimeImmersiveExportClass::FallbackExport
    );
    assert_eq!(
        surround_export.export_authority,
        crate::RuntimeImmersiveExportAuthority::RuntimeDefault
    );
    assert_eq!(
        surround_export.export_outcome,
        crate::RuntimeImmersiveExportOutcome::BypassImmersiveExport
    );
    assert_eq!(surround.balance.as_deref(), Some("0.350"));
    assert_eq!(
        surround.output_layout.canonical_layout,
        Some(crate::RuntimeCanonicalChannelLayout::Surround5_1)
    );

    let plugin_stage_count = observation
        .plugin_chain_snapshot
        .chains
        .iter()
        .flat_map(|chain| chain.stages.iter())
        .filter(|stage| stage.spatial_execution.is_some())
        .count();
    assert_eq!(plugin_stage_count, 2);
}
