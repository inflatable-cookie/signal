use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExternalMidiBoundarySurfaceKind {
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExternalMidiBoundarySurface {
    id: &'static str,
    kind: ExternalMidiBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExternalMidiBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl ExternalMidiBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn external_midi_boundary_surfaces() -> &'static [ExternalMidiBoundarySurface] {
    &[
        ExternalMidiBoundarySurface {
            id: "runtime-external-midi-report",
            kind: ExternalMidiBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_midi_snapshot and RuntimeSupervisorReport::observation.external_midi_snapshot",
            runtime_anchor:
                "RuntimeExternalMidiEndpointGraphSnapshot::{discovery_state, graph_state, live_ownership, provider_name, device_count, endpoint_count, active_route_count, guarded_route_count} + RuntimeExternalMidiLiveOwnershipSummary::{ownership_posture, attach_continuity, backend_parity, guarded_parity_outcome}",
            rationale:
                "Keeps external MIDI discovery, graph, endpoint, route, live ownership, and backend parity truth on one runtime-owned report seam instead of host-local MIDI device reconstruction or backend-local endpoint policy.",
        },
        ExternalMidiBoundarySurface {
            id: "shared-host-external-midi-report",
            kind: ExternalMidiBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward the same runtime-owned external MIDI graph, live ownership, and backend parity receipts instead of inventing host-private device tables or route heuristics.",
        },
    ]
}

fn external_midi_boundary_validation_steps() -> &'static [ExternalMidiBoundaryValidationStep] {
    &[
        ExternalMidiBoundaryValidationStep {
            id: "runtime-external-midi-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_external_midi_boundary_reports_runtime_owned_endpoint_graph_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned external MIDI graph, live ownership, and backend parity truth through public runtime surfaces alone.",
        },
        ExternalMidiBoundaryValidationStep {
            id: "local-host-external-midi-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_external_midi_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned external MIDI graph and live ownership receipts instead of rebuilding local MIDI device truth.",
        },
        ExternalMidiBoundaryValidationStep {
            id: "server-host-external-midi-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_external_midi_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned external MIDI graph and backend parity receipts instead of inventing server-local device reconstruction.",
        },
        ExternalMidiBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools external_midi_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable external MIDI, live ownership, and backend parity boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        ExternalMidiBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-external-midi-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared external MIDI graph, live ownership, and backend parity proof seam without reading host-private MIDI integration code.",
        },
    ]
}

pub(crate) fn render_external_midi_boundary_text() -> String {
    let mut rendered = format!(
        "external_midi_boundary: {EXTERNAL_MIDI_BOUNDARY}\ncontract_path: {EXTERNAL_MIDI_CONTRACT_PATH}\nacceptance_task: {EXTERNAL_MIDI_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in external_midi_boundary_surfaces() {
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
    for step in external_midi_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared external MIDI graph, live ownership, and backend parity truth are now consumable, but richer session-manager, reservation, and attach-policy depth remain later work",
        "the current boundary proves bounded unavailable and empty ownership baselines through runtime and stable host edges, not fuller MIDI 2.0, MPE, or control-surface transport depth",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_external_midi_boundary_json() -> String {
    let surfaces = external_midi_boundary_surfaces()
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
    let validation_steps = external_midi_boundary_validation_steps()
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
        "shared external MIDI graph, live ownership, and backend parity truth are now consumable, but richer session-manager, reservation, and attach-policy depth remain later work",
        "the current boundary proves bounded unavailable and empty ownership baselines through runtime and stable host edges, not fuller MIDI 2.0, MPE, or control-surface transport depth",
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
            "\"validation_step_count\":{},",
            "\"surfaces\":[{}],",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(EXTERNAL_MIDI_BOUNDARY),
        json_string(EXTERNAL_MIDI_CONTRACT_PATH),
        json_string(EXTERNAL_MIDI_ACCEPTANCE_TASK),
        external_midi_boundary_surfaces().len(),
        external_midi_boundary_validation_steps().len(),
        surfaces,
        validation_steps,
        deferred_scope,
    )
}
