use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecallPortabilityBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl RecallPortabilityBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecallPortabilityBoundarySurface {
    id: &'static str,
    kind: RecallPortabilityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecallPortabilityBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

fn recall_portability_boundary_surfaces() -> &'static [RecallPortabilityBoundarySurface] {
    &[
        RecallPortabilityBoundarySurface {
            id: "runtime-plugin-chain-recall-report",
            kind: RecallPortabilityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_chain_snapshot and RuntimeSupervisorReport::observation.plugin_chain_snapshot",
            runtime_anchor: "RuntimePluginRecallPayload",
            rationale:
                "Keeps portable versus guarded, native-only, context-only, and unsupported recall truth on the shared plugin-chain report seam instead of adapter-native preset heuristics.",
        },
        RecallPortabilityBoundarySurface {
            id: "runtime-plugin-recall-handoff",
            kind: RecallPortabilityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_recall_handoff_snapshot()",
            runtime_anchor: "RuntimePluginRecallHandoffSnapshot",
            rationale:
                "Lets offline, export, and downstream consumers inspect widened preset descriptor and bounded ARA-context transfer on a runtime-owned handoff snapshot instead of host-local blob planning.",
        },
        RecallPortabilityBoundarySurface {
            id: "shared-host-recall-supervisor-report",
            kind: RecallPortabilityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward the same runtime-owned portability and ARA-context truth without adapter-local preset reconstruction or host-owned portability classes.",
        },
    ]
}

fn recall_portability_boundary_validation_steps(
) -> &'static [RecallPortabilityBoundaryValidationStep] {
    &[
        RecallPortabilityBoundaryValidationStep {
            id: "runtime-recall-portability-public-proof",
            command:
                "cargo test -p signal-runtime --test public_contract_boundary public_runtime_recall_interchange_and_ara_context_truth_is_consumable_from_reexports",
            rationale:
                "Proves a downstream-style runtime consumer can inspect portable versus non-portable recall outcomes and bounded ARA-context transfer through public runtime reexports alone.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "local-host-recall-portability-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_boundary local_shared_host_edge_exports_runtime_recall_portability_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned recall portability and ARA-context truth on supervisor export.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "server-host-recall-portability-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_boundary server_shared_host_edge_exports_runtime_recall_portability_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned recall portability and bounded ARA-context transfer on supervisor export.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools recall_portability_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable recall portability descriptor aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-recall-portability-boundary --format=json",
            rationale:
                "Lets consumers inspect portable versus native-only recall outcomes and bounded ARA-context transfer without reading private host glue or adapter-native preset parsing code.",
        },
    ]
}

pub(crate) fn render_recall_portability_boundary_text() -> String {
    let mut rendered = format!(
        "recall_portability_boundary: {RECALL_PORTABILITY_BOUNDARY}\ncontract_path: {RECALL_PORTABILITY_CONTRACT_PATH}\nacceptance_task: {RECALL_PORTABILITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in recall_portability_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in recall_portability_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared preset-state portability truth is now consumable, but lossless cross-adapter preset interchange, richer preset families, and adapter-native document models remain later work",
        "the current boundary proves bounded ARA document, source, and region context transfer through runtime and stable host edges, not fuller ARA editor workflow or persistent product document semantics",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_recall_portability_boundary_json() -> String {
    let surfaces = recall_portability_boundary_surfaces()
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
    let validation_steps = recall_portability_boundary_validation_steps()
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
        "shared preset-state portability truth is now consumable, but lossless cross-adapter preset interchange, richer preset families, and adapter-native document models remain later work",
        "the current boundary proves bounded ARA document, source, and region context transfer through runtime and stable host edges, not fuller ARA editor workflow or persistent product document semantics",
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
        json_string(RECALL_PORTABILITY_BOUNDARY),
        json_string(RECALL_PORTABILITY_CONTRACT_PATH),
        json_string(RECALL_PORTABILITY_ACCEPTANCE_TASK),
        recall_portability_boundary_surfaces().len(),
        surfaces,
        recall_portability_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
