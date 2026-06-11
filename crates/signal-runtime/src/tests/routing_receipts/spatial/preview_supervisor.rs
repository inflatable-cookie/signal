use super::super::super::*;

pub(super) fn assert_spatial_preview_and_supervisor_receipts(
    runtime: &SignalRuntime,
    observation: &RuntimeObservationReport,
) {
    let handoff = runtime.get_plugin_recall_handoff_snapshot();
    let preview = RuntimeOfflineRenderContractPreview::from_runtime_state(
        &RuntimeOfflineRenderRequest {
            request_id: "render:spatial-preview".into(),
            timeline_start_samples: 0,
            duration_samples: 24_000,
            export_sample_rate_hz: 48_000,
            include_main_mix: true,
            artifact_root_path: None,
            stem_targets: Vec::new(),
            freeze_artifacts: Vec::new(),
        },
        &runtime.get_execution_topology_summary(),
        &runtime.get_clip_processing_pipeline_snapshot(),
        &runtime.get_media_pipeline_snapshot(),
        &runtime.get_tempo_map_snapshot(),
        &runtime.get_marker_analysis_snapshot(),
        &handoff,
    )
    .expect("build offline render spatial preview");
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
    assert!(preview
        .chain_contract
        .spatial_stages
        .iter()
        .any(|stage| stage.node_id == "spatial-stereo"
            && stage.spatial.execution_mode == crate::RuntimeSpatialExecutionMode::BalanceGroups
            && stage.spatial.bed_class == crate::RuntimeSpatialBedClass::StereoBed
            && stage.spatial.mix_policy == crate::RuntimeSpatialMixPolicy::BedOnly));
    assert!(preview
        .chain_contract
        .spatial_stages
        .iter()
        .any(|stage| stage.node_id == "spatial-surround"
            && stage.spatial.fallback_outcome
                == Some(crate::RuntimeSpatialFallbackOutcome::BypassSpatialProcessing)
            && stage.spatial.expanded_fallback_outcome
                == Some(crate::RuntimeSpatialExpandedFallbackOutcome::CollapseToBaselineSpatial)
            && stage.spatial.bed_class == crate::RuntimeSpatialBedClass::CanonicalSurroundBed
            && stage
                .spatial
                .immersive_room_policy
                .as_ref()
                .is_some_and(|immersive| {
                    immersive.room_policy_class == crate::RuntimeRoomPolicyClass::FallbackRoom
                        && immersive.room_outcome
                            == crate::RuntimeImmersiveRoomOutcome::BypassRoomPolicy
                })
            && stage
                .spatial
                .deployment_monitoring
                .as_ref()
                .is_some_and(|monitoring| {
                    monitoring.deployment_class == crate::RuntimeDeploymentClass::FallbackDeployment
                        && monitoring.fold_down_policy
                            == crate::RuntimeFoldDownPolicy::FoldDownToReferenceBed
                        && monitoring.monitoring_scene_class
                            == crate::RuntimeMonitoringSceneClass::FallbackScene
                        && monitoring.monitoring_outcome
                            == crate::RuntimeMonitoringOutcome::BypassMonitoringScene
                })
            && stage
                .spatial
                .renderer_export
                .as_ref()
                .is_some_and(|renderer| {
                    renderer.renderer_capability_posture
                        == crate::RuntimeRendererCapabilityNegotiationPosture::FallbackNegotiation
                        && renderer.immersive_export_class
                            == crate::RuntimeImmersiveExportClass::FallbackExport
                        && renderer.export_outcome
                            == crate::RuntimeImmersiveExportOutcome::BypassImmersiveExport
                })));

    let _supervisor = RuntimeSupervisorReport::capture(runtime, &RuntimeEventRecorder::default());

    assert_eq!(observation.execution_topology_summary.spatial_node_count, 2);
}
