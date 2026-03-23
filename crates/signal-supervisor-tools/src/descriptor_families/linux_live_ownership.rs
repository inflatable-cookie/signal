use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxLiveOwnershipBoundarySurfaceKind {
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxLiveOwnershipBoundarySurface {
    id: &'static str,
    kind: LinuxLiveOwnershipBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxLiveOwnershipBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl LinuxLiveOwnershipBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn linux_live_ownership_boundary_surfaces() -> &'static [LinuxLiveOwnershipBoundarySurface] {
    &[
        LinuxLiveOwnershipBoundarySurface {
            id: "runtime-linux-live-session-report",
            kind: LinuxLiveOwnershipBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::linux_backend_session_snapshot and RuntimeSupervisorReport::observation.linux_backend_session_snapshot",
            runtime_anchor: "RuntimeLinuxBackendSessionSnapshot",
            rationale:
                "Keeps live ALSA, JACK, and PipeWire session ownership, lifecycle, device-claim, role, and guarded fallback truth on one runtime-owned seam instead of host-private Linux session state machines.",
        },
        LinuxLiveOwnershipBoundarySurface {
            id: "local-host-linux-live-session-report",
            kind: LinuxLiveOwnershipBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local",
            surface: "LocalRuntimeHost::host_supervisor_report() -> RuntimeHostSupervisorReport",
            runtime_anchor: "RuntimeHostSupervisorReport",
            rationale:
                "Proves the stable local host edge forwards an explicit non-Linux answer instead of omitting the live-session seam on unsupported hosts.",
        },
        LinuxLiveOwnershipBoundarySurface {
            id: "server-host-linux-live-session-report",
            kind: LinuxLiveOwnershipBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Proves the stable server host edge forwards runtime-owned PipeWire-style live-session truth instead of inventing backend-private Linux ownership policy.",
        },
    ]
}

fn linux_live_ownership_boundary_validation_steps(
) -> &'static [LinuxLiveOwnershipBoundaryValidationStep] {
    &[
        LinuxLiveOwnershipBoundaryValidationStep {
            id: "runtime-linux-live-session-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_linux_live_ownership_boundary_reports_runtime_owned_session_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect ALSA, JACK, and PipeWire live-session ownership and lifecycle truth through public runtime surfaces alone.",
        },
        LinuxLiveOwnershipBoundaryValidationStep {
            id: "local-host-linux-live-session-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_linux_live_ownership_truth",
            rationale:
                "Proves the stable local host edge forwards an explicit runtime-owned non-Linux live-session answer instead of leaving the seam absent.",
        },
        LinuxLiveOwnershipBoundaryValidationStep {
            id: "server-host-linux-live-session-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_linux_live_ownership_truth",
            rationale:
                "Proves the stable server host edge forwards runtime-owned live-session ownership and device-claim truth instead of host-local Linux session matrices.",
        },
        LinuxLiveOwnershipBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-linux-live-ownership-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared live Linux ownership proof boundary without reading backend-private Linux host code.",
        },
    ]
}

pub(crate) fn render_linux_live_ownership_boundary_text() -> String {
    let mut rendered = format!(
        "linux_live_ownership_boundary: {LINUX_LIVE_OWNERSHIP_BOUNDARY}\ncontract_path: {LINUX_LIVE_OWNERSHIP_CONTRACT_PATH}\nacceptance_task: {LINUX_LIVE_OWNERSHIP_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in linux_live_ownership_boundary_surfaces() {
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
    for step in linux_live_ownership_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str(
        "residual_risk: the current boundary proves bounded live-session ownership, lifecycle, and device-claim truth, not real ALSA/JACK/PipeWire daemon coordination, transport, or recovery depth\n",
    );
    rendered
}

pub(crate) fn render_linux_live_ownership_boundary_json() -> String {
    let surfaces = linux_live_ownership_boundary_surfaces()
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
    let validation_steps = linux_live_ownership_boundary_validation_steps()
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
            "\"validation_steps\":[{}],",
            "\"residual_risk\":{}",
            "}}"
        ),
        json_string(LINUX_LIVE_OWNERSHIP_BOUNDARY),
        json_string(LINUX_LIVE_OWNERSHIP_CONTRACT_PATH),
        json_string(LINUX_LIVE_OWNERSHIP_ACCEPTANCE_TASK),
        linux_live_ownership_boundary_surfaces().len(),
        linux_live_ownership_boundary_validation_steps().len(),
        surfaces,
        validation_steps,
        json_string(
            "the current boundary proves bounded live-session ownership, lifecycle, and device-claim truth, not real ALSA/JACK/PipeWire daemon coordination, transport, or recovery depth",
        ),
    )
}
