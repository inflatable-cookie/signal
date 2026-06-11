#[path = "support/public_host_edge_runtime_surface.rs"]
mod public_host_edge_runtime_surface;

use public_host_edge_runtime_surface::apply_public_spatial_graph;
use signal_host_local::LocalRuntimeHost;
use signal_runtime::{
    RuntimeConfig, RuntimeConfigRequest, RuntimeDeploymentClass, RuntimeFoldDownPolicy,
    RuntimeImmersiveObjectRenderingPosture, RuntimeImmersiveRoomOutcome, RuntimeLifecycleApi,
    RuntimeMonitoringOutcome, RuntimeMonitoringSceneAuthority, RuntimeMonitoringSceneClass,
    RuntimeRendererCapabilityAuthority, RuntimeRendererCapabilityNegotiationPosture,
    RuntimeRoomPolicyAuthority, RuntimeRoomPolicyClass, RuntimeSpatialBedClass,
    RuntimeSpatialExecutionMode, RuntimeSpatialExpandedFallbackOutcome,
    RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy, SignalRuntime,
};

#[test]
fn local_shared_host_edge_exports_runtime_spatial_truth() {
    let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    runtime
        .handshake(signal_runtime::HandshakeRequest {
            client_version: "public-host-local-spatial".into(),
            anticipative_preferred: true,
            max_sample_rate_hint: Some(96_000),
        })
        .expect("public local spatial handshake should succeed");
    runtime
        .configure(RuntimeConfigRequest::new(48_000, 256))
        .expect("public local spatial configure should succeed");
    apply_public_spatial_graph(&mut runtime, "graph:host-local:spatial");

    let host = LocalRuntimeHost::new(runtime);
    let report = host.supervisor_report();
    let topology = &report.observation.execution_topology_summary;
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
    assert!(topology.nodes.iter().any(|node| {
        node.node_id == "spatial-stereo"
            && node.spatial_execution.as_ref().is_some_and(|spatial| {
                spatial.execution_mode == RuntimeSpatialExecutionMode::BalanceGroups
                    && spatial.bed_class == RuntimeSpatialBedClass::StereoBed
                    && spatial.mix_policy == RuntimeSpatialMixPolicy::BedOnly
            })
    }));
    assert!(report
        .observation
        .plugin_chain_snapshot
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
                        && spatial
                            .immersive_room_policy
                            .as_ref()
                            .is_some_and(|immersive| {
                                immersive.object_rendering_posture
                                    == RuntimeImmersiveObjectRenderingPosture::NotRequested
                                    && immersive.room_policy_class
                                        == RuntimeRoomPolicyClass::FallbackRoom
                                    && immersive.room_policy_authority
                                        == RuntimeRoomPolicyAuthority::RuntimeDefault
                                    && immersive.room_outcome
                                        == RuntimeImmersiveRoomOutcome::BypassRoomPolicy
                            })
                        && spatial
                            .deployment_monitoring
                            .as_ref()
                            .is_some_and(|monitoring| {
                                monitoring.deployment_class
                                    == RuntimeDeploymentClass::FallbackDeployment
                                    && monitoring.fold_down_policy
                                        == RuntimeFoldDownPolicy::FoldDownToReferenceBed
                                    && monitoring.monitoring_scene_class
                                        == RuntimeMonitoringSceneClass::FallbackScene
                                    && monitoring.monitoring_scene_authority
                                        == RuntimeMonitoringSceneAuthority::RuntimeDefault
                                    && monitoring.monitoring_outcome
                                        == RuntimeMonitoringOutcome::BypassMonitoringScene
                            })
                        && spatial.renderer_export.as_ref().is_some_and(|renderer| {
                            renderer.renderer_capability_posture
                                == RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
                                && renderer.capability_authority
                                    == RuntimeRendererCapabilityAuthority::RuntimeDefault
                        })
                })
        }));

}
