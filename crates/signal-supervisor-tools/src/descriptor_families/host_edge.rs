use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostEdgeStabilityTier {
    Public,
    ConsumerFacingButUnstable,
    ScenarioOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostEdgeSurfaceRecord {
    id: &'static str,
    tier: HostEdgeStabilityTier,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HostEdgeValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

impl HostEdgeStabilityTier {
    fn label(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::ConsumerFacingButUnstable => "consumer-facing-but-unstable",
            Self::ScenarioOnly => "scenario-only",
        }
    }
}

fn host_edge_surface_records() -> &'static [HostEdgeSurfaceRecord] {
    &[
        HostEdgeSurfaceRecord {
            id: "host-constructors",
            tier: HostEdgeStabilityTier::Public,
            crate_name: "signal-host-local + signal-host-server",
            surface: "LocalRuntimeHost::new and ServerRuntimeHost::new",
            runtime_anchor: "SignalRuntime configuration and subscribed event stream ownership",
            rationale:
                "Host construction is shared-stable only as the thin entry into runtime-owned authority.",
        },
        HostEdgeSurfaceRecord {
            id: "shared-runtime-supervisor-api",
            tier: HostEdgeStabilityTier::Public,
            crate_name: "signal-host-local + signal-host-server",
            surface: "RuntimeSupervisorApi implemented by both hosts",
            runtime_anchor: "RuntimeSupervisorApi and runtime-owned receipts",
            rationale:
                "The shared stable host edge is the supervisor-oriented convenience layer that delegates back into runtime-owned orchestration.",
        },
        HostEdgeSurfaceRecord {
            id: "shared-supervisor-report",
            tier: HostEdgeStabilityTier::Public,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorReport and signal.supervisor.export",
            rationale:
                "Consumers inspect the stable shared host edge through runtime-owned report/export surfaces rather than host-private summaries.",
        },
        HostEdgeSurfaceRecord {
            id: "host-enriched-reports",
            tier: HostEdgeStabilityTier::ConsumerFacingButUnstable,
            crate_name: "signal-host-local",
            surface: "observation_report(), host_observation_report(), host_supervisor_report()",
            runtime_anchor:
                "RuntimeObservationReport, RuntimeHostObservationReport, RuntimeHostSupervisorReport",
            rationale:
                "These enrich runtime-owned meaning with host-specific context, but they remain asymmetric and are not yet part of the shared stable tier.",
        },
        HostEdgeSurfaceRecord {
            id: "host-summary-dtos",
            tier: HostEdgeStabilityTier::ConsumerFacingButUnstable,
            crate_name: "signal-host-local + signal-host-server",
            surface: "LocalRuntimeHostSummary and ServerRuntimeHostSummary",
            runtime_anchor: "Host summary structs only; not runtime-owned receipts",
            rationale:
                "Summary DTOs are still explanatory convenience shells rather than the canonical consumer inspection boundary.",
        },
        HostEdgeSurfaceRecord {
            id: "local-delegated-executor-helpers",
            tier: HostEdgeStabilityTier::ConsumerFacingButUnstable,
            crate_name: "signal-host-local",
            surface:
                "finalize_offline_render_with_local_delegated_executor() and render_offline_with_local_delegated_executor()",
            runtime_anchor: "runtime-owned delegated offline execution boundary",
            rationale:
                "These methods are useful local helpers, but they encode one adapter path and are not yet a backend-neutral host promise.",
        },
        HostEdgeSurfaceRecord {
            id: "scenario-boot-helpers",
            tier: HostEdgeStabilityTier::ScenarioOnly,
            crate_name: "signal-host-local + signal-host-server",
            surface: "boot_* fault, recovery, watchdog, and soak helpers",
            runtime_anchor: "scenario fixtures only",
            rationale:
                "Scenario boot helpers are fixtures and demos, not reusable stable consumer APIs.",
        },
    ]
}

fn host_edge_validation_steps() -> &'static [HostEdgeValidationStep] {
    &[
        HostEdgeValidationStep {
            id: "host-edge-boundary-description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json",
            rationale:
                "Consumers need one machine-readable descriptor for the shared host-edge boundary without reading private host code.",
        },
        HostEdgeValidationStep {
            id: "host-edge-boundary-acceptance",
            command: HOST_EDGE_ACCEPTANCE_TASK,
            rationale:
                "The repo-owned acceptance task keeps the boundary descriptor runnable instead of prose-only.",
        },
        HostEdgeValidationStep {
            id: "workspace-health",
            command: "effigy health",
            rationale:
                "Shared host-edge claims still depend on the repo-owned build baseline staying healthy.",
        },
    ]
}

pub(crate) fn render_host_edge_boundary_text() -> String {
    let mut rendered = format!(
        "host_edge_boundary: {HOST_EDGE_BOUNDARY}\ncontract_path: {HOST_EDGE_CONTRACT_PATH}\nacceptance_task: {HOST_EDGE_ACCEPTANCE_TASK}\nstable_surfaces:\n"
    );
    for surface in host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier == HostEdgeStabilityTier::Public)
    {
        rendered.push_str(&format!(
            "- id: {}\n  tier: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.tier.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("intentionally_unstable:\n");
    for surface in host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier != HostEdgeStabilityTier::Public)
    {
        rendered.push_str(&format!(
            "- id: {}\n  tier: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.tier.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in host_edge_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered
}

pub(crate) fn render_host_edge_boundary_json() -> String {
    let stable_surfaces = host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier == HostEdgeStabilityTier::Public)
        .map(|surface| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"tier\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"runtime_anchor\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(surface.id),
                json_string(surface.tier.label()),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let unstable_surfaces = host_edge_surface_records()
        .iter()
        .filter(|surface| surface.tier != HostEdgeStabilityTier::Public)
        .map(|surface| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"tier\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"runtime_anchor\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(surface.id),
                json_string(surface.tier.label()),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = host_edge_validation_steps()
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
            "\"stable_surfaces\":[{}],",
            "\"intentionally_unstable\":[{}],",
            "\"validation_steps\":[{}]",
            "}}"
        ),
        json_string(HOST_EDGE_BOUNDARY),
        json_string(HOST_EDGE_CONTRACT_PATH),
        json_string(HOST_EDGE_ACCEPTANCE_TASK),
        stable_surfaces,
        unstable_surfaces,
        validation_steps,
    )
}
