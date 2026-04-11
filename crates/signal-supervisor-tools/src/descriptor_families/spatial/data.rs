#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SpatialBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SpatialBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: SpatialBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SpatialBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

impl SpatialBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

pub(super) fn spatial_boundary_surfaces() -> &'static [SpatialBoundarySurface] {
    &[
        SpatialBoundarySurface {
            id: "runtime-spatial-topology-report",
            kind: SpatialBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::execution_topology_summary and RuntimeSupervisorReport::observation.execution_topology_summary",
            runtime_anchor:
                "RuntimeExecutionTopologySummary::{spatial_node_count,active_spatial_node_count,bypassed_spatial_node_count,fallback_spatial_node_count,surround_bed_spatial_node_count,object_aware_spatial_node_count,expanded_fallback_spatial_node_count,immersive_spatial_node_count,room_policy_aware_spatial_node_count,fallback_room_policy_spatial_node_count,deployment_spatial_node_count,folded_down_spatial_node_count,fallback_monitoring_scene_spatial_node_count,renderer_capability_spatial_node_count,negotiated_renderer_spatial_node_count,immersive_export_spatial_node_count,fallback_immersive_export_spatial_node_count} + RuntimeExecutionNodeSummary::spatial_execution.{immersive_room_policy,deployment_monitoring,renderer_export}",
            rationale:
                "Keeps surround-bed, mix-policy, render-scope, immersive room-policy, deployment class, fold-down posture, monitoring-scene outcome, and renderer or export posture on one runtime-owned topology surface instead of host-local or renderer-local reinterpretation.",
        },
        SpatialBoundarySurface {
            id: "runtime-spatial-plugin-chain-snapshot",
            kind: SpatialBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_chain_snapshot and RuntimeSupervisorReport::observation.plugin_chain_snapshot",
            runtime_anchor:
                "RuntimePluginChainStageSnapshot::spatial_execution.{immersive_room_policy,deployment_monitoring,renderer_export}",
            rationale:
                "Lets downstream consumers inspect richer spatial stage meaning, including immersive room-policy posture, deployment class, fold-down policy, fallback monitoring outcomes, and bounded renderer or export posture, on live plugin-chain receipts instead of inferring renderer behavior from adapter-private control names.",
        },
        SpatialBoundarySurface {
            id: "runtime-spatial-render-contract-preview",
            kind: SpatialBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeOfflineRenderContractPreview::chain_contract",
            runtime_anchor:
                "RuntimeOfflineRenderChainDependencyPreview::{spatial_stage_count,active_spatial_stage_count,bypassed_spatial_stage_count,fallback_spatial_stage_count,surround_bed_spatial_stage_count,object_aware_spatial_stage_count,expanded_fallback_spatial_stage_count,immersive_spatial_stage_count,room_policy_aware_spatial_stage_count,fallback_room_policy_spatial_stage_count,deployment_spatial_stage_count,folded_down_spatial_stage_count,fallback_monitoring_scene_spatial_stage_count,renderer_capability_spatial_stage_count,negotiated_renderer_spatial_stage_count,immersive_export_spatial_stage_count,fallback_immersive_export_spatial_stage_count,spatial_stages}",
            rationale:
                "Carries the same richer spatial execution, immersive room-policy, deployment-monitoring posture, and bounded renderer or export truth into deferred render preview instead of rebuilding export policy per render path.",
        },
        SpatialBoundarySurface {
            id: "shared-host-spatial-report",
            kind: SpatialBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned richer spatial, deployment, monitoring, and renderer or export receipts without host-local speaker heuristics or renderer-local reinterpretation.",
        },
    ]
}

pub(super) fn spatial_boundary_validation_steps() -> &'static [SpatialBoundaryValidationStep] {
    &[
        SpatialBoundaryValidationStep {
            id: "runtime-spatial-public-proof",
            command:
                "cargo test -p signal-runtime --test public_contract_boundary_spatial public_runtime_spatial_boundary_reports_runtime_owned_execution_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves a downstream-style runtime consumer can inspect surround-bed, mix-policy, render-scope, immersive room-policy, deployment class, fold-down policy, monitoring-scene posture, bounded renderer negotiation, immersive export outcome, and render-preview receipts through public runtime surfaces alone.",
        },
        SpatialBoundaryValidationStep {
            id: "local-host-spatial-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_spatial local_shared_host_edge_exports_runtime_spatial_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge forwards runtime-owned richer spatial, immersive room-policy, and deployment-monitoring receipts on supervisor export.",
        },
        SpatialBoundaryValidationStep {
            id: "server-host-spatial-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_spatial server_shared_host_edge_exports_runtime_spatial_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned richer spatial, deployment, monitoring, and renderer or export receipts without host-local or renderer-local reinterpretation.",
        },
        SpatialBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools spatial_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable spatial boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        SpatialBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-spatial-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared spatial execution seam without reading adapter-private renderer glue or host-local pan policy.",
        },
    ]
}
