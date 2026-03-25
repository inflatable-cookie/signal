#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ClockTopologyBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl ClockTopologyBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ClockTopologyBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: ClockTopologyBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ClockTopologyBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ExternalIoBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

impl ExternalIoBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExternalIoBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: ExternalIoBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct ExternalIoBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn clock_topology_boundary_surfaces() -> &'static [ClockTopologyBoundarySurface] {
    &[
        ClockTopologyBoundarySurface {
            id: "runtime-host-clocking-report",
            kind: ClockTopologyBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeHostObservationReport::host_io and RuntimeHostSupervisorReport::observation.host_io",
            runtime_anchor: "RuntimeHostClockingSummary + RuntimeExternalIoSnapshot",
            rationale:
                "Keeps drift, discontinuity, duplex-mismatch, and endpoint-topology meaning on one runtime-owned live-path seam instead of backend-private callback or device-list heuristics.",
        },
        ClockTopologyBoundarySurface {
            id: "runtime-external-io-alignment",
            kind: ClockTopologyBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
            runtime_anchor: "RuntimeExternalIoSnapshot",
            rationale:
                "Keeps live clocking semantics aligned with the shared external-I/O receipt family instead of a separate host-only topology shell.",
        },
        ClockTopologyBoundarySurface {
            id: "shared-local-host-clock-topology-report",
            kind: ClockTopologyBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local",
            surface: "LocalRuntimeHost::host_supervisor_report() -> RuntimeHostSupervisorReport",
            runtime_anchor: "RuntimeHostSupervisorReport",
            rationale:
                "Proves the stable local host edge forwards runtime-owned drift, discontinuity, duplex-mismatch, and endpoint-topology truth without backend-private clock reconstruction.",
        },
    ]
}

pub(super) fn clock_topology_boundary_validation_steps(
) -> &'static [ClockTopologyBoundaryValidationStep] {
    &[
        ClockTopologyBoundaryValidationStep {
            id: "runtime-clock-topology-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned drift, discontinuity, duplex-mismatch, and endpoint-topology truth through public reexports.",
        },
        ClockTopologyBoundaryValidationStep {
            id: "local-host-clock-topology-public-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_clock_topology_truth",
            rationale:
                "Proves the stable local host edge exposes runtime-owned live host clocking and topology receipts without private callback heuristics.",
        },
        ClockTopologyBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools clock_topology_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable clock-topology boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        ClockTopologyBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-clock-topology-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared live clocking and endpoint-topology seam without reading private backend callback or device enumeration glue.",
        },
    ]
}

pub(super) fn external_io_boundary_surfaces() -> &'static [ExternalIoBoundarySurface] {
    &[
        ExternalIoBoundarySurface {
            id: "runtime-external-io-report",
            kind: ExternalIoBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
            runtime_anchor: "RuntimeExternalIoSnapshot",
            rationale:
                "Keeps external-I/O role, monitor state, tap-point, and bounded loopback meaning on one runtime-owned seam instead of host-private monitor helpers.",
        },
        ExternalIoBoundarySurface {
            id: "runtime-host-external-io-report",
            kind: ExternalIoBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeHostObservationReport::observation.external_io_snapshot and RuntimeHostSupervisorReport::observation.observation.external_io_snapshot",
            runtime_anchor: "RuntimeHostObservationReport + RuntimeHostSupervisorReport",
            rationale:
                "Shows the same runtime-owned external-I/O receipt family remains aligned when host-I/O context is added to broader host observation exports.",
        },
        ExternalIoBoundarySurface {
            id: "shared-local-host-external-io-report",
            kind: ExternalIoBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local",
            surface: "LocalRuntimeHost::supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorReport",
            rationale:
                "Proves the stable local host edge forwards runtime-owned direct and faulted external-I/O monitoring truth without private monitor helpers.",
        },
        ExternalIoBoundarySurface {
            id: "shared-server-host-external-io-report",
            kind: ExternalIoBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-server",
            surface: "ServerRuntimeHost::supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorReport",
            rationale:
                "Proves the stable server host edge exports the same runtime-owned external-I/O receipt shape with explicit unavailable monitoring and loopback state instead of adapter-local reconstruction.",
        },
    ]
}

pub(super) fn external_io_boundary_validation_steps() -> &'static [ExternalIoBoundaryValidationStep]
{
    &[
        ExternalIoBoundaryValidationStep {
            id: "runtime-external-io-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect runtime-owned monitoring, tap-point, and loopback truth without host-private helper code.",
        },
        ExternalIoBoundaryValidationStep {
            id: "local-host-external-io-public-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_external_io_truth",
            rationale:
                "Proves the stable local host edge exposes runtime-owned direct and explicit faulted external-I/O receipts without local monitor reconstruction.",
        },
        ExternalIoBoundaryValidationStep {
            id: "server-host-external-io-public-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_external_io_truth",
            rationale:
                "Proves the stable server host edge exports explicit unavailable monitoring and loopback state through the shared runtime receipt family.",
        },
        ExternalIoBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools external_io_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable external-I/O boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        ExternalIoBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-external-io-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared external-I/O, monitoring, tap-point, and loopback seam without reading private host derivation code.",
        },
    ]
}
