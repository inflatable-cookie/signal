use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxBackendClockTopologyBoundarySurfaceKind {
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxBackendClockTopologyBoundarySurface {
    id: &'static str,
    kind: LinuxBackendClockTopologyBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxBackendClockTopologyBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl LinuxBackendClockTopologyBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn linux_backend_clock_topology_boundary_surfaces(
) -> &'static [LinuxBackendClockTopologyBoundarySurface] {
    &[
        LinuxBackendClockTopologyBoundarySurface {
            id: "runtime-linux-backend-clock-topology-report",
            kind: LinuxBackendClockTopologyBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
            runtime_anchor: "RuntimeExternalIoSnapshot + RuntimeHostClockingSummary",
            rationale:
                "Keeps Linux-specific clocking, duplex, and endpoint-topology parity on one runtime-owned external-I/O seam instead of backend-private Linux capability matrices.",
        },
        LinuxBackendClockTopologyBoundarySurface {
            id: "local-host-linux-backend-clock-topology-report",
            kind: LinuxBackendClockTopologyBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local",
            surface: "LocalRuntimeHost::host_supervisor_report() -> RuntimeHostSupervisorReport",
            runtime_anchor: "RuntimeHostSupervisorReport",
            rationale:
                "Proves the stable local host edge forwards explicit unsupported Linux parity on non-Linux hardware instead of leaving the gap implicit.",
        },
        LinuxBackendClockTopologyBoundarySurface {
            id: "server-host-linux-backend-clock-topology-report",
            kind: LinuxBackendClockTopologyBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Proves the stable server host edge forwards explicit unavailable Linux backend clocking and topology parity instead of host-local Linux heuristics.",
        },
    ]
}

fn linux_backend_clock_topology_boundary_validation_steps(
) -> &'static [LinuxBackendClockTopologyBoundaryValidationStep] {
    &[
        LinuxBackendClockTopologyBoundaryValidationStep {
            id: "runtime-linux-backend-clock-topology-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_linux_backend_clock_topology_boundary_reports_runtime_owned_linux_parity_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect Linux-specific clocking, duplex, and endpoint-topology parity truth through public runtime surfaces alone.",
        },
        LinuxBackendClockTopologyBoundaryValidationStep {
            id: "local-host-linux-backend-clock-topology-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_linux_backend_clock_topology_truth",
            rationale:
                "Proves the stable local host edge forwards explicit unsupported Linux parity on non-Linux hardware instead of a missing-field gap.",
        },
        LinuxBackendClockTopologyBoundaryValidationStep {
            id: "server-host-linux-backend-clock-topology-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_linux_backend_clock_topology_truth",
            rationale:
                "Proves the stable server host edge forwards explicit unavailable Linux backend clocking and topology parity on supervisor export.",
        },
        LinuxBackendClockTopologyBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools linux_backend_clock_topology_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable Linux backend clock-topology boundary aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        LinuxBackendClockTopologyBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-linux-backend-clock-topology-boundary --format=json",
            rationale:
                "Lets consumers inspect the widened Linux backend clocking and topology proof seam without reading backend-private Linux host code.",
        },
    ]
}

pub(crate) fn render_linux_backend_clock_topology_boundary_text() -> String {
    let mut rendered = format!(
        "linux_backend_clock_topology_boundary: {LINUX_BACKEND_CLOCK_TOPOLOGY_BOUNDARY}\ncontract_path: {LINUX_BACKEND_CLOCK_TOPOLOGY_CONTRACT_PATH}\nacceptance_task: {LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in linux_backend_clock_topology_boundary_surfaces() {
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
    for step in linux_backend_clock_topology_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered
}

pub(crate) fn render_linux_backend_clock_topology_boundary_json() -> String {
    let surfaces = linux_backend_clock_topology_boundary_surfaces()
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
    let validation_steps = linux_backend_clock_topology_boundary_validation_steps()
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
            "\"validation_steps\":[{}]",
            "}}"
        ),
        json_string(LINUX_BACKEND_CLOCK_TOPOLOGY_BOUNDARY),
        json_string(LINUX_BACKEND_CLOCK_TOPOLOGY_CONTRACT_PATH),
        json_string(LINUX_BACKEND_CLOCK_TOPOLOGY_ACCEPTANCE_TASK),
        linux_backend_clock_topology_boundary_surfaces().len(),
        linux_backend_clock_topology_boundary_validation_steps().len(),
        surfaces,
        validation_steps,
    )
}
