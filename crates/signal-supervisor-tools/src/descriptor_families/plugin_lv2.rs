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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxLv2ExecutionBoundarySurfaceKind {
    RuntimeReport,
    RuntimeLifecycle,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxLv2ExecutionBoundarySurface {
    id: &'static str,
    kind: LinuxLv2ExecutionBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxLv2ExecutionBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl LinuxLv2ExecutionBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeLifecycle => "runtime-lifecycle",
            Self::HostEdge => "host-edge",
        }
    }
}

fn linux_lv2_execution_boundary_surfaces() -> &'static [LinuxLv2ExecutionBoundarySurface] {
    &[
        LinuxLv2ExecutionBoundarySurface {
            id: "runtime-lv2-discovery-report",
            kind: LinuxLv2ExecutionBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor: "RuntimePluginDiscoverySnapshot",
            rationale:
                "Keeps real LV2 bundle and manifest traversal visible through the shared runtime discovery report instead of a Linux-only server test fixture contract.",
        },
        LinuxLv2ExecutionBoundarySurface {
            id: "runtime-lv2-broker-lifecycle-report",
            kind: LinuxLv2ExecutionBoundarySurfaceKind::RuntimeLifecycle,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeSupervisorReport::observation.plugin_lifecycle_snapshot",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps prepared negotiation, broker transport attachment, and bounded LV2 execution stream truth on the runtime-owned lifecycle seam instead of a broker-local debug log.",
        },
        LinuxLv2ExecutionBoundarySurface {
            id: "server-host-lv2-broker-supervisor-report",
            kind: LinuxLv2ExecutionBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Proves the Linux-facing stable server host edge exports real LV2 discovery plus broker-backed execution stream truth from one runtime-owned supervisor surface.",
        },
    ]
}

fn linux_lv2_execution_boundary_validation_steps(
) -> &'static [LinuxLv2ExecutionBoundaryValidationStep] {
    &[
        LinuxLv2ExecutionBoundaryValidationStep {
            id: "runtime-lv2-discovery-and-lifecycle-proof",
            command:
                "cargo test -p signal-runtime public_runtime_lv2_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
            rationale:
                "Proves the public runtime boundary still exposes real LV2 discovery and lifecycle truth after the Linux broker-backed execution lane deepened.",
        },
        LinuxLv2ExecutionBoundaryValidationStep {
            id: "server-host-lv2-broker-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_route_lv2_sandbox_through_broker_process -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable Linux-facing server host edge exports prepared negotiation plus bounded broker-backed LV2 execution stream truth on the healthy path.",
        },
        LinuxLv2ExecutionBoundaryValidationStep {
            id: "server-host-lv2-broker-recovery-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_sandbox_broker server_public_host_edge_can_drive_broker_backed_lv2_crash_recovery -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the same Linux-facing server host edge preserves LV2 execution stream truth through one recovery-owned broker path instead of collapsing back to generic attach markers.",
        },
        LinuxLv2ExecutionBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-linux-lv2-execution-boundary --format=json",
            rationale:
                "Lets consumers inspect the focused Linux LV2 discovery-plus-broker-execution acceptance surface without reading broker or host implementation detail.",
        },
    ]
}

pub(crate) fn render_linux_lv2_execution_boundary_text() -> String {
    let mut rendered = format!(
        "linux_lv2_execution_boundary: {LINUX_LV2_EXECUTION_BOUNDARY}\ncontract_path: {LINUX_LV2_EXECUTION_CONTRACT_PATH}\nacceptance_task: {LINUX_LV2_EXECUTION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in linux_lv2_execution_boundary_surfaces() {
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
    for step in linux_lv2_execution_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "this boundary proves one honest Linux-native LV2 lane through real discovery, bounded negotiation, and broker-backed execution stream truth, but it does not yet claim LV2 UI, atom-schema breadth, or distro-wide backend certification",
        "the stable server host is the authority for this Linux LV2 proof; local-host parity and later interactive demo coverage remain separate milestones",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_linux_lv2_execution_boundary_json() -> String {
    let surfaces = linux_lv2_execution_boundary_surfaces()
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
    let validation_steps = linux_lv2_execution_boundary_validation_steps()
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
        "this boundary proves one honest Linux-native LV2 lane through real discovery, bounded negotiation, and broker-backed execution stream truth, but it does not yet claim LV2 UI, atom-schema breadth, or distro-wide backend certification",
        "the stable server host is the authority for this Linux LV2 proof; local-host parity and later interactive demo coverage remain separate milestones",
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
        json_string(LINUX_LV2_EXECUTION_BOUNDARY),
        json_string(LINUX_LV2_EXECUTION_CONTRACT_PATH),
        json_string(LINUX_LV2_EXECUTION_ACCEPTANCE_TASK),
        linux_lv2_execution_boundary_surfaces().len(),
        surfaces,
        linux_lv2_execution_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
