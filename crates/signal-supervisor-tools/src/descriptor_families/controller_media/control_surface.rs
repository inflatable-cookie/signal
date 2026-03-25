use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlSurfaceBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl ControlSurfaceBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ControlSurfaceBoundarySurface {
    id: &'static str,
    kind: ControlSurfaceBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ControlSurfaceBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

fn control_surface_boundary_surfaces() -> &'static [ControlSurfaceBoundarySurface] {
    &[
        ControlSurfaceBoundarySurface {
            id: "runtime-control-surface-report",
            kind: ControlSurfaceBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::control_surface_snapshot and RuntimeSupervisorReport::observation.control_surface_snapshot",
            runtime_anchor: "RuntimeControlSurfaceSnapshot",
            rationale:
                "Keeps control-surface graph state, transport posture, mapping posture, feedback readiness, and bounded capability on one runtime-owned report seam instead of host-local controller policy.",
        },
        ControlSurfaceBoundarySurface {
            id: "runtime-control-surface-external-midi-anchor",
            kind: ControlSurfaceBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_midi_snapshot and RuntimeSupervisorReport::observation.external_midi_snapshot",
            runtime_anchor: "RuntimeExternalMidiEndpointGraphSnapshot",
            rationale:
                "Keeps the control-surface baseline explicitly derived from the closed external MIDI endpoint graph instead of creating a second controller-device shell.",
        },
        ControlSurfaceBoundarySurface {
            id: "shared-host-control-surface-report",
            kind: ControlSurfaceBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward the same runtime-owned control-surface baseline without host-local controller-policy reconstruction.",
        },
    ]
}

fn control_surface_boundary_validation_steps() -> &'static [ControlSurfaceBoundaryValidationStep] {
    &[
        ControlSurfaceBoundaryValidationStep {
            id: "runtime-control-surface-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_control_surface_boundary_reports_runtime_owned_transport_and_feedback_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect control-surface graph state, transport posture, mapping posture, and feedback readiness through public runtime surfaces.",
        },
        ControlSurfaceBoundaryValidationStep {
            id: "local-host-control-surface-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_control_surface_truth",
            rationale:
                "Proves the stable local host edge forwards the runtime-owned control-surface baseline instead of rebuilding local controller policy.",
        },
        ControlSurfaceBoundaryValidationStep {
            id: "server-host-control-surface-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_control_surface_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned control-surface baseline instead of inventing server-local controller heuristics.",
        },
        ControlSurfaceBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools control_surface_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable control-surface boundary aligned with the focused runtime and host-edge proof spine instead of drifting into prose-only documentation.",
        },
        ControlSurfaceBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-control-surface-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared control-surface proof seam without reading host-local controller policy or implementation detail.",
        },
    ]
}

pub(crate) fn render_control_surface_boundary_text() -> String {
    let mut rendered = format!(
        "control_surface_boundary: {CONTROL_SURFACE_BOUNDARY}\ncontract_path: {CONTROL_SURFACE_CONTRACT_PATH}\nacceptance_task: {CONTROL_SURFACE_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in control_surface_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in control_surface_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves a bounded control-surface baseline, not fuller vendor protocol, display, motor, haptic, or scripting-safe extensibility depth",
        "the current seam keeps runtime-owned control-surface transport, mapping posture, feedback readiness, and guarded capability consumable through runtime and stable host edges, but richer feedback transport and product mapping workflows remain later work",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_control_surface_boundary_json() -> String {
    let surfaces = control_surface_boundary_surfaces()
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
                json_string(surface.rationale)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = control_surface_boundary_validation_steps()
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
                json_string(step.rationale)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred_scope = [
        "the shared boundary now proves a bounded control-surface baseline, not fuller vendor protocol, display, motor, haptic, or scripting-safe extensibility depth",
        "the current seam keeps runtime-owned control-surface transport, mapping posture, feedback readiness, and guarded capability consumable through runtime and stable host edges, but richer feedback transport and product mapping workflows remain later work",
    ].iter().map(|scope| json_string(scope)).collect::<Vec<_>>().join(",");
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
        json_string(CONTROL_SURFACE_BOUNDARY),
        json_string(CONTROL_SURFACE_CONTRACT_PATH),
        json_string(CONTROL_SURFACE_ACCEPTANCE_TASK),
        control_surface_boundary_surfaces().len(),
        surfaces,
        control_surface_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
