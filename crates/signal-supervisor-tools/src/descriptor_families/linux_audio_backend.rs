use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxAudioBackendBoundarySurfaceKind {
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxAudioBackendBoundarySurface {
    id: &'static str,
    kind: LinuxAudioBackendBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxAudioBackendBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl LinuxAudioBackendBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn linux_audio_backend_boundary_surfaces() -> &'static [LinuxAudioBackendBoundarySurface] {
    &[
        LinuxAudioBackendBoundarySurface {
            id: "runtime-linux-audio-observation-report",
            kind: LinuxAudioBackendBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
            runtime_anchor: "RuntimeExternalIoSnapshot",
            rationale:
                "Keeps ALSA, JACK, PipeWire, unavailable, and non-Linux backend classification on one runtime-owned external-I/O seam instead of backend-private capability tables.",
        },
        LinuxAudioBackendBoundarySurface {
            id: "server-host-linux-audio-supervisor-report",
            kind: LinuxAudioBackendBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures the stable Linux-facing server host edge forwards explicit runtime-owned unavailable and fallback Linux backend truth instead of host-local Linux hardware heuristics.",
        },
    ]
}

fn linux_audio_backend_boundary_validation_steps(
) -> &'static [LinuxAudioBackendBoundaryValidationStep] {
    &[
        LinuxAudioBackendBoundaryValidationStep {
            id: "runtime-linux-audio-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_linux_audio_backend_boundary_reports_runtime_owned_backend_identity_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect ALSA, JACK, PipeWire, and unavailable Linux backend identity and portability-band truth through public runtime surfaces alone.",
        },
        LinuxAudioBackendBoundaryValidationStep {
            id: "server-host-linux-audio-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_linux_audio_backend_truth",
            rationale:
                "Proves the stable server host edge forwards the runtime-owned unavailable Linux backend fallback state instead of inventing host-local Linux capability matrices.",
        },
        LinuxAudioBackendBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-linux-audio-backend-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared Linux audio backend proof boundary without reading backend-private Linux host code.",
        },
    ]
}

pub(crate) fn render_linux_audio_backend_boundary_text() -> String {
    let mut output = format!(
        "linux_audio_backend_boundary: {LINUX_AUDIO_BACKEND_BOUNDARY}\ncontract_path: {LINUX_AUDIO_BACKEND_CONTRACT_PATH}\nacceptance_task: {LINUX_AUDIO_BACKEND_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in linux_audio_backend_boundary_surfaces() {
        output.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    output.push_str("validation_steps:\n");
    for step in linux_audio_backend_boundary_validation_steps() {
        output.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    output.push_str(
        "residual_risk: the current boundary proves typed Linux backend identity, portability-band, and unavailable fallback export, not live ALSA/JACK/PipeWire host ownership or deeper backend-native graph behavior\n",
    );
    output
}

pub(crate) fn render_linux_audio_backend_boundary_json() -> String {
    let surfaces = linux_audio_backend_boundary_surfaces()
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
    let validation_steps = linux_audio_backend_boundary_validation_steps()
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
        json_string(LINUX_AUDIO_BACKEND_BOUNDARY),
        json_string(LINUX_AUDIO_BACKEND_CONTRACT_PATH),
        json_string(LINUX_AUDIO_BACKEND_ACCEPTANCE_TASK),
        linux_audio_backend_boundary_surfaces().len(),
        linux_audio_backend_boundary_validation_steps().len(),
        surfaces,
        validation_steps,
        json_string(
            "the current boundary proves typed Linux backend identity, portability-band, and unavailable fallback export, not live ALSA/JACK/PipeWire host ownership or deeper backend-native graph behavior",
        ),
    )
}
