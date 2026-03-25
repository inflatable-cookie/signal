#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum InterruptionBoundarySurfaceKind {
    RuntimeReport,
    ContinuityReceipt,
    HostEdge,
}

impl InterruptionBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::ContinuityReceipt => "continuity-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct InterruptionBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: InterruptionBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct InterruptionBoundaryValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecordingContinuityBoundarySurfaceKind {
    RuntimeReceipt,
    RuntimeReport,
    HostEdge,
}

impl RecordingContinuityBoundarySurfaceKind {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::RuntimeReceipt => "runtime-receipt",
            Self::RuntimeReport => "runtime-report",
            Self::HostEdge => "host-edge",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordingContinuityBoundarySurface {
    pub(super) id: &'static str,
    pub(super) kind: RecordingContinuityBoundarySurfaceKind,
    pub(super) crate_name: &'static str,
    pub(super) surface: &'static str,
    pub(super) runtime_anchor: &'static str,
    pub(super) rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct RecordingContinuityValidationStep {
    pub(super) id: &'static str,
    pub(super) command: &'static str,
    pub(super) rationale: &'static str,
}

pub(super) fn interruption_boundary_surfaces() -> &'static [InterruptionBoundarySurface] {
    &[
        InterruptionBoundarySurface {
            id: "runtime-fault-status",
            kind: InterruptionBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationReport::fault_status and RuntimeSupervisorReport::observation.fault_status",
            runtime_anchor: "RuntimeFaultStatusSnapshot",
            rationale:
                "Carries the runtime-owned recovery-state and primary-fault classification without host-local inference.",
        },
        InterruptionBoundarySurface {
            id: "runtime-interruption-summary",
            kind: InterruptionBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationReport::interruption_summary and RuntimeSupervisorReport::observation.interruption_summary",
            runtime_anchor: "RuntimeInterruptionSummary",
            rationale:
                "Carries the shared interruption taxonomy directly on the public observation and supervisor boundary.",
        },
        InterruptionBoundarySurface {
            id: "deferred-service-interruption-receipt",
            kind: InterruptionBoundarySurfaceKind::ContinuityReceipt,
            crate_name: "signal-runtime",
            surface: "RuntimeDeferredServiceReceipt::interruption_class",
            runtime_anchor: "RuntimeDeferredServiceReceipt",
            rationale:
                "Keeps deferred-work resumability and terminal abort semantics typed instead of implied by queue policy prose.",
        },
        InterruptionBoundarySurface {
            id: "offline-render-execution-interruption-receipt",
            kind: InterruptionBoundarySurfaceKind::ContinuityReceipt,
            crate_name: "signal-runtime",
            surface: "RuntimeOfflineRenderExecutionProgressReceipt::interruption_class",
            runtime_anchor: "RuntimeOfflineRenderExecutionProgressReceipt",
            rationale:
                "Keeps paused, recoverable, and completed offline execution continuity visible through the same interruption vocabulary.",
        },
        InterruptionBoundarySurface {
            id: "shared-host-supervisor-report",
            kind: InterruptionBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges expose runtime-owned interruption meaning without their own recovery taxonomy.",
        },
    ]
}

pub(super) fn interruption_boundary_validation_steps(
) -> &'static [InterruptionBoundaryValidationStep] {
    &[
        InterruptionBoundaryValidationStep {
            id: "runtime-restartable-proof",
            command:
                "cargo test -p signal-runtime public_runtime_interruption_boundary_reports_restartable_runtime_state",
            rationale:
                "Proves a downstream-style runtime consumer can inspect a non-steady restartable interruption class through public reexports.",
        },
        InterruptionBoundaryValidationStep {
            id: "runtime-resumable-deferred-proof",
            command:
                "cargo test -p signal-runtime public_runtime_interruption_boundary_reports_resumable_deferred_state",
            rationale:
                "Proves resumable deferred-work continuity stays visible on public runtime receipts and observation export.",
        },
        InterruptionBoundaryValidationStep {
            id: "local-host-edge-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_is_consumable_without_private_helpers",
            rationale:
                "Proves the local shared host edge forwards interruption state through supervisor_report() without private helpers.",
        },
        InterruptionBoundaryValidationStep {
            id: "server-host-edge-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_is_consumable_without_private_helpers",
            rationale:
                "Proves the server shared host edge forwards interruption state through supervisor_report() without private helpers.",
        },
        InterruptionBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-interruption-boundary --format=json",
            rationale:
                "Lets consumers inspect the runtime and host-edge interruption proof boundary without reading private implementation detail.",
        },
    ]
}

