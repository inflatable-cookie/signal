use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PipeWireAlsaParityBoundarySurfaceKind {
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PipeWireAlsaParityBoundarySurface {
    id: &'static str,
    kind: PipeWireAlsaParityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PipeWireAlsaParityBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl PipeWireAlsaParityBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn pipewire_alsa_parity_boundary_surfaces() -> &'static [PipeWireAlsaParityBoundarySurface] {
    &[
        PipeWireAlsaParityBoundarySurface {
            id: "runtime-pipewire-alsa-parity-report",
            kind: PipeWireAlsaParityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::pipewire_alsa_parity_snapshot and RuntimeSupervisorReport::observation.pipewire_alsa_parity_snapshot",
            runtime_anchor: "RuntimePipeWireAlsaParitySnapshot",
            rationale:
                "Keeps PipeWire and ALSA session-role, device-claim, stream-policy, and guarded parity on one runtime-owned receipt instead of host-local daemon or callback policy.",
        },
        PipeWireAlsaParityBoundarySurface {
            id: "shared-host-pipewire-alsa-supervisor-report",
            kind: PipeWireAlsaParityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures stable host edges forward the same runtime-owned PipeWire/ALSA parity seam without host-specific reclassification.",
        },
    ]
}

fn pipewire_alsa_parity_boundary_validation_steps(
) -> &'static [PipeWireAlsaParityBoundaryValidationStep] {
    &[
        PipeWireAlsaParityBoundaryValidationStep {
            id: "runtime-pipewire-alsa-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_pipewire_alsa_parity_boundary_reports_runtime_owned_claim_and_policy_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect ALSA and PipeWire session-role, claim, policy, and guarded parity through public runtime surfaces.",
        },
        PipeWireAlsaParityBoundaryValidationStep {
            id: "local-host-pipewire-alsa-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_pipewire_alsa_parity_truth",
            rationale:
                "Proves the stable local host edge exports an explicit non-target PipeWire/ALSA answer instead of leaving the parity seam absent.",
        },
        PipeWireAlsaParityBoundaryValidationStep {
            id: "server-host-pipewire-alsa-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_pipewire_alsa_parity_truth",
            rationale:
                "Proves the stable server host edge forwards the bounded backend-managed PipeWire parity baseline without server-local policy.",
        },
        PipeWireAlsaParityBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-pipewire-alsa-parity-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared PipeWire/ALSA parity proof boundary without reading backend-private host code.",
        },
    ]
}

pub(crate) fn render_pipewire_alsa_parity_boundary_text() -> String {
    let mut rendered = format!(
        "pipewire_alsa_parity_boundary: {PIPEWIRE_ALSA_PARITY_BOUNDARY}\ncontract_path: {PIPEWIRE_ALSA_PARITY_CONTRACT_PATH}\nacceptance_task: {PIPEWIRE_ALSA_PARITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in pipewire_alsa_parity_boundary_surfaces() {
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
    for step in pipewire_alsa_parity_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str(
        "residual_risk: the current boundary proves bounded PipeWire and ALSA parity receipts, not full daemon, portal, reservation, or distro-policy depth\n",
    );
    rendered
}

pub(crate) fn render_pipewire_alsa_parity_boundary_json() -> String {
    let surfaces = pipewire_alsa_parity_boundary_surfaces()
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
    let validation_steps = pipewire_alsa_parity_boundary_validation_steps()
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
        json_string(PIPEWIRE_ALSA_PARITY_BOUNDARY),
        json_string(PIPEWIRE_ALSA_PARITY_CONTRACT_PATH),
        json_string(PIPEWIRE_ALSA_PARITY_ACCEPTANCE_TASK),
        pipewire_alsa_parity_boundary_surfaces().len(),
        pipewire_alsa_parity_boundary_validation_steps().len(),
        surfaces,
        validation_steps,
        json_string(
            "the current boundary proves bounded PipeWire and ALSA parity receipts, not full daemon, portal, reservation, or distro-policy depth",
        ),
    )
}
