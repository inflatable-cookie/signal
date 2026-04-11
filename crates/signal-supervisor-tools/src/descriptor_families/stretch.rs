use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StretchBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StretchBoundarySurface {
    id: &'static str,
    kind: StretchBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct StretchBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl StretchBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn stretch_boundary_surfaces() -> &'static [StretchBoundarySurface] {
    &[
        StretchBoundarySurface {
            id: "runtime-stretch-observation-report",
            kind: StretchBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::stretch_engine_snapshot and RuntimeSupervisorReport::observation.stretch_engine_snapshot",
            runtime_anchor: "RuntimeStretchEngineSnapshot",
            rationale:
                "Keeps sample-domain stretch engine class, readiness, degraded-state, and fallback truth on one runtime-owned report seam instead of host-local transform reconstruction.",
        },
        StretchBoundarySurface {
            id: "runtime-stretch-render-preview-snapshot",
            kind: StretchBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeClipRenderResult::stretch_engine_snapshot and RuntimeOfflineRenderContractPreview::stretch_engine_snapshot",
            runtime_anchor: "RuntimeStretchClipSnapshot + RuntimeStretchEngineSnapshot",
            rationale:
                "Lets downstream consumers inspect the same runtime-owned stretch truth on clip render and offline preview surfaces instead of rebuilding transform posture per export path.",
        },
        StretchBoundarySurface {
            id: "shared-host-stretch-report",
            kind: StretchBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned stretch receipts without host-local transform or preview-policy reconstruction.",
        },
    ]
}

fn stretch_boundary_validation_steps() -> &'static [StretchBoundaryValidationStep] {
    &[
        StretchBoundaryValidationStep {
            id: "runtime-stretch-public-proof",
            command:
                "cargo test -p signal-runtime --test public_contract_boundary_stretch public_runtime_stretch_boundary_reports_runtime_owned_engine_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves a downstream-style runtime consumer can inspect stretch engine class, readiness, degraded-state, fallback, clip render, and offline preview truth through public runtime surfaces alone.",
        },
        StretchBoundaryValidationStep {
            id: "local-host-stretch-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_stretch local_shared_host_edge_exports_runtime_stretch_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge forwards runtime-owned stretch receipts on supervisor export instead of rebuilding local transform posture.",
        },
        StretchBoundaryValidationStep {
            id: "server-host-stretch-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_stretch server_shared_host_edge_exports_runtime_stretch_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned stretch receipts without server-local transform reconstruction.",
        },
        StretchBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools stretch_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable stretch boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        StretchBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-stretch-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared stretch proof seam without reading host-local preview or render transform glue.",
        },
    ]
}

pub(crate) fn render_stretch_boundary_text() -> String {
    let mut rendered = format!(
        "stretch_boundary: {STRETCH_BOUNDARY}\ncontract_path: {STRETCH_CONTRACT_PATH}\nacceptance_task: {STRETCH_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in stretch_boundary_surfaces() {
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
    for step in stretch_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned sample-domain stretch receipts through runtime, supervisor, render preview, and stable host-edge surfaces, but marker-analysis, artifact-cache, and low-latency audition depth still belongs to later g07 work",
        "this closes the bounded stretch-engine consumer seam, not broader algorithm-support breadth, warp-marker editing workflows, or product-local preview transforms",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_stretch_boundary_json() -> String {
    let surfaces = stretch_boundary_surfaces()
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
    let validation_steps = stretch_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned sample-domain stretch receipts through runtime, supervisor, render preview, and stable host-edge surfaces, but marker-analysis, artifact-cache, and low-latency audition depth still belongs to later g07 work",
        "this closes the bounded stretch-engine consumer seam, not broader algorithm-support breadth, warp-marker editing workflows, or product-local preview transforms",
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
        json_string(STRETCH_BOUNDARY),
        json_string(STRETCH_CONTRACT_PATH),
        json_string(STRETCH_ACCEPTANCE_TASK),
        stretch_boundary_surfaces().len(),
        surfaces,
        stretch_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
