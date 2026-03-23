use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Vst3BoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Vst3BoundarySurface {
    id: &'static str,
    kind: Vst3BoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Vst3BoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl Vst3BoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn vst3_boundary_surfaces() -> &'static [Vst3BoundarySurface] {
    &[
        Vst3BoundarySurface {
            id: "runtime-vst3-discovery-report",
            kind: Vst3BoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor: "RuntimePluginDiscoverySnapshot",
            rationale:
                "Keeps discovered VST3 types and format-filtered scan intent consumable through shared runtime reports rather than adapter-private catalogs.",
        },
        Vst3BoundarySurface {
            id: "runtime-vst3-lifecycle-snapshot",
            kind: Vst3BoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps VST3 sandbox lifecycle, readiness, and transport attachment truth on the existing runtime-owned lifecycle seam instead of a format-specific lifecycle shell.",
        },
        Vst3BoundarySurface {
            id: "shared-host-vst3-supervisor-report",
            kind: Vst3BoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward VST3 discovery and lifecycle truth without adapter-local reconstruction or host-private VST3 ledgers.",
        },
    ]
}

fn vst3_boundary_validation_steps() -> &'static [Vst3BoundaryValidationStep] {
    &[
        Vst3BoundaryValidationStep {
            id: "runtime-vst3-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect VST3 discovery and lifecycle truth through public runtime reexports alone.",
        },
        Vst3BoundaryValidationStep {
            id: "local-host-vst3-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_vst3_baseline_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned VST3 discovery and lifecycle state on supervisor export.",
        },
        Vst3BoundaryValidationStep {
            id: "server-host-vst3-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_vst3_baseline_truth",
            rationale:
                "Proves the server stable host edge forwards Linux-rooted runtime-owned VST3 discovery and lifecycle state on supervisor export.",
        },
        Vst3BoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-vst3-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared VST3 proof boundary without reading host or adapter implementation detail.",
        },
    ]
}

pub(crate) fn render_vst3_boundary_text() -> String {
    let mut rendered = format!(
        "vst3_boundary: {VST3_BOUNDARY}\ncontract_path: {VST3_CONTRACT_PATH}\nacceptance_task: {VST3_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in vst3_boundary_surfaces() {
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
    for step in vst3_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared VST3 discovery and lifecycle truth is now public, but richer event, unit, and program-list depth still remain later cross-adapter work",
        "the current boundary proves adapter realization through runtime and stable host edges, not wider cross-format parity or publication-grade plugin breadth",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_vst3_boundary_json() -> String {
    let surfaces = vst3_boundary_surfaces()
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
    let validation_steps = vst3_boundary_validation_steps()
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
        "shared VST3 discovery and lifecycle truth is now public, but richer event, unit, and program-list depth still remain later cross-adapter work",
        "the current boundary proves adapter realization through runtime and stable host edges, not wider cross-format parity or publication-grade plugin breadth",
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
        json_string(VST3_BOUNDARY),
        json_string(VST3_CONTRACT_PATH),
        json_string(VST3_ACCEPTANCE_TASK),
        vst3_boundary_surfaces().len(),
        surfaces,
        vst3_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
