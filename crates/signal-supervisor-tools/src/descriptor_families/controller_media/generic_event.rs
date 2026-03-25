use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenericEventBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl GenericEventBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenericEventBoundarySurface {
    id: &'static str,
    kind: GenericEventBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenericEventBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

fn generic_event_boundary_surfaces() -> &'static [GenericEventBoundarySurface] {
    &[
        GenericEventBoundarySurface {
            id: "runtime-generic-event-report",
            kind: GenericEventBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot",
            runtime_anchor: "RuntimePluginEventSnapshot",
            rationale:
                "Keeps parameter, note, note-expression, and MIDI event continuity on one runtime-owned report seam instead of host-private payload counters.",
        },
        GenericEventBoundarySurface {
            id: "runtime-generic-event-capability-coverage",
            kind: GenericEventBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationApi::get_plugin_discovery_snapshot() capability_coverage.supports_note_expression_count",
            runtime_anchor: "RuntimePluginCapabilityCoverageSummary",
            rationale:
                "Keeps note-expression breadth on runtime-owned discovery receipts instead of adapter-local inference from MIDI or note support alone.",
        },
        GenericEventBoundarySurface {
            id: "shared-host-generic-event-supervisor-report",
            kind: GenericEventBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges expose the widened event and capability truth without CLAP, VST3, or AU packet reconstruction.",
        },
    ]
}

fn generic_event_boundary_validation_steps() -> &'static [GenericEventBoundaryValidationStep] {
    &[
        GenericEventBoundaryValidationStep {
            id: "runtime-generic-event-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect widened generic event and note-expression capability truth through public runtime reexports alone.",
        },
        GenericEventBoundaryValidationStep {
            id: "local-host-generic-event-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_generic_event_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned generic event and note-expression capability receipts on supervisor export.",
        },
        GenericEventBoundaryValidationStep {
            id: "server-host-generic-event-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_generic_event_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned generic event and note-expression capability receipts on supervisor export.",
        },
        GenericEventBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools generic_event_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable boundary descriptor aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        GenericEventBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-generic-event-boundary --format=json",
            rationale:
                "Lets consumers inspect the widened generic event proof boundary without reading private host code or adapter packet translation logic.",
        },
    ]
}

pub(crate) fn render_generic_event_boundary_text() -> String {
    let mut rendered = format!(
        "generic_event_boundary: {GENERIC_EVENT_BOUNDARY}\ncontract_path: {GENERIC_EVENT_CONTRACT_PATH}\nacceptance_task: {GENERIC_EVENT_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in generic_event_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in generic_event_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared generic event truth is now consumable, but richer per-format packet families, SysEx, controller mapping, and editor semantics remain later work",
        "the current boundary proves bounded event and note-expression capability receipts through runtime and stable host edges, not full CLAP, VST3, and AU packet-model parity",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_generic_event_boundary_json() -> String {
    let surfaces = generic_event_boundary_surfaces()
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
    let validation_steps = generic_event_boundary_validation_steps()
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
        "shared generic event truth is now consumable, but richer per-format packet families, SysEx, controller mapping, and editor semantics remain later work",
        "the current boundary proves bounded event and note-expression capability receipts through runtime and stable host edges, not full CLAP, VST3, and AU packet-model parity",
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
        json_string(GENERIC_EVENT_BOUNDARY),
        json_string(GENERIC_EVENT_CONTRACT_PATH),
        json_string(GENERIC_EVENT_ACCEPTANCE_TASK),
        generic_event_boundary_surfaces().len(),
        surfaces,
        generic_event_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
