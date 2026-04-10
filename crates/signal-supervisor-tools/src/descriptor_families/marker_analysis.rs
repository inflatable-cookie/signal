use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MarkerAnalysisBoundarySurfaceKind {
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MarkerAnalysisBoundarySurface {
    id: &'static str,
    kind: MarkerAnalysisBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MarkerAnalysisBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl MarkerAnalysisBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn marker_analysis_boundary_surfaces() -> &'static [MarkerAnalysisBoundarySurface] {
    &[
        MarkerAnalysisBoundarySurface {
            id: "runtime-marker-analysis-observation-report",
            kind: MarkerAnalysisBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::marker_analysis_snapshot and RuntimeSupervisorReport::observation.marker_analysis_snapshot",
            runtime_anchor: "RuntimeMarkerAnalysisSnapshot",
            rationale:
                "Keeps warp-marker, transient-anchor, tempo-assist, readiness, and invalidation truth on one runtime-owned report seam instead of host-local stretch-analysis reconstruction.",
        },
        MarkerAnalysisBoundarySurface {
            id: "shared-host-marker-analysis-report",
            kind: MarkerAnalysisBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned marker-analysis receipts without host-local marker heuristics or transform-analysis policy.",
        },
    ]
}

fn marker_analysis_boundary_validation_steps() -> &'static [MarkerAnalysisBoundaryValidationStep] {
    &[
        MarkerAnalysisBoundaryValidationStep {
            id: "runtime-marker-analysis-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_marker_analysis_boundary_reports_runtime_owned_analysis_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned warp-marker, transient-anchor, tempo-assist, readiness, and invalidation truth through public runtime surfaces alone.",
        },
        MarkerAnalysisBoundaryValidationStep {
            id: "local-host-marker-analysis-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_marker_analysis local_shared_host_edge_exports_runtime_marker_analysis_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge forwards runtime-owned marker-analysis receipts instead of rebuilding local stretch-analysis posture.",
        },
        MarkerAnalysisBoundaryValidationStep {
            id: "server-host-marker-analysis-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_marker_analysis server_shared_host_edge_exports_runtime_marker_analysis_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned marker-analysis receipts without server-local transform-analysis reconstruction.",
        },
        MarkerAnalysisBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools marker_analysis_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable marker-analysis boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only closure notes.",
        },
        MarkerAnalysisBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-marker-analysis-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared marker-analysis proof seam without reading host-local stretch-analysis glue.",
        },
    ]
}

pub(crate) fn render_marker_analysis_boundary_text() -> String {
    let mut rendered = format!(
        "marker_analysis_boundary: {MARKER_ANALYSIS_BOUNDARY}\ncontract_path: {MARKER_ANALYSIS_CONTRACT_PATH}\nacceptance_task: {MARKER_ANALYSIS_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in marker_analysis_boundary_surfaces() {
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
    for step in marker_analysis_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned warp-marker, transient-anchor, tempo-assist, readiness, and invalidation receipts through runtime, supervisor, and stable host-edge surfaces, but fuller editor-grade marker tooling, beat-grid authoring, artifact-cache depth, and low-latency audition still belongs to later g07 work",
        "this closes the bounded marker-analysis consumer seam, not a richer transform-analysis engine, arrangement intelligence layer, or product-local editing workflow",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_marker_analysis_boundary_json() -> String {
    let surfaces = marker_analysis_boundary_surfaces()
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
    let validation_steps = marker_analysis_boundary_validation_steps()
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
        "the shared boundary now proves runtime-owned warp-marker, transient-anchor, tempo-assist, readiness, and invalidation receipts through runtime, supervisor, and stable host-edge surfaces, but fuller editor-grade marker tooling, beat-grid authoring, artifact-cache depth, and low-latency audition still belongs to later g07 work",
        "this closes the bounded marker-analysis consumer seam, not a richer transform-analysis engine, arrangement intelligence layer, or product-local editing workflow",
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
        json_string(MARKER_ANALYSIS_BOUNDARY),
        json_string(MARKER_ANALYSIS_CONTRACT_PATH),
        json_string(MARKER_ANALYSIS_ACCEPTANCE_TASK),
        marker_analysis_boundary_surfaces().len(),
        surfaces,
        marker_analysis_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
