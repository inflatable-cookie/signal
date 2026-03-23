use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lv2BoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Lv2BoundarySurface {
    id: &'static str,
    kind: Lv2BoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Lv2BoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl Lv2BoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn lv2_boundary_surfaces() -> &'static [Lv2BoundarySurface] {
    &[
        Lv2BoundarySurface {
            id: "runtime-lv2-extension-report",
            kind: Lv2BoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::lv2_extension_snapshot and RuntimeSupervisorReport::observation.lv2_extension_snapshot",
            runtime_anchor: "RuntimeLv2ExtensionSnapshot",
            rationale:
                "Keeps LV2 worker posture, URID negotiation posture, patch exchange posture, and extension-negotiation state consumable through shared runtime-owned reports instead of adapter-private feature tables.",
        },
        Lv2BoundarySurface {
            id: "runtime-lv2-lifecycle-snapshot",
            kind: Lv2BoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps LV2 extension posture derived from the existing runtime-owned sandbox lifecycle seam instead of a second host-local negotiation model.",
        },
        Lv2BoundarySurface {
            id: "local-host-lv2-supervisor-report",
            kind: Lv2BoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures the stable local host edge exports the same runtime-owned LV2 extension seam without inventing local-only worker, URID, or patch summaries.",
        },
        Lv2BoundarySurface {
            id: "server-host-lv2-supervisor-report",
            kind: Lv2BoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures the stable server host edge forwards LV2 extension truth without adapter-local reconstruction or host-private LV2 negotiation ledgers.",
        },
    ]
}

fn lv2_boundary_validation_steps() -> &'static [Lv2BoundaryValidationStep] {
    &[
        Lv2BoundaryValidationStep {
            id: "runtime-lv2-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned LV2 extension truth through public runtime reexports alone.",
        },
        Lv2BoundaryValidationStep {
            id: "local-host-lv2-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_lv2_extension_truth",
            rationale:
                "Proves the stable local host edge forwards the runtime-owned LV2 extension seam on supervisor export.",
        },
        Lv2BoundaryValidationStep {
            id: "server-host-lv2-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_lv2_extension_truth",
            rationale:
                "Proves the stable server host edge forwards runtime-owned LV2 extension state on supervisor export.",
        },
        Lv2BoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-lv2-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared LV2 proof boundary without reading host or adapter implementation detail.",
        },
    ]
}

pub(crate) fn render_lv2_boundary_text() -> String {
    let mut rendered = format!(
        "lv2_boundary: {LV2_BOUNDARY}\ncontract_path: {LV2_CONTRACT_PATH}\nacceptance_task: {LV2_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in lv2_boundary_surfaces() {
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
    for step in lv2_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared LV2 worker, URID, patch, and bounded extension-negotiation truth is now public, but full atom-schema, UI, custom extension, and worker execution depth still remain later Linux and cross-adapter work",
        "the current boundary proves bounded LV2 extension realization through runtime, supervisor export, and stable host edges, not broader Linux daemon policy or product-local workflow behavior",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_lv2_boundary_json() -> String {
    let surfaces = lv2_boundary_surfaces()
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
    let validation_steps = lv2_boundary_validation_steps()
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
        "shared LV2 worker, URID, patch, and bounded extension-negotiation truth is now public, but full atom-schema, UI, custom extension, and worker execution depth still remain later Linux and cross-adapter work",
        "the current boundary proves bounded LV2 extension realization through runtime, supervisor export, and stable host edges, not broader Linux daemon policy or product-local workflow behavior",
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
        json_string(LV2_BOUNDARY),
        json_string(LV2_CONTRACT_PATH),
        json_string(LV2_ACCEPTANCE_TASK),
        lv2_boundary_surfaces().len(),
        surfaces,
        lv2_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
