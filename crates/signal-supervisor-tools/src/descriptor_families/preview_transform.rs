use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PreviewTransformBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PreviewTransformBoundarySurface {
    id: &'static str,
    kind: PreviewTransformBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PreviewTransformBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl PreviewTransformBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn preview_transform_boundary_surfaces() -> &'static [PreviewTransformBoundarySurface] {
    &[
        PreviewTransformBoundarySurface {
            id: "runtime-preview-transform-observation-report",
            kind: PreviewTransformBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::preview_transform_snapshot and RuntimeSupervisorReport::observation.preview_transform_snapshot",
            runtime_anchor:
                "RuntimePreviewTransformServiceSnapshot::{clip_count,active_audition_clip_count,scrub_supported_clip_count,ready_clip_count,pending_clip_count,degraded_clip_count,invalidated_clip_count,unsupported_clip_count,stretch_aligned_clip_count,artifact_backed_clip_count,fallback_clip_count,preview_device_policy,preview_workflow} + RuntimePreviewDevicePolicySummary::{routing_posture,audition_sink_class,audition_sink_authority,low_latency_device_policy_class,low_latency_device_policy_outcome} + RuntimePreviewWorkflowSummary::{queue_posture,queue_class,queue_outcome,audition_posture,audition_authority,audition_continuity_outcome,transform_scheduling_posture,transform_scheduling_authority,transform_scheduling_outcome}",
            rationale:
                "Keeps low-latency audition, scrub support, readiness, degraded-state, fallback, bounded preview-device routing, and bounded preview-workflow queue or scheduling truth on one runtime-owned report seam instead of browser-local queue state or host-local preview playback policy.",
        },
        PreviewTransformBoundarySurface {
            id: "runtime-preview-transform-render-preview-snapshot",
            kind: PreviewTransformBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeClipRenderResult::preview_transform_snapshot and RuntimeOfflineRenderContractPreview::preview_transform_snapshot",
            runtime_anchor: "RuntimeClipRenderResult + RuntimeOfflineRenderContractPreview",
            rationale:
                "Proves clip render and offline preview carry the same runtime-owned preview-transform, preview-device, and preview-workflow posture instead of splitting preview truth across private render helpers or browser-side queue logic.",
        },
        PreviewTransformBoundarySurface {
            id: "shared-host-preview-transform-report",
            kind: PreviewTransformBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned preview-transform, preview-device, and preview-workflow receipts without browser-local queue or host-local audition scheduler reconstruction.",
        },
    ]
}

fn preview_transform_boundary_validation_steps() -> &'static [PreviewTransformBoundaryValidationStep]
{
    &[
        PreviewTransformBoundaryValidationStep {
            id: "runtime-preview-transform-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_preview_transform_boundary_reports_runtime_owned_preview_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned preview service class, readiness, degraded-state, fallback, bounded preview-device policy, and bounded preview-workflow queue or scheduling truth through public runtime surfaces alone.",
        },
        PreviewTransformBoundaryValidationStep {
            id: "local-host-preview-transform-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_preview_transform_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned preview-transform and preview-device receipts instead of rebuilding local preview playback or sink policy.",
        },
        PreviewTransformBoundaryValidationStep {
            id: "server-host-preview-transform-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_preview_transform_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned preview-transform and preview-device receipts without server-local preview heuristics or device-pick policy.",
        },
        PreviewTransformBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools preview_transform_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable preview-transform, preview-device, and preview-workflow boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only closure notes.",
        },
        PreviewTransformBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-preview-transform-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared preview-transform, preview-device, and preview-workflow proof seam without reading browser-local queue glue, host-local preview playback logic, or device-picker policy.",
        },
    ]
}

pub(crate) fn render_preview_transform_boundary_text() -> String {
    let mut rendered = format!(
        "preview_transform_boundary: {PREVIEW_TRANSFORM_BOUNDARY}\ncontract_path: {PREVIEW_TRANSFORM_CONTRACT_PATH}\nacceptance_task: {PREVIEW_TRANSFORM_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in preview_transform_boundary_surfaces() {
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
    for step in preview_transform_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned low-latency audition, scrub support, preview-transform readiness, degraded-state, fallback, bounded preview-device policy, and bounded preview-workflow queue or scheduling receipts through runtime, supervisor, render-preview, offline preview, and stable host-edge surfaces, but richer browser queue editing and deeper preview workflow depth still belong to later g08 work",
        "this closes the bounded preview-transform, preview-device, and preview-workflow consumer seam, not a full preview playback engine, browser shell, product-local audition workflow, or end-user device picker",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_preview_transform_boundary_json() -> String {
    let surfaces = preview_transform_boundary_surfaces()
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
    let validation_steps = preview_transform_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned low-latency audition, scrub support, preview-transform readiness, degraded-state, fallback, bounded preview-device policy, and bounded preview-workflow queue or scheduling receipts through runtime, supervisor, render-preview, offline preview, and stable host-edge surfaces, but richer browser queue editing and deeper preview workflow depth still belong to later g08 work",
        "this closes the bounded preview-transform, preview-device, and preview-workflow consumer seam, not a full preview playback engine, browser shell, product-local audition workflow, or end-user device picker",
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
        json_string(PREVIEW_TRANSFORM_BOUNDARY),
        json_string(PREVIEW_TRANSFORM_CONTRACT_PATH),
        json_string(PREVIEW_TRANSFORM_ACCEPTANCE_TASK),
        preview_transform_boundary_surfaces().len(),
        surfaces,
        preview_transform_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