pub(super) fn recording_continuity_boundary_surfaces(
) -> &'static [RecordingContinuityBoundarySurface] {
    &[
        RecordingContinuityBoundarySurface {
            id: "runtime-recording-capture-snapshot",
            kind: RecordingContinuityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::recording_capture_snapshot and RuntimeSupervisorReport::observation.recording_capture_snapshot",
            runtime_anchor: "RuntimeRecordingCaptureSnapshot",
            rationale:
                "Carries the runtime-owned capture identity, typed checkpoints, and continuity class directly on the public observation boundary.",
        },
        RecordingContinuityBoundarySurface {
            id: "runtime-recording-capture-commit-receipt",
            kind: RecordingContinuityBoundarySurfaceKind::RuntimeReceipt,
            crate_name: "signal-runtime",
            surface: "RuntimeRecordingCaptureCommitReceipt::committed_checkpoint",
            runtime_anchor: "RuntimeRecordingCaptureCommitReceipt",
            rationale:
                "Keeps committed capture evidence tied to the same runtime-owned checkpoint family instead of leaving commit continuity implicit.",
        },
        RecordingContinuityBoundarySurface {
            id: "shared-host-recording-supervisor-report",
            kind: RecordingContinuityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures stable host edges expose resumable, restartable, and terminal capture truth without host-local recovery policy.",
        },
    ]
}

pub(super) fn recording_continuity_validation_steps() -> &'static [RecordingContinuityValidationStep]
{
    &[
        RecordingContinuityValidationStep {
            id: "runtime-resumable-capture-proof",
            command:
                "cargo test -p signal-runtime runtime_recording_capture_resumes_same_identity_after_safe_mode_clears",
            rationale:
                "Proves same-identity resumable capture state survives degraded runtime conditions and later commits under the same runtime-owned boundary.",
        },
        RecordingContinuityValidationStep {
            id: "runtime-restartable-capture-proof",
            command:
                "cargo test -p signal-runtime runtime_recording_capture_preserves_restartable_checkpoint_across_stop_and_reconfigure",
            rationale:
                "Proves restartable capture preserves buffered checkpoint truth across runtime stop or reconfigure instead of disappearing silently.",
        },
        RecordingContinuityValidationStep {
            id: "runtime-terminal-capture-proof",
            command:
                "cargo test -p signal-runtime runtime_recording_capture_reports_terminal_checkpoint_on_commit_failure",
            rationale:
                "Proves terminal capture failure is exported as a typed failed checkpoint rather than log-only error context.",
        },
        RecordingContinuityValidationStep {
            id: "runtime-public-boundary-proof",
            command:
                "cargo test -p signal-runtime public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states",
            rationale:
                "Proves a downstream-style runtime consumer can distinguish all three capture outcomes through public reexports.",
        },
        RecordingContinuityValidationStep {
            id: "local-host-recording-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_resumable_recording_checkpoint_truth",
            rationale:
                "Proves the local shared host edge preserves resumable recording checkpoint meaning on supervisor export.",
        },
        RecordingContinuityValidationStep {
            id: "server-host-recording-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_restartable_and_terminal_recording_checkpoint_truth",
            rationale:
                "Proves the server shared host edge preserves restartable and terminal recording checkpoint meaning on supervisor export.",
        },
        RecordingContinuityValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-recording-continuity-boundary --format=json",
            rationale:
                "Lets consumers inspect the recording continuity proof boundary without reading private runtime or host implementation detail.",
        },
    ]
}
