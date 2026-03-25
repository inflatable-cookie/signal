use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PluginContinuityBoundarySurfaceKind {
    RuntimeSnapshot,
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PluginContinuityBoundarySurface {
    id: &'static str,
    kind: PluginContinuityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PluginContinuityValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl PluginContinuityBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

fn plugin_continuity_boundary_surfaces() -> &'static [PluginContinuityBoundarySurface] {
    &[
        PluginContinuityBoundarySurface {
            id: "runtime-plugin-lifecycle-snapshot",
            kind: PluginContinuityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeSupervisorReport::observation.plugin_lifecycle_snapshot",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Carries runtime-owned placement outcome, grouping key, shared-boundary member count, continuity class, and rebindability directly on public reports.",
        },
        PluginContinuityBoundarySurface {
            id: "runtime-plugin-chain-snapshot",
            kind: PluginContinuityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_chain_snapshot()",
            runtime_anchor: "RuntimePluginChainSnapshot",
            rationale:
                "Keeps stage-level placement and continuity truth inspectable without reconstructing blast radius from host-private transport notes.",
        },
        PluginContinuityBoundarySurface {
            id: "runtime-plugin-placement-policy",
            kind: PluginContinuityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeProjectionApi::apply_plugin_placement_policy()",
            runtime_anchor: "RuntimePluginPlacementPolicy",
            rationale:
                "Freezes one runtime-owned allowlist, denylist, and by-format placement surface instead of product-local sandbox policy tables.",
        },
        PluginContinuityBoundarySurface {
            id: "shared-host-plugin-supervisor-report",
            kind: PluginContinuityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward plugin placement and shared-boundary continuity truth without host-local rule reconstruction.",
        },
    ]
}

fn plugin_continuity_validation_steps() -> &'static [PluginContinuityValidationStep] {
    &[
        PluginContinuityValidationStep {
            id: "runtime-shared-boundary-blast-radius-proof",
            command:
                "cargo test -p signal-runtime runtime_shared_sandbox_blast_radius_stays_boundary_local_across_recovery_and_terminal_states",
            rationale:
                "Proves one shared boundary can degrade, recover, and fail terminally across several member instances without contaminating sibling boundaries.",
        },
        PluginContinuityValidationStep {
            id: "runtime-placement-policy-proof",
            command:
                "cargo test -p signal-runtime runtime_plugin_placement_policy_exports_allowlist_denylist_and_by_format_receipts",
            rationale:
                "Proves runtime-owned allowlist, denylist, and by-format policy outcomes stay explicit on lifecycle and chain receipts.",
        },
        PluginContinuityValidationStep {
            id: "runtime-public-boundary-proof",
            command:
                "cargo test -p signal-runtime public_runtime_plugin_continuity_boundary_reports_shared_boundary_and_policy_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect shared-boundary continuity and policy truth through public reexports.",
        },
        PluginContinuityValidationStep {
            id: "local-host-plugin-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth",
            rationale:
                "Proves the local shared host edge preserves placement and shared-boundary continuity truth on supervisor export.",
        },
        PluginContinuityValidationStep {
            id: "server-host-plugin-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_plugin_placement_and_shared_boundary_continuity_truth",
            rationale:
                "Proves the server shared host edge preserves placement and shared-boundary continuity truth on supervisor export.",
        },
        PluginContinuityValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-plugin-continuity-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared plugin continuity proof boundary without reading private runtime or host implementation detail.",
        },
    ]
}

pub(crate) fn render_plugin_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "plugin_continuity_boundary: {PLUGIN_CONTINUITY_BOUNDARY}\ncontract_path: {PLUGIN_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {PLUGIN_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in plugin_continuity_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id, surface.kind.label(), surface.crate_name, surface.surface, surface.runtime_anchor, surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in plugin_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared-boundary blast radius is now explicit, but dedicated blast-radius DTOs are still deferred beyond the current lifecycle and chain receipts",
        "the exercised proof path is still sandbox-first, so deeper in-process parity and broader adapter transport tuning remain later plugin-format work",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

pub(crate) fn render_plugin_continuity_boundary_json() -> String {
    let surfaces = plugin_continuity_boundary_surfaces()
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
    let validation_steps = plugin_continuity_validation_steps()
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
        "shared-boundary blast radius is now explicit, but dedicated blast-radius DTOs are still deferred beyond the current lifecycle and chain receipts",
        "the exercised proof path is still sandbox-first, so deeper in-process parity and broader adapter transport tuning remain later plugin-format work",
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
        json_string(PLUGIN_CONTINUITY_BOUNDARY),
        json_string(PLUGIN_CONTINUITY_CONTRACT_PATH),
        json_string(PLUGIN_CONTINUITY_ACCEPTANCE_TASK),
        plugin_continuity_boundary_surfaces().len(),
        surfaces,
        plugin_continuity_validation_steps().len(),
        validation_steps,
        deferred_scope
    )
}
