use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrossAdapterParityBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CrossAdapterParityBoundarySurface {
    id: &'static str,
    kind: CrossAdapterParityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CrossAdapterParityBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl CrossAdapterParityBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn cross_adapter_parity_boundary_surfaces() -> &'static [CrossAdapterParityBoundarySurface] {
    &[
        CrossAdapterParityBoundarySurface {
            id: "runtime-cross-adapter-discovery-report",
            kind: CrossAdapterParityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor: "RuntimePluginDiscoverySnapshot",
            rationale:
                "Keeps CLAP, VST3, and AU parity bands plus supported and unsupported platform scope consumable through the shared discovery report seam instead of a host-local portability matrix.",
        },
        CrossAdapterParityBoundarySurface {
            id: "runtime-cross-adapter-lifecycle-snapshot",
            kind: CrossAdapterParityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps cross-adapter parity counts for ready, degraded, faulted, and active-transport sandbox state on the existing runtime-owned lifecycle seam.",
        },
        CrossAdapterParityBoundarySurface {
            id: "shared-host-cross-adapter-supervisor-report",
            kind: CrossAdapterParityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward one cross-adapter parity vocabulary without private host portability tables or adapter-specific reconstruction.",
        },
    ]
}

fn cross_adapter_parity_boundary_validation_steps(
) -> &'static [CrossAdapterParityBoundaryValidationStep] {
    &[
        CrossAdapterParityBoundaryValidationStep {
            id: "runtime-cross-adapter-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect CLAP, VST3, and AU parity coverage through public runtime reexports alone.",
        },
        CrossAdapterParityBoundaryValidationStep {
            id: "local-host-cross-adapter-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_cross_adapter_parity_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned cross-adapter parity coverage on supervisor export.",
        },
        CrossAdapterParityBoundaryValidationStep {
            id: "server-host-cross-adapter-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_cross_adapter_parity_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned cross-adapter parity coverage on supervisor export.",
        },
        CrossAdapterParityBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-cross-adapter-parity-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared cross-adapter parity proof boundary without reading private host code or adapter internals.",
        },
    ]
}

pub(crate) fn render_cross_adapter_parity_boundary_text() -> String {
    let mut rendered = format!(
        "cross_adapter_parity_boundary: {CROSS_ADAPTER_PARITY_BOUNDARY}\ncontract_path: {CROSS_ADAPTER_PARITY_CONTRACT_PATH}\nacceptance_task: {CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in cross_adapter_parity_boundary_surfaces() {
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
    for step in cross_adapter_parity_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared CLAP, VST3, and AU parity truth is now public, but richer event-model, preset, editor, and unit-tree parity still remain later cross-adapter work",
        "the current boundary proves bounded platform coverage and lifecycle parity through runtime and stable host edges, not publication-grade capability marketing or deeper adapter internals",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_cross_adapter_parity_boundary_json() -> String {
    let surfaces = cross_adapter_parity_boundary_surfaces()
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
    let validation_steps = cross_adapter_parity_boundary_validation_steps()
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
        "shared CLAP, VST3, and AU parity truth is now public, but richer event-model, preset, editor, and unit-tree parity still remain later cross-adapter work",
        "the current boundary proves bounded platform coverage and lifecycle parity through runtime and stable host edges, not publication-grade capability marketing or deeper adapter internals",
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
        json_string(CROSS_ADAPTER_PARITY_BOUNDARY),
        json_string(CROSS_ADAPTER_PARITY_CONTRACT_PATH),
        json_string(CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK),
        cross_adapter_parity_boundary_surfaces().len(),
        surfaces,
        cross_adapter_parity_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
