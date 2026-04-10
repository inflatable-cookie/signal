#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MultiBusBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl MultiBusBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MultiBusBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: MultiBusBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MultiBusBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SidechainBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl SidechainBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SidechainBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: SidechainBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SidechainBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn multi_bus_boundary_surfaces() -> &'static [MultiBusBoundarySurface] {
    &[
        MultiBusBoundarySurface {
            id: "runtime-multi-bus-topology-report",
            kind: MultiBusBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::execution_topology_summary, RuntimeObservationReport::metering_snapshot, and RuntimeSupervisorReport::observation.{execution_topology_summary,metering_snapshot}",
            runtime_anchor:
                "RuntimeExecutionTopologySummary + RuntimeMeteringSnapshot multi-bus connection and auxiliary-path receipts",
            rationale:
                "Keeps bus-role, connection-identity, auxiliary-path, and fallback meaning on one runtime-owned report seam across live execution and diagnostics.",
        },
        MultiBusBoundarySurface {
            id: "runtime-multi-bus-render-contract-preview",
            kind: MultiBusBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeOfflineRenderContractPreview::chain_contract",
            runtime_anchor: "RuntimeOfflineRenderChainDependencyPreview",
            rationale:
                "Lets downstream consumers inspect the same runtime-owned multi-bus connection and auxiliary-path receipts on offline dependency preview without host-local route reconstruction.",
        },
        MultiBusBoundarySurface {
            id: "shared-host-multi-bus-report",
            kind: MultiBusBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned multi-bus connection and auxiliary-path receipts without private routing reinterpretation.",
        },
    ]
}

pub(super) fn multi_bus_boundary_validation_steps() -> &'static [MultiBusBoundaryValidationStep] {
    &[
        MultiBusBoundaryValidationStep {
            id: "runtime-multi-bus-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_multi_bus_boundary_reports_runtime_owned_connection_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect bus-role, connection-identity, and auxiliary-path receipts through public runtime reports alone.",
        },
        MultiBusBoundaryValidationStep {
            id: "local-host-multi-bus-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_multi_bus local_shared_host_edge_exports_runtime_multi_bus_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge forwards runtime-owned multi-bus topology and metering receipts on supervisor export.",
        },
        MultiBusBoundaryValidationStep {
            id: "server-host-multi-bus-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_multi_bus server_shared_host_edge_exports_runtime_multi_bus_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned multi-bus routing receipts without host-local reinterpretation.",
        },
        MultiBusBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools multi_bus_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable multi-bus boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        MultiBusBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-multi-bus-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared multi-bus connection and auxiliary-path seam without reading host-local routing glue.",
        },
    ]
}

pub(super) fn sidechain_boundary_surfaces() -> &'static [SidechainBoundarySurface] {
    &[
        SidechainBoundarySurface {
            id: "runtime-sidechain-topology-report",
            kind: SidechainBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::execution_topology_summary, RuntimeSupervisorReport::observation.{execution_topology_summary,plugin_chain_snapshot}, and RuntimeOfflineRenderContractPreview::chain_contract",
            runtime_anchor:
                "RuntimeExecutionTopologySummary + RuntimePluginChainSnapshot + RuntimeOfflineRenderChainDependencyPreview",
            rationale:
                "Keeps sidechain source, target, attachment policy, and fallback meaning on one runtime-owned routing seam across live and offline surfaces.",
        },
        SidechainBoundarySurface {
            id: "runtime-sidechain-contract-projection",
            kind: SidechainBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "GraphNodeBufferContractProjection::secondary_input",
            runtime_anchor: "RuntimeSecondaryInputContractProjection",
            rationale:
                "Lets downstream consumers inspect the declared sidechain contract before host-local patching or adapter-private routing can reinterpret it.",
        },
        SidechainBoundarySurface {
            id: "shared-host-sidechain-report",
            kind: SidechainBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned sidechain and secondary-input receipts without product-local routing reconstruction.",
        },
    ]
}

pub(super) fn sidechain_boundary_validation_steps() -> &'static [SidechainBoundaryValidationStep] {
    &[
        SidechainBoundaryValidationStep {
            id: "runtime-sidechain-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_sidechain_boundary_reports_runtime_owned_secondary_input_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect sidechain source, target, and fallback meaning through public runtime reports alone.",
        },
        SidechainBoundaryValidationStep {
            id: "local-host-sidechain-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_boundary local_shared_host_edge_exports_runtime_sidechain_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable local host edge forwards runtime-owned sidechain topology and plugin-stage receipts on supervisor export.",
        },
        SidechainBoundaryValidationStep {
            id: "server-host-sidechain-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_boundary server_shared_host_edge_exports_runtime_sidechain_truth -- --exact --nocapture --test-threads=1",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned sidechain receipts without host-local routing reinterpretation.",
        },
        SidechainBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools sidechain_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable sidechain boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        SidechainBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-sidechain-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared secondary-input routing seam without reading host-local patch or adapter glue.",
        },
    ]
}
