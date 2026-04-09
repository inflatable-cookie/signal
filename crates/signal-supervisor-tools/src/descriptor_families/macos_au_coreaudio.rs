use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MacosAuCoreaudioSurfaceKind {
    Backend,
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MacosAuCoreaudioSurface {
    id: &'static str,
    kind: MacosAuCoreaudioSurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MacosAuCoreaudioValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl MacosAuCoreaudioSurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::Backend => "backend",
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn macos_au_coreaudio_boundary_surfaces() -> &'static [MacosAuCoreaudioSurface] {
    &[
        MacosAuCoreaudioSurface {
            id: "coreaudio-device-enumeration",
            kind: MacosAuCoreaudioSurfaceKind::Backend,
            crate_name: "signal-hardware-coreaudio",
            surface: "enumerate_devices() -> Vec<AudioDeviceDescriptor>",
            runtime_anchor: "AudioDeviceDescriptor",
            rationale:
                "Keeps macOS device identity, default-output presence, and degraded-versus-healthy diagnostics anchored in the real CoreAudio realization layer instead of a synthetic default-device shortcut.",
        },
        MacosAuCoreaudioSurface {
            id: "runtime-au-lifecycle-and-fault-report",
            kind: MacosAuCoreaudioSurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeHostObservationReport::host_io",
            runtime_anchor:
                "RuntimePluginLifecycleSnapshot + RuntimeHostIoSummary",
            rationale:
                "Keeps AU lifecycle, readiness, and fault truth aligned with runtime-owned host-I/O summaries so device-backed macOS bring-up remains one shared report surface.",
        },
        MacosAuCoreaudioSurface {
            id: "local-host-au-coreaudio-supervisor-report",
            kind: MacosAuCoreaudioSurfaceKind::HostEdge,
            crate_name: "signal-host-local",
            surface: "host_supervisor_report() -> RuntimeHostSupervisorReport",
            runtime_anchor: "LocalRuntimeHost::host_supervisor_report()",
            rationale:
                "Proves the stable local host edge exports real CoreAudio device truth plus AU lifecycle or AU fault truth from the same runtime-owned macOS supervisor surface.",
        },
    ]
}

fn macos_au_coreaudio_boundary_validation_steps() -> &'static [MacosAuCoreaudioValidationStep] {
    &[
        MacosAuCoreaudioValidationStep {
            id: "coreaudio-backend-proof",
            command: "cargo test -p signal-hardware-coreaudio",
            rationale:
                "Proves the CoreAudio backend enumerates real bounded device truth and exports degraded versus healthy diagnostics without the synthetic default-device production path.",
        },
        MacosAuCoreaudioValidationStep {
            id: "runtime-au-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
            rationale:
                "Keeps the AU discovery and lifecycle half of the macOS lane grounded in runtime-owned public contract truth.",
        },
        MacosAuCoreaudioValidationStep {
            id: "runtime-external-io-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth",
            rationale:
                "Keeps the host-I/O and device-facing side of the macOS lane grounded in runtime-owned public contract truth.",
        },
        MacosAuCoreaudioValidationStep {
            id: "local-host-au-coreaudio-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_au -- --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge exports AU baseline and AU fault truth alongside real CoreAudio-backed host-I/O state.",
        },
        MacosAuCoreaudioValidationStep {
            id: "local-host-hardware-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_external_io -- --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge exports runtime-owned CoreAudio-backed external-I/O truth from the same macOS host family.",
        },
        MacosAuCoreaudioValidationStep {
            id: "local-host-supervision-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_device_supervision -- --nocapture --test-threads=1",
            rationale:
                "Proves the same macOS host edge keeps device supervision and recovery truth aligned with the real CoreAudio-backed hardware path.",
        },
        MacosAuCoreaudioValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-macos-au-coreaudio-boundary --format=json",
            rationale:
                "Lets consumers inspect the focused macOS plugin-plus-device acceptance surface without reading host, AU adapter, or CoreAudio implementation detail.",
        },
    ]
}

pub(crate) fn render_macos_au_coreaudio_boundary_text() -> String {
    let mut rendered = format!(
        "macos_au_coreaudio_boundary: {MACOS_AU_COREAUDIO_BOUNDARY}\ncontract_path: {MACOS_AU_COREAUDIO_CONTRACT_PATH}\nacceptance_task: {MACOS_AU_COREAUDIO_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in macos_au_coreaudio_boundary_surfaces() {
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
    for step in macos_au_coreaudio_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "this boundary proves one honest macOS plugin-plus-device lane through AU lifecycle or AU fault truth and CoreAudio device truth, but it does not yet claim editor hosting, deeper parameter-tree breadth, or publication-grade AU render parity",
        "the stable local host edge is the authority for this focused macOS proof; broader server-host parity and the later interactive demo substrate remain separate milestones",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_macos_au_coreaudio_boundary_json() -> String {
    let surfaces = macos_au_coreaudio_boundary_surfaces()
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
    let validation_steps = macos_au_coreaudio_boundary_validation_steps()
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
        "this boundary proves one honest macOS plugin-plus-device lane through AU lifecycle or AU fault truth and CoreAudio device truth, but it does not yet claim editor hosting, deeper parameter-tree breadth, or publication-grade AU render parity",
        "the stable local host edge is the authority for this focused macOS proof; broader server-host parity and the later interactive demo substrate remain separate milestones",
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
        json_string(MACOS_AU_COREAUDIO_BOUNDARY),
        json_string(MACOS_AU_COREAUDIO_CONTRACT_PATH),
        json_string(MACOS_AU_COREAUDIO_ACCEPTANCE_TASK),
        macos_au_coreaudio_boundary_surfaces().len(),
        surfaces,
        macos_au_coreaudio_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
