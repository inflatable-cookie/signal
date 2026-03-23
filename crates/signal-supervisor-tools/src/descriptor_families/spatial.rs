use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpatialBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpatialBoundarySurface {
    id: &'static str,
    kind: SpatialBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SpatialBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl SpatialBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn spatial_boundary_surfaces() -> &'static [SpatialBoundarySurface] {
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

fn spatial_boundary_validation_steps() -> &'static [SpatialBoundaryValidationStep] {
    &[
        SpatialBoundaryValidationStep {
            id: "runtime-spatial-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_spatial_boundary_reports_runtime_owned_execution_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect surround-bed, mix-policy, render-scope, immersive room-policy, deployment class, fold-down policy, monitoring-scene posture, bounded renderer negotiation, immersive export outcome, and render-preview receipts through public runtime surfaces alone.",
        },
        SpatialBoundaryValidationStep {
            id: "local-host-spatial-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_spatial_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned richer spatial, immersive room-policy, and deployment-monitoring receipts on supervisor export.",
        },
        SpatialBoundaryValidationStep {
            id: "server-host-spatial-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_spatial_truth",
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

pub(crate) fn render_spatial_boundary_text() -> String {
    let mut rendered = format!(
        "spatial_boundary: {SPATIAL_BOUNDARY}\ncontract_path: {SPATIAL_CONTRACT_PATH}\nacceptance_task: {SPATIAL_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in spatial_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in spatial_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned richer spatial, deployment-monitoring, and bounded renderer or immersive-export receipts through runtime, supervisor, and stable host-edge surfaces, but true renderer-backed object execution and monitoring-scene breadth still belong to later g08 work",
        "this closes the bounded renderer-capability and immersive-export consumer seam, not renderer-vendor package schemas, publication workflows, or product-local immersive export UX",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_spatial_boundary_json() -> String {
    let surfaces = spatial_boundary_surfaces()
        .iter()
        .map(|surface| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"runtime_anchor\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(surface.id),
                json_string(surface.kind.label()),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = spatial_boundary_validation_steps()
        .iter()
        .map(|step| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(step.id),
                json_string(step.command),
                json_string(step.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred_scope = [
        "the shared boundary now proves runtime-owned richer spatial, deployment-monitoring, and bounded renderer or immersive-export receipts through runtime, supervisor, and stable host-edge surfaces, but true renderer-backed object execution and monitoring-scene breadth still belong to later g08 work",
        "this closes the bounded renderer-capability and immersive-export consumer seam, not renderer-vendor package schemas, publication workflows, or product-local immersive export UX",
    ]
    .iter()
    .map(|scope| json_string(scope))
    .collect::<Vec<_>>()
    .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"surface_count\":{},",
            "\"surfaces\":[{}],",
            "\"validation_step_count\":{},",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(SPATIAL_BOUNDARY),
        json_string(SPATIAL_CONTRACT_PATH),
        json_string(SPATIAL_ACCEPTANCE_TASK),
        spatial_boundary_surfaces().len(),
        surfaces,
        spatial_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
