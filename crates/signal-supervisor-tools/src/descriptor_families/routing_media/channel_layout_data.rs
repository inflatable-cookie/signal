#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum MultichannelBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl MultichannelBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MultichannelBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: MultichannelBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct MultichannelBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn multichannel_boundary_surfaces() -> &'static [MultichannelBoundarySurface] {
    &[
        MultichannelBoundarySurface {
            id: "runtime-multichannel-topology-report",
            kind: MultichannelBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::execution_topology_summary, RuntimeObservationReport::external_io_snapshot, and RuntimeSupervisorReport::observation.{execution_topology_summary,external_io_snapshot}",
            runtime_anchor:
                "RuntimeExecutionTopologySummary + RuntimeExternalIoSnapshot multichannel receipts",
            rationale:
                "Keeps canonical layout, channel-role, and bus-intent meaning on one runtime-owned report seam instead of host-local topology reinterpretation.",
        },
        MultichannelBoundarySurface {
            id: "runtime-multichannel-plugin-discovery-snapshot",
            kind: MultichannelBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_discovery_snapshot()",
            runtime_anchor: "RuntimePluginDiscoveredTypeRecord::default_multichannel_io",
            rationale:
                "Lets downstream consumers inspect canonical default plugin multichannel layout and channel-role coverage directly from runtime-owned discovery snapshots.",
        },
        MultichannelBoundarySurface {
            id: "shared-host-multichannel-report",
            kind: MultichannelBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned canonical layout, channel-role, and bus-intent receipts without host-local reinterpretation.",
        },
    ]
}

pub(super) fn multichannel_boundary_validation_steps(
) -> &'static [MultichannelBoundaryValidationStep] {
    &[
        MultichannelBoundaryValidationStep {
            id: "runtime-multichannel-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_multichannel_boundary_reports_runtime_owned_layout_and_role_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect canonical layouts, channel roles, and bus intents through public runtime reexports alone.",
        },
        MultichannelBoundaryValidationStep {
            id: "local-host-multichannel-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_multichannel_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned multichannel topology and plugin discovery receipts on supervisor export.",
        },
        MultichannelBoundaryValidationStep {
            id: "server-host-multichannel-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_multichannel_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned canonical layout and bus-intent receipts without private host reinterpretation.",
        },
        MultichannelBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools multichannel_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable multichannel boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        MultichannelBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-multichannel-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared canonical multichannel layout and channel-role seam without reading host-local topology glue.",
        },
    ]
}
