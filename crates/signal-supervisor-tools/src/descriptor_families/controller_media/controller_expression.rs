use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControllerExpressionBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl ControllerExpressionBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ControllerExpressionBoundarySurface {
    id: &'static str,
    kind: ControllerExpressionBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ControllerExpressionBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

fn controller_expression_boundary_surfaces() -> &'static [ControllerExpressionBoundarySurface] {
    &[
        ControllerExpressionBoundarySurface {
            id: "runtime-controller-expression-report",
            kind: ControllerExpressionBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot",
            runtime_anchor: "RuntimePluginEventSnapshot",
            rationale:
                "Keeps widened note-expression family totals plus runtime-owned MPE and MIDI 2.0 posture on one shared report seam instead of adapter-private packet counters.",
        },
        ControllerExpressionBoundarySurface {
            id: "runtime-controller-expression-device-capability",
            kind: ControllerExpressionBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_midi_snapshot.endpoints[*].capability and RuntimeSupervisorReport::observation.external_midi_snapshot.endpoints[*].capability",
            runtime_anchor: "RuntimeExternalMidiEndpointCapabilitySummary",
            rationale:
                "Keeps widened external-device controller-expression capability posture runtime-owned instead of host-local or backend-private capability matrices.",
        },
        ControllerExpressionBoundarySurface {
            id: "shared-host-controller-expression-report",
            kind: ControllerExpressionBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward widened controller-expression posture through shared runtime reports without packet reconstruction.",
        },
    ]
}

fn controller_expression_boundary_validation_steps(
) -> &'static [ControllerExpressionBoundaryValidationStep] {
    &[
        ControllerExpressionBoundaryValidationStep {
            id: "runtime-controller-expression-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_controller_expression_boundary_reports_runtime_owned_expression_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect widened note-expression family totals plus device capability posture through public runtime surfaces.",
        },
        ControllerExpressionBoundaryValidationStep {
            id: "local-host-controller-expression-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_controller_expression_truth",
            rationale:
                "Proves the stable local host edge forwards widened controller-expression posture from runtime-owned reports instead of host-private packet decoding.",
        },
        ControllerExpressionBoundaryValidationStep {
            id: "server-host-controller-expression-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_controller_expression_truth",
            rationale:
                "Proves the stable server host edge forwards the same widened controller-expression posture instead of server-local capability heuristics.",
        },
        ControllerExpressionBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools controller_expression_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable controller-expression boundary aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        ControllerExpressionBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-controller-expression-boundary --format=json",
            rationale:
                "Lets consumers inspect the widened controller-expression proof seam without reading adapter-private packet or capability code.",
        },
    ]
}

pub(crate) fn render_controller_expression_boundary_text() -> String {
    let mut rendered = format!(
        "controller_expression_boundary: {CONTROLLER_EXPRESSION_BOUNDARY}\ncontract_path: {CONTROLLER_EXPRESSION_CONTRACT_PATH}\nacceptance_task: {CONTROLLER_EXPRESSION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in controller_expression_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in controller_expression_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves bounded richer controller-expression posture, not full MIDI 2.0 UMP transport, negotiation, or profile exchange depth",
        "the current seam keeps runtime-owned MPE and MIDI 2.0 posture consumable through runtime and stable host edges, but control-surface mapping and feedback semantics remain later work",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_controller_expression_boundary_json() -> String {
    let surfaces = controller_expression_boundary_surfaces()
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
    let validation_steps = controller_expression_boundary_validation_steps()
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
        "the shared boundary now proves bounded richer controller-expression posture, not full MIDI 2.0 UMP transport, negotiation, or profile exchange depth",
        "the current seam keeps runtime-owned MPE and MIDI 2.0 posture consumable through runtime and stable host edges, but control-surface mapping and feedback semantics remain later work",
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
        json_string(CONTROLLER_EXPRESSION_BOUNDARY),
        json_string(CONTROLLER_EXPRESSION_CONTRACT_PATH),
        json_string(CONTROLLER_EXPRESSION_ACCEPTANCE_TASK),
        controller_expression_boundary_surfaces().len(),
        surfaces,
        controller_expression_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
