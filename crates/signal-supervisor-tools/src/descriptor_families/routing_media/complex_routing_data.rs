#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ComplexIoBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl ComplexIoBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ComplexIoBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: ComplexIoBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ComplexIoBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn complex_io_boundary_surfaces() -> &'static [ComplexIoBoundarySurface] {
    &[
        ComplexIoBoundarySurface {
            id: "runtime-complex-io-discovery-report",
            kind: ComplexIoBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor:
                "RuntimePluginDiscoveredTypeRecord::complex_io_summary + plugin discovery coverage receipts",
            rationale:
                "Keeps complex plugin-I/O, multi-output instrument, and bus-capable FX meaning on one runtime-owned discovery and capability surface instead of adapter-local pin reconstruction.",
        },
        ComplexIoBoundarySurface {
            id: "runtime-plugin-pin-matrix-report",
            kind: ComplexIoBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_pin_matrix_snapshot and RuntimeSupervisorReport::observation.plugin_pin_matrix_snapshot",
            runtime_anchor: "RuntimePluginPinMatrixSnapshot",
            rationale:
                "Keeps pin-group identity, pin-matrix posture, dynamic bus-negotiation posture, and bounded fallback outcome on one runtime-owned routing surface instead of host-local bus activation policy.",
        },
        ComplexIoBoundarySurface {
            id: "runtime-complex-io-plugin-chain-snapshot",
            kind: ComplexIoBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_chain_snapshot and RuntimeSupervisorReport::observation.plugin_chain_snapshot",
            runtime_anchor: "RuntimePluginChainStageSnapshot::complex_io_summary",
            rationale:
                "Lets downstream consumers inspect complex plugin-I/O topology on live stage receipts rather than inferring multi-output or bus-capable behavior from adapter-private bus names.",
        },
        ComplexIoBoundarySurface {
            id: "runtime-complex-io-render-contract-preview",
            kind: ComplexIoBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeOfflineRenderContractPreview::chain_contract",
            runtime_anchor:
                "RuntimeOfflineRenderChainDependencyPreview::{complex_io_stage_count,complex_io_stages}",
            rationale:
                "Carries the same runtime-owned complex plugin-I/O topology into deferred render dependency preview instead of rebuilding topology per export path.",
        },
        ComplexIoBoundarySurface {
            id: "shared-host-complex-io-report",
            kind: ComplexIoBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned complex plugin-I/O topology receipts without adapter-local pin reconstruction.",
        },
    ]
}

pub(super) fn complex_io_boundary_validation_steps() -> &'static [ComplexIoBoundaryValidationStep] {
    &[
        ComplexIoBoundaryValidationStep {
            id: "runtime-complex-io-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_complex_io_boundary_reports_runtime_owned_topology_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect complex plugin-I/O discovery, stage, and offline preview receipts through public runtime surfaces alone.",
        },
        ComplexIoBoundaryValidationStep {
            id: "local-host-complex-io-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_complex_io_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned complex plugin-I/O, multi-output instrument, and bus-capable FX receipts on supervisor export.",
        },
        ComplexIoBoundaryValidationStep {
            id: "server-host-complex-io-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_complex_io_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned complex plugin-I/O receipts without adapter-local pin reconstruction.",
        },
        ComplexIoBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools complex_io_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable complex plugin-I/O boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        ComplexIoBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-complex-io-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared complex plugin-I/O seam without reading adapter-private pin or bus glue.",
        },
    ]
}
