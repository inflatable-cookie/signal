use super::super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum HostEdgeStabilityTier {
    Public,
    ConsumerFacingButUnstable,
    ScenarioOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HostEdgeSurfaceRecord {
    pub(super) id: &'static str,
    pub(super) tier: HostEdgeStabilityTier,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct HostEdgeValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

impl HostEdgeStabilityTier {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::ConsumerFacingButUnstable => "consumer-facing-but-unstable",
            Self::ScenarioOnly => "scenario-only",
        }
    }
}

pub(super) fn host_edge_surface_records() -> &'static [HostEdgeSurfaceRecord] {
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

pub(super) fn host_edge_validation_steps() -> &'static [HostEdgeValidationStep] {
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
