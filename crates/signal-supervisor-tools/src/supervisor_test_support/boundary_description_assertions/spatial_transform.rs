pub(crate) fn assert_spatial_boundary_text(rendered: &str) {
    for expected in [
        "spatial_boundary: signal.runtime.spatial-boundary",
        "acceptance_task: effigy acceptance:spatial-boundary",
        "contract_path: docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md",
        "surface: RuntimeObservationReport::execution_topology_summary and RuntimeSupervisorReport::observation.execution_topology_summary",
        "surface: RuntimeOfflineRenderContractPreview::chain_contract",
        "spatial_execution.{immersive_room_policy,deployment_monitoring,renderer_export}",
        "cargo test -p signal-runtime --test public_contract_boundary_spatial public_runtime_spatial_boundary_reports_runtime_owned_execution_truth -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-spatial-boundary --format=json",
        "immersive_spatial_node_count",
        "deployment_spatial_node_count",
        "fallback_monitoring_scene_spatial_node_count",
        "renderer_capability_spatial_node_count",
        "immersive_export_spatial_node_count",
        "fallback_room_policy_spatial_stage_count",
        "deployment_spatial_stage_count",
        "fallback_monitoring_scene_spatial_stage_count",
        "renderer_capability_spatial_stage_count",
        "fallback_immersive_export_spatial_stage_count",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_spatial_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.spatial-boundary\"",
        "\"contract_path\":\"docs/contracts/059-renderer-capability-negotiation-and-immersive-export-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:spatial-boundary\"",
        "\"id\":\"runtime-spatial-topology-report\"",
        "\"id\":\"runtime-spatial-plugin-chain-snapshot\"",
        "\"id\":\"runtime-spatial-render-contract-preview\"",
        "\"id\":\"shared-host-spatial-report\"",
        "\"id\":\"runtime-spatial-public-proof\"",
        "immersive_spatial_stage_count",
        "fallback_room_policy_spatial_node_count",
        "deployment_spatial_node_count",
        "folded_down_spatial_stage_count",
        "fallback_monitoring_scene_spatial_stage_count",
        "renderer_capability_spatial_node_count",
        "immersive_export_spatial_stage_count",
        "deployment_monitoring",
        "renderer_export",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_stretch_boundary_text(rendered: &str) {
    for expected in [
        "stretch_boundary: signal.runtime.stretch-boundary",
        "acceptance_task: effigy acceptance:stretch-boundary",
        "surface: RuntimeObservationReport::stretch_engine_snapshot and RuntimeSupervisorReport::observation.stretch_engine_snapshot",
        "surface: RuntimeClipRenderResult::stretch_engine_snapshot and RuntimeOfflineRenderContractPreview::stretch_engine_snapshot",
        "cargo test -p signal-runtime --test public_contract_boundary_stretch public_runtime_stretch_boundary_reports_runtime_owned_engine_truth -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-stretch-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_stretch_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.stretch-boundary\"",
        "\"contract_path\":\"docs/contracts/046-sample-domain-time-stretch-engine-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:stretch-boundary\"",
        "\"id\":\"runtime-stretch-observation-report\"",
        "\"id\":\"runtime-stretch-render-preview-snapshot\"",
        "\"id\":\"shared-host-stretch-report\"",
        "\"id\":\"runtime-stretch-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_marker_analysis_boundary_text(rendered: &str) {
    for expected in [
        "marker_analysis_boundary: signal.runtime.marker-analysis-boundary",
        "acceptance_task: effigy acceptance:marker-analysis-boundary",
        "surface: RuntimeObservationReport::marker_analysis_snapshot and RuntimeSupervisorReport::observation.marker_analysis_snapshot",
        "cargo test -p signal-runtime --test public_contract_boundary_marker_analysis public_runtime_marker_analysis_boundary_reports_runtime_owned_analysis_truth -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-marker-analysis-boundary --format=json",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_marker_analysis_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.marker-analysis-boundary\"",
        "\"contract_path\":\"docs/contracts/047-warp-marker-transient-anchor-and-tempo-assist-analysis-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:marker-analysis-boundary\"",
        "\"id\":\"runtime-marker-analysis-observation-report\"",
        "\"id\":\"shared-host-marker-analysis-report\"",
        "\"id\":\"runtime-marker-analysis-public-proof\"",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_transform_artifact_boundary_text(rendered: &str) {
    for expected in [
        "transform_artifact_boundary: signal.runtime.transform-artifact-boundary",
        "acceptance_task: effigy acceptance:transform-artifact-boundary",
        "contract_path: docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md",
        "surface: RuntimeObservationReport::transform_artifact_snapshot and RuntimeSupervisorReport::observation.transform_artifact_snapshot",
        "surface: RuntimeClipRenderResult::transform_artifact_snapshot and RuntimeOfflineRenderContractPreview::transform_artifact_snapshot",
        "cargo test -p signal-runtime --test public_contract_boundary_transform_artifact public_runtime_transform_artifact_boundary_reports_runtime_owned_artifact_truth -- --exact --nocapture --test-threads=1",
        "cargo run -p signal-supervisor-tools -- --describe-transform-artifact-boundary --format=json",
        "transform_persistence",
        "persistence_posture",
        "retention_outcome",
        "cache_placement_outcome",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_transform_artifact_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.transform-artifact-boundary\"",
        "\"contract_path\":\"docs/contracts/064-asset-session-transform-persistence-retention-and-cache-placement-policy-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:transform-artifact-boundary\"",
        "\"id\":\"runtime-transform-artifact-observation-report\"",
        "\"id\":\"runtime-transform-artifact-render-preview-snapshot\"",
        "\"id\":\"shared-host-transform-artifact-report\"",
        "\"id\":\"runtime-transform-artifact-public-proof\"",
        "transform_persistence",
        "persistence_posture",
        "retention_outcome",
        "cache_placement_outcome",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_preview_transform_boundary_text(rendered: &str) {
    for expected in [
        "preview_transform_boundary: signal.runtime.preview-transform-boundary",
        "acceptance_task: effigy acceptance:preview-transform-boundary",
        "contract_path: docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md",
        "surface: RuntimeObservationReport::preview_transform_snapshot and RuntimeSupervisorReport::observation.preview_transform_snapshot",
        "surface: RuntimeClipRenderResult::preview_transform_snapshot and RuntimeOfflineRenderContractPreview::preview_transform_snapshot",
        "cargo test -p signal-runtime public_runtime_preview_transform_boundary_reports_runtime_owned_preview_truth",
        "cargo run -p signal-supervisor-tools -- --describe-preview-transform-boundary --format=json",
        "preview_device_policy",
        "preview_workflow",
        "queue_posture",
        "audition_continuity_outcome",
        "transform_scheduling_outcome",
        "routing_posture",
        "audition_sink_class",
        "low_latency_device_policy_outcome",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}

pub(crate) fn assert_preview_transform_boundary_json(rendered: &str) {
    for expected in [
        "\"boundary\":\"signal.runtime.preview-transform-boundary\"",
        "\"contract_path\":\"docs/contracts/063-preview-browser-queue-media-audition-and-transform-scheduling-contract.md\"",
        "\"acceptance_task\":\"effigy acceptance:preview-transform-boundary\"",
        "\"id\":\"runtime-preview-transform-observation-report\"",
        "\"id\":\"runtime-preview-transform-render-preview-snapshot\"",
        "\"id\":\"shared-host-preview-transform-report\"",
        "\"id\":\"runtime-preview-transform-public-proof\"",
        "preview_device_policy",
        "preview_workflow",
        "queue_posture",
        "audition_continuity_outcome",
        "transform_scheduling_outcome",
        "routing_posture",
        "audition_sink_class",
        "low_latency_device_policy_outcome",
    ] {
        assert!(rendered.contains(expected), "missing {expected}");
    }
}
