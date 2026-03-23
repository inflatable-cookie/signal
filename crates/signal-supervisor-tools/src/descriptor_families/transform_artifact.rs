use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TransformArtifactBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TransformArtifactBoundarySurface {
    id: &'static str,
    kind: TransformArtifactBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TransformArtifactBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl TransformArtifactBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn transform_artifact_boundary_surfaces() -> &'static [TransformArtifactBoundarySurface] {
    &[
        TransformArtifactBoundarySurface {
            id: "runtime-transform-artifact-observation-report",
            kind: TransformArtifactBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::transform_artifact_snapshot and RuntimeSupervisorReport::observation.transform_artifact_snapshot",
            runtime_anchor:
                "RuntimeTransformArtifactSnapshot::{clip_count, ready_clip_count, pending_media_clip_count, degraded_clip_count, invalidated_clip_count, unsupported_clip_count, cached_media_ready_clip_count, reusable_clip_count, requires_render_clip_count, guarded_reuse_clip_count, transform_persistence} + RuntimeTransformPersistenceSummary::{persistence_posture, retention_policy_class, retention_authority, retention_outcome, cache_placement_posture, cache_placement_authority, cache_placement_outcome}",
            rationale:
                "Keeps post-warp render, cache readiness, invalidation, reuse, retention, and cache-placement truth on one runtime-owned report seam instead of host-local preview-cache or browser-local persistence policy.",
        },
        TransformArtifactBoundarySurface {
            id: "runtime-transform-artifact-render-preview-snapshot",
            kind: TransformArtifactBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeClipRenderResult::transform_artifact_snapshot and RuntimeOfflineRenderContractPreview::transform_artifact_snapshot",
            runtime_anchor:
                "RuntimeClipRenderResult::transform_artifact_snapshot + RuntimeOfflineRenderContractPreview::transform_artifact_snapshot",
            rationale:
                "Proves clip render and offline preview carry the same runtime-owned transform-artifact and transform-persistence posture instead of splitting render, cache, and retention truth across private paths.",
        },
        TransformArtifactBoundarySurface {
            id: "shared-host-transform-artifact-report",
            kind: TransformArtifactBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned transform-artifact and transform-persistence receipts without host-local preview-cache or cache-policy reconstruction.",
        },
    ]
}

fn transform_artifact_boundary_validation_steps(
) -> &'static [TransformArtifactBoundaryValidationStep] {
    &[
        TransformArtifactBoundaryValidationStep {
            id: "runtime-transform-artifact-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_transform_artifact_boundary_reports_runtime_owned_artifact_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned transform-artifact readiness, invalidation, reuse, retention, and cache-placement truth through public runtime surfaces alone.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "local-host-transform-artifact-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_transform_artifact_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned transform-artifact and transform-persistence receipts instead of rebuilding local preview-cache or persistence posture.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "server-host-transform-artifact-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_transform_artifact_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned transform-artifact and transform-persistence receipts without server-local cache heuristics.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools transform_artifact_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable transform-artifact and transform-persistence boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only closure notes.",
        },
        TransformArtifactBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-transform-artifact-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared transform-artifact and transform-persistence proof seam without reading host-local preview-cache or cache-policy glue.",
        },
    ]
}

pub(crate) fn render_transform_artifact_boundary_text() -> String {
    let mut rendered = format!(
        "transform_artifact_boundary: {TRANSFORM_ARTIFACT_BOUNDARY}\ncontract_path: {TRANSFORM_ARTIFACT_CONTRACT_PATH}\nacceptance_task: {TRANSFORM_ARTIFACT_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in transform_artifact_boundary_surfaces() {
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
    for step in transform_artifact_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned post-warp render, cache readiness, invalidation, reuse, persistence, retention, and cache-placement receipts through runtime, supervisor, clip-render, offline preview, and stable host-edge surfaces, but fuller session persistence UX, cloud sync, quota, and eviction depth still belongs to later g08 work",
        "this closes the bounded transform-artifact and transform-persistence consumer seam, not a full cache engine, browser storage ledger, or product-local transform management workflow",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_transform_artifact_boundary_json() -> String {
    let surfaces = transform_artifact_boundary_surfaces()
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
    let validation_steps = transform_artifact_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned post-warp render, cache readiness, invalidation, reuse, persistence, retention, and cache-placement receipts through runtime, supervisor, clip-render, offline preview, and stable host-edge surfaces, but fuller session persistence UX, cloud sync, quota, and eviction depth still belongs to later g08 work",
        "this closes the bounded transform-artifact and transform-persistence consumer seam, not a full cache engine, browser storage ledger, or product-local transform management workflow",
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
        json_string(TRANSFORM_ARTIFACT_BOUNDARY),
        json_string(TRANSFORM_ARTIFACT_CONTRACT_PATH),
        json_string(TRANSFORM_ARTIFACT_ACCEPTANCE_TASK),
        transform_artifact_boundary_surfaces().len(),
        surfaces,
        transform_artifact_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
