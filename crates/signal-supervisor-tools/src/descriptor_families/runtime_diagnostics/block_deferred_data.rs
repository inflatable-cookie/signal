#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BlockTimingBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

impl BlockTimingBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BlockTimingBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: BlockTimingBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BlockTimingBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum DeferredWorkPolicyBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

impl DeferredWorkPolicyBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeferredWorkPolicyBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: DeferredWorkPolicyBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct DeferredWorkPolicyBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn block_timing_boundary_surfaces() -> &'static [BlockTimingBoundarySurface] {
    &[
        BlockTimingBoundarySurface {
            id: "runtime-engine-block-snapshot",
            kind: BlockTimingBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::engine_block_snapshot and RuntimeSupervisorReport::observation.engine_block_snapshot",
            runtime_anchor: "RuntimeEngineBlockSnapshot",
            rationale:
                "Carries the canonical bounded block timing, deadline budget, pressure class, and overrun counters directly on the public runtime report boundary.",
        },
        BlockTimingBoundarySurface {
            id: "runtime-performance-digests",
            kind: BlockTimingBoundarySurfaceKind::RuntimeReceipt,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt",
            runtime_anchor: "RuntimePerformanceSnapshot + RuntimePerformanceTraceReceipt",
            rationale:
                "Keeps consumer and automation timing evidence aligned to the same runtime-owned measurement seam instead of private tracing hooks.",
        },
        BlockTimingBoundarySurface {
            id: "shared-host-block-timing-report",
            kind: BlockTimingBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned block timing and pressure truth without host-local callback reinterpretation.",
        },
    ]
}

pub(super) fn block_timing_boundary_validation_steps(
) -> &'static [BlockTimingBoundaryValidationStep] {
    &[
        BlockTimingBoundaryValidationStep {
            id: "runtime-public-block-timing-proof",
            command:
                "cargo test -p signal-runtime public_runtime_block_timing_boundary_reports_bounded_runtime_measurements",
            rationale:
                "Proves a downstream-style runtime consumer can inspect block timing, deadline pressure, and performance digests through public reexports.",
        },
        BlockTimingBoundaryValidationStep {
            id: "local-host-block-timing-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_block_timing_truth",
            rationale:
                "Proves the local shared host edge forwards the same block timing and pressure truth on supervisor export without private tracing hooks.",
        },
        BlockTimingBoundaryValidationStep {
            id: "server-host-block-timing-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_block_timing_truth",
            rationale:
                "Proves the server shared host edge forwards the same block timing and pressure truth on supervisor export without server-local reinterpretation.",
        },
        BlockTimingBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-block-timing-boundary --format=json",
            rationale:
                "Lets downstream consumers inspect the bounded block timing proof boundary without reading private runtime or host implementation detail.",
        },
    ]
}

pub(super) fn deferred_work_policy_boundary_surfaces(
) -> &'static [DeferredWorkPolicyBoundarySurface] {
    &[
        DeferredWorkPolicyBoundarySurface {
            id: "runtime-deferred-service-policy-receipt",
            kind: DeferredWorkPolicyBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::last_deferred_service_receipt and RuntimeSupervisorReport::observation.last_deferred_service_receipt",
            runtime_anchor: "RuntimeDeferredServiceReceipt",
            rationale:
                "Carries runtime-owned priority, blocking-priority, backpressure, starvation, and cancellation meaning directly on the public observation boundary.",
        },
        DeferredWorkPolicyBoundarySurface {
            id: "runtime-performance-policy-digests",
            kind: DeferredWorkPolicyBoundarySurfaceKind::RuntimeReceipt,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt",
            runtime_anchor: "RuntimePerformanceSnapshot + RuntimePerformanceTraceReceipt",
            rationale:
                "Keeps latest and peak deferred-work scheduler-policy evidence aligned to the same runtime-owned timing and hotspot digests.",
        },
        DeferredWorkPolicyBoundarySurface {
            id: "shared-host-deferred-policy-report",
            kind: DeferredWorkPolicyBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward deferred-work scheduler-policy truth without private queue helpers or host-local reclassification.",
        },
    ]
}

pub(super) fn deferred_work_policy_boundary_validation_steps(
) -> &'static [DeferredWorkPolicyBoundaryValidationStep] {
    &[
        DeferredWorkPolicyBoundaryValidationStep {
            id: "runtime-public-deferred-policy-proof",
            command:
                "cargo test -p signal-runtime public_runtime_deferred_work_policy_boundary_reports_runtime_owned_scheduler_receipts",
            rationale:
                "Proves a downstream-style runtime consumer can inspect defer, abort, starvation, backpressure, cancellation, and trace evidence through public reexports.",
        },
        DeferredWorkPolicyBoundaryValidationStep {
            id: "local-host-deferred-policy-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_deferred_work_policy_truth",
            rationale:
                "Proves the local shared host edge forwards deferred-work scheduler-policy truth on supervisor export without private queue helpers.",
        },
        DeferredWorkPolicyBoundaryValidationStep {
            id: "server-host-deferred-policy-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_deferred_work_policy_truth",
            rationale:
                "Proves the server shared host edge forwards deferred-work scheduler-policy truth on supervisor export without server-local policy forks.",
        },
        DeferredWorkPolicyBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-deferred-work-policy-boundary --format=json",
            rationale:
                "Lets downstream consumers inspect the deferred-work policy boundary, proof commands, and deferred scope without reading private runtime or host implementation detail.",
        },
    ]
}
