use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeviceSupervisionBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl DeviceSupervisionBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceSupervisionBoundarySurface {
    id: &'static str,
    kind: DeviceSupervisionBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceSupervisionBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

fn device_supervision_boundary_surfaces() -> &'static [DeviceSupervisionBoundarySurface] {
    &[
        DeviceSupervisionBoundarySurface {
            id: "runtime-device-supervision-report",
            kind: DeviceSupervisionBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::device_supervision_snapshot and RuntimeSupervisorReport::observation.device_supervision_snapshot",
            runtime_anchor: "RuntimeDeviceSupervisionSnapshot",
            rationale:
                "Keeps restart-state, exhaustion, and fault-boundary meaning on a shared runtime-owned report seam instead of host-private restart heuristics.",
        },
        DeviceSupervisionBoundarySurface {
            id: "runtime-supervision-fault-alignment",
            kind: DeviceSupervisionBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::fault_status and RuntimeObservationReport::interruption_summary",
            runtime_anchor: "RuntimeFaultStatusSnapshot + RuntimeInterruptionSummary",
            rationale:
                "Keeps device supervision classification aligned with shared runtime fault and interruption truth instead of a second hardware-only taxonomy.",
        },
        DeviceSupervisionBoundarySurface {
            id: "shared-host-device-supervision-supervisor-report",
            kind: DeviceSupervisionBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned device supervision truth without private restart-loop reconstruction or host-local fault classes.",
        },
    ]
}

fn device_supervision_boundary_validation_steps(
) -> &'static [DeviceSupervisionBoundaryValidationStep] {
    &[
        DeviceSupervisionBoundaryValidationStep {
            id: "runtime-device-supervision-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states",
            rationale:
                "Proves a downstream-style runtime consumer can inspect recovering and explicit faulted device supervision truth through public runtime reexports alone.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "local-host-device-supervision-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_device_supervision_truth",
            rationale:
                "Proves the local stable host edge forwards recovered, exhausted, and faulted device supervision outcomes on the shared supervisor report seam.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "server-host-device-supervision-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_device_supervision_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned recovering and faulted device supervision outcomes without host-private restart policy.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools device_supervision_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable device supervision boundary aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-device-supervision-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared device supervision proof seam without reading private host restart policy or hardware-loop glue.",
        },
    ]
}

pub(crate) fn render_device_supervision_boundary_text() -> String {
    let mut rendered = format!(
        "device_supervision_boundary: {DEVICE_SUPERVISION_BOUNDARY}\ncontract_path: {DEVICE_SUPERVISION_CONTRACT_PATH}\nacceptance_task: {DEVICE_SUPERVISION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in device_supervision_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in device_supervision_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared device supervision truth is now public, but broader backend-matrix breadth and endpoint-topology depth still remain later hardware work",
        "the current boundary proves recovering, exhausted, and faulted device outcomes on shared runtime and host edges, not product-local recovery UX or remote hardware orchestration",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_device_supervision_boundary_json() -> String {
    let surfaces = device_supervision_boundary_surfaces()
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
    let validation_steps = device_supervision_boundary_validation_steps()
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
        "shared device supervision truth is now public, but broader backend-matrix breadth and endpoint-topology depth still remain later hardware work",
        "the current boundary proves recovering, exhausted, and faulted device outcomes on shared runtime and host edges, not product-local recovery UX or remote hardware orchestration",
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
        json_string(DEVICE_SUPERVISION_BOUNDARY),
        json_string(DEVICE_SUPERVISION_CONTRACT_PATH),
        json_string(DEVICE_SUPERVISION_ACCEPTANCE_TASK),
        device_supervision_boundary_surfaces().len(),
        surfaces,
        device_supervision_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
