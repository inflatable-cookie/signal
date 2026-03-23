use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JackCoordinationBoundarySurfaceKind {
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct JackCoordinationBoundarySurface {
    id: &'static str,
    kind: JackCoordinationBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct JackCoordinationBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl JackCoordinationBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn jack_coordination_boundary_surfaces() -> &'static [JackCoordinationBoundarySurface] {
    &[
        JackCoordinationBoundarySurface {
            id: "runtime-jack-coordination-report",
            kind: JackCoordinationBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::jack_coordination_snapshot and RuntimeSupervisorReport::observation.jack_coordination_snapshot",
            runtime_anchor: "RuntimeJackCoordinationSnapshot",
            rationale:
                "Keeps JACK transport posture, graph coordination, client role, and guarded state on one runtime-owned seam instead of host-private callback or daemon policy.",
        },
        JackCoordinationBoundarySurface {
            id: "runtime-transport-session-report",
            kind: JackCoordinationBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::transport_session_summary and RuntimeSupervisorReport::observation.transport_session_summary",
            runtime_anchor: "TransportSessionSummary",
            rationale:
                "Keeps the transport-session evidence feeding JACK coordination inspectable on the same public runtime boundary instead of forcing consumers into private transport internals.",
        },
        JackCoordinationBoundarySurface {
            id: "shared-host-jack-supervisor-report",
            kind: JackCoordinationBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward explicit JACK coordination answers without host-local transport or graph reclassification.",
        },
    ]
}

fn jack_coordination_boundary_validation_steps() -> &'static [JackCoordinationBoundaryValidationStep]
{
    &[
        JackCoordinationBoundaryValidationStep {
            id: "runtime-jack-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_jack_coordination_boundary_reports_runtime_owned_transport_graph_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect JACK transport, graph, client-role, and guarded coordination truth through public runtime surfaces alone.",
        },
        JackCoordinationBoundaryValidationStep {
            id: "local-host-jack-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_jack_coordination_truth",
            rationale:
                "Proves the stable local host edge exports an explicit `NotJack` answer instead of omitting JACK coordination on unsupported hosts.",
        },
        JackCoordinationBoundaryValidationStep {
            id: "server-host-jack-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_jack_coordination_truth",
            rationale:
                "Proves the stable server host edge forwards the bounded JACK graph and guarded transport baseline without server-local graph policy.",
        },
        JackCoordinationBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-jack-coordination-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared JACK coordination proof boundary without reading backend-private host code.",
        },
    ]
}

pub(crate) fn render_jack_coordination_boundary_text() -> String {
    let mut rendered = format!(
        "jack_coordination_boundary: {JACK_COORDINATION_BOUNDARY}\ncontract_path: {JACK_COORDINATION_CONTRACT_PATH}\nacceptance_task: {JACK_COORDINATION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in jack_coordination_boundary_surfaces() {
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
    for step in jack_coordination_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str(
        "residual_risk: the current boundary proves bounded JACK transport, graph, and guarded coordination truth, not real JACK daemon integration, session-manager depth, or callback-thread ownership\n",
    );
    rendered
}

pub(crate) fn render_jack_coordination_boundary_json() -> String {
    let surfaces = jack_coordination_boundary_surfaces()
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
    let validation_steps = jack_coordination_boundary_validation_steps()
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
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"surface_count\":{},",
            "\"validation_step_count\":{},",
            "\"surfaces\":[{}],",
            "\"validation_steps\":[{}],",
            "\"residual_risk\":{}",
            "}}"
        ),
        json_string(JACK_COORDINATION_BOUNDARY),
        json_string(JACK_COORDINATION_CONTRACT_PATH),
        json_string(JACK_COORDINATION_ACCEPTANCE_TASK),
        jack_coordination_boundary_surfaces().len(),
        jack_coordination_boundary_validation_steps().len(),
        surfaces,
        validation_steps,
        json_string(
            "the current boundary proves bounded JACK transport, graph, and guarded coordination truth, not real JACK daemon integration, session-manager depth, or callback-thread ownership",
        ),
    )
}
