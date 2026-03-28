use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LinuxPluginParityBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxPluginParityBoundarySurface {
    id: &'static str,
    kind: LinuxPluginParityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LinuxPluginParityBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl LinuxPluginParityBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

fn linux_plugin_parity_boundary_surfaces() -> &'static [LinuxPluginParityBoundarySurface] {
    &[
        LinuxPluginParityBoundarySurface {
            id: "runtime-linux-parity-discovery-report",
            kind: LinuxPluginParityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor: "RuntimePluginDiscoverySnapshot",
            rationale:
                "Keeps CLAP, VST3, and LV2 Linux parity bands, Linux support, and Linux sandbox defaults consumable through the shared discovery report seam instead of a Linux-only host matrix.",
        },
        LinuxPluginParityBoundarySurface {
            id: "runtime-linux-parity-lifecycle-snapshot",
            kind: LinuxPluginParityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps Linux placement, restart, rebindability, and failure counts on the existing runtime-owned lifecycle seam.",
        },
        LinuxPluginParityBoundarySurface {
            id: "server-host-linux-parity-supervisor-report",
            kind: LinuxPluginParityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures the stable Linux-facing server host edge forwards one runtime-owned Linux plugin vocabulary without host-local portability matrices.",
        },
    ]
}

fn linux_plugin_parity_boundary_validation_steps(
) -> &'static [LinuxPluginParityBoundaryValidationStep] {
    &[
        LinuxPluginParityBoundaryValidationStep {
            id: "runtime-linux-parity-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_linux_plugin_parity_boundary_reports_runtime_owned_linux_policy_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect Linux-specific parity and sandbox-policy truth through public runtime reexports alone.",
        },
        LinuxPluginParityBoundaryValidationStep {
            id: "server-host-linux-parity-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_linux_plugin_parity_truth",
            rationale:
                "Proves the stable server host edge forwards runtime-owned Linux parity, restart, and failure posture on supervisor export.",
        },
        LinuxPluginParityBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-linux-plugin-parity-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared Linux plugin parity proof boundary without reading host-private Linux policy code or adapter internals.",
        },
    ]
}

pub(crate) fn render_linux_plugin_parity_boundary_text() -> String {
    let mut rendered = format!(
        "linux_plugin_parity_boundary: {LINUX_PLUGIN_PARITY_BOUNDARY}\ncontract_path: {LINUX_PLUGIN_PARITY_CONTRACT_PATH}\nacceptance_task: {LINUX_PLUGIN_PARITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in linux_plugin_parity_boundary_surfaces() {
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
    for step in linux_plugin_parity_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared Linux CLAP, VST3, and LV2 parity truth is now public, but richer extension-depth parity such as CLAP extensions, VST3 units, and LV2 worker or UI behavior still remains later work",
        "the current boundary proves one bounded Linux plugin vocabulary for discovery, lifecycle, placement, restart, and failure policy, not broader ALSA, JACK, or PipeWire backend parity",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_linux_plugin_parity_boundary_json() -> String {
    let surfaces = linux_plugin_parity_boundary_surfaces()
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
    let validation_steps = linux_plugin_parity_boundary_validation_steps()
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
        "shared Linux CLAP, VST3, and LV2 parity truth is now public, but richer extension-depth parity such as CLAP extensions, VST3 units, and LV2 worker or UI behavior still remains later work",
        "the current boundary proves one bounded Linux plugin vocabulary for discovery, lifecycle, placement, restart, and failure policy, not broader ALSA, JACK, or PipeWire backend parity",
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
        json_string(LINUX_PLUGIN_PARITY_BOUNDARY),
        json_string(LINUX_PLUGIN_PARITY_CONTRACT_PATH),
        json_string(LINUX_PLUGIN_PARITY_ACCEPTANCE_TASK),
        linux_plugin_parity_boundary_surfaces().len(),
        surfaces,
        linux_plugin_parity_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}
