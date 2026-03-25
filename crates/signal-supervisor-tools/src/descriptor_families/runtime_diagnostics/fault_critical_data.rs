#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FaultDiagnosticBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

impl FaultDiagnosticBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaultDiagnosticBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: FaultDiagnosticBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FaultDiagnosticBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CriticalPathBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

impl CriticalPathBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CriticalPathBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: CriticalPathBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct CriticalPathBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn fault_diagnostic_boundary_surfaces() -> &'static [FaultDiagnosticBoundarySurface] {
    &[
        FaultDiagnosticBoundarySurface {
            id: "runtime-observation-fault-diagnostic",
            kind: FaultDiagnosticBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::fault_diagnostic_receipt and RuntimeSupervisorReport::observation.fault_diagnostic_receipt",
            runtime_anchor: "RuntimeFaultDiagnosticReceipt",
            rationale:
                "Carries the canonical primary-family and typed contribution evidence directly on the public runtime observation and supervisor surfaces.",
        },
        FaultDiagnosticBoundarySurface {
            id: "runtime-profiling-fault-diagnostic",
            kind: FaultDiagnosticBoundarySurfaceKind::RuntimeReceipt,
            crate_name: "signal-runtime",
            surface: "RuntimeProfilingReceipt::fault_diagnostic_receipt",
            runtime_anchor: "RuntimeProfilingReceipt",
            rationale:
                "Keeps later profiling and soak work aligned to the same runtime-owned fault-diagnostic receipt rather than a separate performance-only taxonomy.",
        },
        FaultDiagnosticBoundarySurface {
            id: "shared-host-fault-diagnostic-report",
            kind: FaultDiagnosticBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures stable host edges expose the same canonical primary-family and contribution evidence without host-local causal reconstruction.",
        },
    ]
}

pub(super) fn fault_diagnostic_boundary_validation_steps(
) -> &'static [FaultDiagnosticBoundaryValidationStep] {
    &[
        FaultDiagnosticBoundaryValidationStep {
            id: "runtime-public-fault-diagnostic-proof",
            command:
                "cargo test -p signal-runtime public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts",
            rationale:
                "Proves a downstream-style runtime consumer can read canonical primary-family and typed contribution evidence through public runtime surfaces.",
        },
        FaultDiagnosticBoundaryValidationStep {
            id: "local-host-fault-diagnostic-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_fault_diagnostic_truth",
            rationale:
                "Proves the local shared host edge forwards the runtime-owned fault-diagnostic receipt without private host-side diagnosis.",
        },
        FaultDiagnosticBoundaryValidationStep {
            id: "server-host-fault-diagnostic-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_fault_diagnostic_truth",
            rationale:
                "Proves the server shared host edge forwards the same runtime-owned fault-diagnostic receipt without server-local causal rewriting.",
        },
        FaultDiagnosticBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-fault-diagnostic-boundary --format=json",
            rationale:
                "Lets downstream tooling inspect the fault-diagnostic boundary, proof commands, and deferred scope without private implementation detail.",
        },
    ]
}

pub(super) fn critical_path_boundary_surfaces() -> &'static [CriticalPathBoundarySurface] {
    &[
        CriticalPathBoundarySurface {
            id: "runtime-performance-hotspot-report",
            kind: CriticalPathBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::performance_snapshot() and RuntimeSupervisorReport::performance_snapshot()",
            runtime_anchor: "RuntimePerformanceSnapshot",
            rationale:
                "Carries the bounded hot-node, hot-group, critical-path lane, and typed worker-lane summaries directly on the public runtime report boundary.",
        },
        CriticalPathBoundarySurface {
            id: "runtime-performance-trace-digest",
            kind: CriticalPathBoundarySurfaceKind::RuntimeReceipt,
            crate_name: "signal-runtime",
            surface: "RuntimePerformanceTraceReceipt",
            runtime_anchor: "RuntimePerformanceTraceReceipt",
            rationale:
                "Keeps peak hot-group and critical-path lane evidence consumable across an observation window without private tracing hooks.",
        },
        CriticalPathBoundarySurface {
            id: "shared-host-critical-path-report",
            kind: CriticalPathBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures stable host edges forward the same bounded hotspot and lane receipts without host-local scheduler reconstruction.",
        },
    ]
}

pub(super) fn critical_path_boundary_validation_steps(
) -> &'static [CriticalPathBoundaryValidationStep] {
    &[
        CriticalPathBoundaryValidationStep {
            id: "runtime-public-critical-path-proof",
            command:
                "cargo test -p signal-runtime public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts",
            rationale:
                "Proves a downstream-style runtime consumer can inspect bounded hot-node, hot-group, critical-path lane, and worker-lane summaries through public reexports.",
        },
        CriticalPathBoundaryValidationStep {
            id: "local-host-critical-path-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_critical_path_truth",
            rationale:
                "Proves the local shared host edge forwards the same bounded hotspot and lane receipts on supervisor export without private runtime hooks.",
        },
        CriticalPathBoundaryValidationStep {
            id: "server-host-critical-path-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_critical_path_truth",
            rationale:
                "Proves the server shared host edge forwards the same bounded hotspot and lane receipts on supervisor export without server-local reinterpretation.",
        },
        CriticalPathBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-critical-path-boundary --format=json",
            rationale:
                "Lets downstream consumers inspect the bounded critical-path proof boundary without reading private runtime or host implementation detail.",
        },
    ]
}
