use std::env;

use signal_host_local::{LocalRuntimeHost, LocalRuntimeHostSummary};
use signal_host_server::{ServerRuntimeHost, ServerRuntimeHostSummary};
use signal_runtime::{
    RuntimeConfig, RuntimeProfilingReceipt, RuntimeSoakReceipt, RuntimeSupervisorReport,
    SignalRuntime,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostProfile {
    Local,
    Server,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Scenario {
    Default,
    Timeout,
    Crash,
    Heartbeat,
    Soak,
    Mixed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Text,
    Json,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HostSummaryDebugSection {
    Payload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CliMode {
    Run {
        profile: HostProfile,
        scenario: Scenario,
    },
    DescribeExport,
    DescribeConformanceMatrix,
    DescribeInterruptionBoundary,
    DescribeFaultDiagnosticBoundary,
    DescribeCriticalPathBoundary,
    DescribeBlockTimingBoundary,
    DescribeDeferredWorkPolicyBoundary,
    DescribeRecordingContinuityBoundary,
    DescribeOfflineRenderContinuityBoundary,
    DescribePluginContinuityBoundary,
    DescribeVst3Boundary,
    DescribeAuBoundary,
    DescribeCrossAdapterParityBoundary,
    DescribeGenericEventBoundary,
    DescribeRecallPortabilityBoundary,
    DescribeDeviceSupervisionBoundary,
    DescribeClockTopologyBoundary,
    DescribeExternalIoBoundary,
    DescribeMediaServiceBoundary,
    DescribeAnalysisMetadataBoundary,
    DescribeIntegratedAcceptanceLane,
    DescribeG06SoakLane,
    DescribeHostEdgeBoundary,
    DescribeReleaseBoundary,
    DescribePackagingManifest,
    DescribeDownstreamAutomation,
    DescribeDownstreamFailGates,
    DescribeGenerationCloseout,
}

const EXPORT_SCHEMA: &str = "signal.supervisor.export";
const EXPORT_SCHEMA_VERSION: u32 = 1;
const DEFAULT_HOST_SUMMARY_SECTIONS: &[&str] = &["execution", "transport", "faults"];
const SUPPORTED_DEBUG_SECTIONS: &[HostSummaryDebugSection] = &[HostSummaryDebugSection::Payload];
const INTERRUPTION_BOUNDARY: &str = "signal.runtime.interruption-boundary";
const INTERRUPTION_CONTRACT_PATH: &str =
    "docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md";
const INTERRUPTION_ACCEPTANCE_TASK: &str = "effigy acceptance:interruption-boundary";
const FAULT_DIAGNOSTIC_BOUNDARY: &str = "signal.runtime.fault-diagnostic-boundary";
const FAULT_DIAGNOSTIC_CONTRACT_PATH: &str =
    "docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md";
const FAULT_DIAGNOSTIC_ACCEPTANCE_TASK: &str = "effigy acceptance:fault-diagnostic-boundary";
const CRITICAL_PATH_BOUNDARY: &str = "signal.runtime.critical-path-boundary";
const CRITICAL_PATH_CONTRACT_PATH: &str =
    "docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md";
const CRITICAL_PATH_ACCEPTANCE_TASK: &str = "effigy acceptance:critical-path-boundary";
const BLOCK_TIMING_BOUNDARY: &str = "signal.runtime.block-timing-boundary";
const BLOCK_TIMING_CONTRACT_PATH: &str =
    "docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md";
const BLOCK_TIMING_ACCEPTANCE_TASK: &str = "effigy acceptance:block-timing-boundary";
const DEFERRED_WORK_POLICY_BOUNDARY: &str = "signal.runtime.deferred-work-policy-boundary";
const DEFERRED_WORK_POLICY_CONTRACT_PATH: &str =
    "docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md";
const DEFERRED_WORK_POLICY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:deferred-work-policy-boundary";
const RECORDING_CONTINUITY_BOUNDARY: &str = "signal.runtime.recording-continuity-boundary";
const RECORDING_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md";
const RECORDING_CONTINUITY_ACCEPTANCE_TASK: &str = "effigy acceptance:recording-continuity";
const OFFLINE_RENDER_CONTINUITY_BOUNDARY: &str =
    "signal.runtime.offline-render-continuity-boundary";
const OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/015-offline-render-recovery-and-resumability-contract.md";
const OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:offline-render-continuity";
const PLUGIN_CONTINUITY_BOUNDARY: &str = "signal.runtime.plugin-continuity-boundary";
const PLUGIN_CONTINUITY_CONTRACT_PATH: &str =
    "docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md";
const PLUGIN_CONTINUITY_ACCEPTANCE_TASK: &str = "effigy acceptance:plugin-continuity";
const VST3_BOUNDARY: &str = "signal.runtime.vst3-boundary";
const VST3_CONTRACT_PATH: &str =
    "docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md";
const VST3_ACCEPTANCE_TASK: &str = "effigy acceptance:vst3-boundary";
const AU_BOUNDARY: &str = "signal.runtime.au-boundary";
const AU_CONTRACT_PATH: &str =
    "docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md";
const AU_ACCEPTANCE_TASK: &str = "effigy acceptance:au-boundary";
const CROSS_ADAPTER_PARITY_BOUNDARY: &str = "signal.runtime.cross-adapter-parity-boundary";
const CROSS_ADAPTER_PARITY_CONTRACT_PATH: &str =
    "docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md";
const CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK: &str =
    "effigy acceptance:cross-adapter-parity-boundary";
const GENERIC_EVENT_BOUNDARY: &str = "signal.runtime.generic-event-boundary";
const GENERIC_EVENT_CONTRACT_PATH: &str =
    "docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md";
const GENERIC_EVENT_ACCEPTANCE_TASK: &str = "effigy acceptance:generic-event-boundary";
const RECALL_PORTABILITY_BOUNDARY: &str = "signal.runtime.recall-portability-boundary";
const RECALL_PORTABILITY_CONTRACT_PATH: &str =
    "docs/contracts/024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md";
const RECALL_PORTABILITY_ACCEPTANCE_TASK: &str = "effigy acceptance:recall-portability-boundary";
const DEVICE_SUPERVISION_BOUNDARY: &str = "signal.runtime.device-supervision-boundary";
const DEVICE_SUPERVISION_CONTRACT_PATH: &str =
    "docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md";
const DEVICE_SUPERVISION_ACCEPTANCE_TASK: &str = "effigy acceptance:device-supervision-boundary";
const CLOCK_TOPOLOGY_BOUNDARY: &str = "signal.runtime.clock-topology-boundary";
const CLOCK_TOPOLOGY_CONTRACT_PATH: &str =
    "docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md";
const CLOCK_TOPOLOGY_ACCEPTANCE_TASK: &str = "effigy acceptance:clock-topology-boundary";
const EXTERNAL_IO_BOUNDARY: &str = "signal.runtime.external-io-boundary";
const EXTERNAL_IO_CONTRACT_PATH: &str =
    "docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md";
const EXTERNAL_IO_ACCEPTANCE_TASK: &str = "effigy acceptance:external-io-boundary";
const MEDIA_SERVICE_BOUNDARY: &str = "signal.runtime.media-service-boundary";
const MEDIA_SERVICE_CONTRACT_PATH: &str =
    "docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md";
const MEDIA_SERVICE_ACCEPTANCE_TASK: &str = "effigy acceptance:media-service-boundary";
const ANALYSIS_METADATA_BOUNDARY: &str = "signal.runtime.analysis-metadata-boundary";
const ANALYSIS_METADATA_CONTRACT_PATH: &str =
    "docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md";
const ANALYSIS_METADATA_ACCEPTANCE_TASK: &str = "effigy acceptance:analysis-metadata-boundary";
const INTEGRATED_ACCEPTANCE_LANE: &str = "signal.runtime.integrated-acceptance-lane";
const INTEGRATED_ACCEPTANCE_CONTRACT_PATH: &str =
    "docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md";
const INTEGRATED_ACCEPTANCE_TASK: &str = "effigy acceptance:integrated-acceptance-lane";
const G06_SOAK_LANE: &str = "signal.g06.long-session-soak-lane";
const G06_SOAK_CONTRACT_PATH: &str =
    "docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md";
const G06_SOAK_ACCEPTANCE_TASK: &str = "effigy acceptance:g06-soak-lane";
const INTEGRATED_ACCEPTANCE_REQUIRED_TASKS: &[&str] = &[
    INTERRUPTION_ACCEPTANCE_TASK,
    FAULT_DIAGNOSTIC_ACCEPTANCE_TASK,
    CRITICAL_PATH_ACCEPTANCE_TASK,
    DEFERRED_WORK_POLICY_ACCEPTANCE_TASK,
    PLUGIN_CONTINUITY_ACCEPTANCE_TASK,
    CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK,
    DEVICE_SUPERVISION_ACCEPTANCE_TASK,
    CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
    EXTERNAL_IO_ACCEPTANCE_TASK,
    MEDIA_SERVICE_ACCEPTANCE_TASK,
    ANALYSIS_METADATA_ACCEPTANCE_TASK,
];
const INTEGRATED_ACCEPTANCE_ADVISORY_TASKS: &[&str] = &[
    RECORDING_CONTINUITY_ACCEPTANCE_TASK,
    OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK,
    VST3_ACCEPTANCE_TASK,
    AU_ACCEPTANCE_TASK,
    GENERIC_EVENT_ACCEPTANCE_TASK,
    RECALL_PORTABILITY_ACCEPTANCE_TASK,
];
const RECOVERY_AND_FAULT_REQUIRED_TASKS: &[&str] = &[
    INTERRUPTION_ACCEPTANCE_TASK,
    FAULT_DIAGNOSTIC_ACCEPTANCE_TASK,
    DEVICE_SUPERVISION_ACCEPTANCE_TASK,
];
const RECOVERY_AND_FAULT_ADVISORY_TASKS: &[&str] = &[
    RECORDING_CONTINUITY_ACCEPTANCE_TASK,
    OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK,
];
const SCHEDULING_AND_PRESSURE_REQUIRED_TASKS: &[&str] = &[
    CRITICAL_PATH_ACCEPTANCE_TASK,
    DEFERRED_WORK_POLICY_ACCEPTANCE_TASK,
];
const SCHEDULING_AND_PRESSURE_ADVISORY_TASKS: &[&str] = &[BLOCK_TIMING_ACCEPTANCE_TASK];
const ADAPTER_AND_PORTABILITY_REQUIRED_TASKS: &[&str] = &[
    PLUGIN_CONTINUITY_ACCEPTANCE_TASK,
    CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK,
];
const ADAPTER_AND_PORTABILITY_ADVISORY_TASKS: &[&str] = &[
    VST3_ACCEPTANCE_TASK,
    AU_ACCEPTANCE_TASK,
    GENERIC_EVENT_ACCEPTANCE_TASK,
    RECALL_PORTABILITY_ACCEPTANCE_TASK,
];
const HARDWARE_AND_EXTERNAL_IO_REQUIRED_TASKS: &[&str] = &[
    DEVICE_SUPERVISION_ACCEPTANCE_TASK,
    CLOCK_TOPOLOGY_ACCEPTANCE_TASK,
    EXTERNAL_IO_ACCEPTANCE_TASK,
];
const HARDWARE_AND_EXTERNAL_IO_ADVISORY_TASKS: &[&str] = &[];
const MEDIA_AND_LIBRARY_REQUIRED_TASKS: &[&str] = &[
    MEDIA_SERVICE_ACCEPTANCE_TASK,
    ANALYSIS_METADATA_ACCEPTANCE_TASK,
];
const MEDIA_AND_LIBRARY_ADVISORY_TASKS: &[&str] = &[];
const HOST_EDGE_BOUNDARY: &str = "signal.host.edge.boundary";
const HOST_EDGE_CONTRACT_PATH: &str =
    "docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md";
const HOST_EDGE_ACCEPTANCE_TASK: &str = "effigy acceptance:host-edge-consumer";
const RELEASE_BOUNDARY: &str = "signal.release.boundary";
const RELEASE_VERSION_SOURCE: &str = "workspace.package.version";
const RELEASE_CHANGELOG_PATH: &str = "CHANGELOG.md";
const RELEASE_CONFORMANCE_TASK: &str = "effigy acceptance:conformance";
const PACKAGING_MANIFEST: &str = "signal.release.packaging-manifest";
const PACKAGING_MANIFEST_CONTRACT_PATH: &str =
    "docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md";
const PACKAGING_MANIFEST_ACCEPTANCE_TASK: &str = "effigy acceptance:release-packaging-consumer";
const DOWNSTREAM_AUTOMATION_BOUNDARY: &str = "signal.downstream.automation";
const DOWNSTREAM_AUTOMATION_CONTRACT_PATH: &str =
    "docs/contracts/011-shared-downstream-conformance-and-release-acceptance-automation-contract.md";
const DOWNSTREAM_AUTOMATION_MANDATORY_TASK: &str = "effigy acceptance:downstream-release";
const DOWNSTREAM_AUTOMATION_OPTIONAL_TASK: &str = "effigy acceptance:downstream-depth";
const DOWNSTREAM_AUTOMATION_COMBINED_TASK: &str = "effigy acceptance:downstream-automation";
const DOWNSTREAM_FAIL_GATES: &str = "signal.downstream.fail-gates";
const DOWNSTREAM_FAIL_GATE_TASK: &str = "effigy acceptance:downstream-gate";
const GENERATION_CLOSEOUT: &str = "signal.generation.closeout";
const GENERATION_CLOSEOUT_GENERATION: &str = "g06";
const GENERATION_CLOSEOUT_TASK: &str = "effigy acceptance:g06-closeout";
const GENERATION_CLOSEOUT_CONTRACT_PATH: &str =
    "docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md";
const G07_README_PATH: &str = "docs/roadmaps/g07/README.md";
const GENERATION_CLOSEOUT_NEXT_QUEUE_STATUS: &str = "promoted-g07-active";
const GENERATION_CLOSEOUT_PROMOTION_DECISION: &str = "promote-g07";
const GENERATION_CLOSEOUT_NEXT_GENERATION_STATUS: &str = "active";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ConformanceMatrixEntryKind {
    PublicBoundaryTest,
    ExportConsumerTest,
    Example,
    Introspection,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ConformanceMatrixEntry {
    id: &'static str,
    kind: ConformanceMatrixEntryKind,
    crate_name: &'static str,
    surface: &'static str,
    command: &'static str,
    rationale: &'static str,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InterruptionBoundarySurfaceKind {
    RuntimeReport,
    ContinuityReceipt,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InterruptionBoundarySurface {
    id: &'static str,
    kind: InterruptionBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InterruptionBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FaultDiagnosticBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FaultDiagnosticBoundarySurface {
    id: &'static str,
    kind: FaultDiagnosticBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FaultDiagnosticBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CriticalPathBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriticalPathBoundarySurface {
    id: &'static str,
    kind: CriticalPathBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CriticalPathBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockTimingBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlockTimingBoundarySurface {
    id: &'static str,
    kind: BlockTimingBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct BlockTimingBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeferredWorkPolicyBoundarySurfaceKind {
    RuntimeReport,
    RuntimeReceipt,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeferredWorkPolicyBoundarySurface {
    id: &'static str,
    kind: DeferredWorkPolicyBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeferredWorkPolicyBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecordingContinuityBoundarySurfaceKind {
    RuntimeReceipt,
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordingContinuityBoundarySurface {
    id: &'static str,
    kind: RecordingContinuityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecordingContinuityValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OfflineRenderContinuityBoundarySurfaceKind {
    RuntimeSnapshot,
    RuntimeReport,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OfflineRenderContinuityBoundarySurface {
    id: &'static str,
    kind: OfflineRenderContinuityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct OfflineRenderContinuityValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Vst3BoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Vst3BoundarySurface {
    id: &'static str,
    kind: Vst3BoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Vst3BoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AuBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AuBoundarySurface {
    id: &'static str,
    kind: AuBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AuBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CrossAdapterParityBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CrossAdapterParityBoundarySurface {
    id: &'static str,
    kind: CrossAdapterParityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CrossAdapterParityBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GenericEventBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenericEventBoundarySurface {
    id: &'static str,
    kind: GenericEventBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenericEventBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecallPortabilityBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecallPortabilityBoundarySurface {
    id: &'static str,
    kind: RecallPortabilityBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RecallPortabilityBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DeviceSupervisionBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceSupervisionBoundarySurface {
    id: &'static str,
    kind: DeviceSupervisionBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DeviceSupervisionBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClockTopologyBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClockTopologyBoundarySurface {
    id: &'static str,
    kind: ClockTopologyBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ClockTopologyBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExternalIoBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExternalIoBoundarySurface {
    id: &'static str,
    kind: ExternalIoBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ExternalIoBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MediaServiceBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MediaServiceBoundarySurface {
    id: &'static str,
    kind: MediaServiceBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MediaServiceBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnalysisMetadataBoundarySurfaceKind {
    RuntimeReport,
    RuntimeSnapshot,
    HostEdge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AnalysisMetadataBoundarySurface {
    id: &'static str,
    kind: AnalysisMetadataBoundarySurfaceKind,
    crate_name: &'static str,
    surface: &'static str,
    runtime_anchor: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct AnalysisMetadataBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntegratedAcceptanceFamily {
    id: &'static str,
    title: &'static str,
    required_tasks: &'static [&'static str],
    advisory_tasks: &'static [&'static str],
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct IntegratedAcceptanceValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct G06SoakLaneScenarioRecord {
    id: &'static str,
    status: &'static str,
    command: &'static str,
    typed_output: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct G06SoakLaneValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenerationReadinessArea {
    id: &'static str,
    status: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ReleaseBoundaryArtifactKind {
    Document,
    ExportDescription,
    ConformanceMatrix,
    PackagingManifest,
    Example,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReleaseBoundaryArtifact {
    id: &'static str,
    kind: ReleaseBoundaryArtifactKind,
    path_or_command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ReleaseBoundaryValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PackagingManifestInputKind {
    Document,
    Descriptor,
    ValidationTask,
    Contract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackagingManifestInput {
    id: &'static str,
    kind: PackagingManifestInputKind,
    path_or_command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackagingReceiptSurface {
    id: &'static str,
    surface: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PackagingManifestValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DownstreamAutomationFixtureKind {
    AcceptanceTask,
    Descriptor,
    ScenarioExport,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DownstreamAutomationFixture {
    id: &'static str,
    kind: DownstreamAutomationFixtureKind,
    command: &'static str,
    typed_output: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DownstreamFailGateRule {
    id: &'static str,
    gate: &'static str,
    command: &'static str,
    blocks_release: bool,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DownstreamDeferredDepthRecord {
    id: &'static str,
    command: &'static str,
    status: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct GenerationCloseoutValidationStep {
    id: &'static str,
    command: &'static str,
    rationale: &'static str,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ExportDebugOptions {
    payload: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CliArgs {
    format: OutputFormat,
    debug: ExportDebugOptions,
    mode: CliMode,
}

impl HostProfile {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "local" => Ok(Self::Local),
            "server" => Ok(Self::Server),
            _ => Err(format!(
                "unknown profile {value:?}; expected one of: local, server"
            )),
        }
    }
}

impl Scenario {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "default" => Ok(Self::Default),
            "timeout" => Ok(Self::Timeout),
            "crash" => Ok(Self::Crash),
            "heartbeat" => Ok(Self::Heartbeat),
            "soak" => Ok(Self::Soak),
            "mixed" => Ok(Self::Mixed),
            _ => Err(format!(
                "unknown scenario {value:?}; expected one of: default, timeout, crash, heartbeat, soak, mixed"
            )),
        }
    }
}

impl OutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "text" => Ok(Self::Text),
            "json" => Ok(Self::Json),
            _ => Err(format!(
                "unknown format {value:?}; expected one of: text, json"
            )),
        }
    }
}

fn print_usage() {
    eprintln!(
        "usage: signal-supervisor-tools [--format text|json] [--include-payload] [--describe-export|--describe-conformance-matrix|--describe-interruption-boundary|--describe-fault-diagnostic-boundary|--describe-critical-path-boundary|--describe-block-timing-boundary|--describe-deferred-work-policy-boundary|--describe-recording-continuity-boundary|--describe-offline-render-continuity-boundary|--describe-plugin-continuity-boundary|--describe-vst3-boundary|--describe-au-boundary|--describe-cross-adapter-parity-boundary|--describe-generic-event-boundary|--describe-recall-portability-boundary|--describe-device-supervision-boundary|--describe-clock-topology-boundary|--describe-external-io-boundary|--describe-media-service-boundary|--describe-analysis-metadata-boundary|--describe-integrated-acceptance-lane|--describe-g06-soak-lane|--describe-host-edge-boundary|--describe-release-boundary|--describe-packaging-manifest|--describe-downstream-automation|--describe-downstream-fail-gates|--describe-generation-closeout] <local|server> <default|timeout|crash|heartbeat|soak|mixed>"
    );
}

impl HostSummaryDebugSection {
    fn label(self) -> &'static str {
        match self {
            Self::Payload => "payload",
        }
    }
}

impl ExportDebugOptions {
    fn supports(self, section: HostSummaryDebugSection) -> bool {
        match section {
            HostSummaryDebugSection::Payload => self.payload,
        }
    }
}

impl ConformanceMatrixEntryKind {
    fn label(self) -> &'static str {
        match self {
            Self::PublicBoundaryTest => "public-boundary-test",
            Self::ExportConsumerTest => "export-consumer-test",
            Self::Example => "example",
            Self::Introspection => "introspection",
        }
    }
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

impl InterruptionBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::ContinuityReceipt => "continuity-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

impl FaultDiagnosticBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

impl CriticalPathBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

impl BlockTimingBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
}

impl DeferredWorkPolicyBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeReceipt => "runtime-receipt",
            Self::HostEdge => "host-edge",
        }
    }
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

impl Vst3BoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl AuBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl CrossAdapterParityBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl GenericEventBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl RecallPortabilityBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl DeviceSupervisionBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl ClockTopologyBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl ExternalIoBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl MediaServiceBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl AnalysisMetadataBoundarySurfaceKind {
    fn label(self) -> &'static str {
        match self {
            Self::RuntimeReport => "runtime-report",
            Self::RuntimeSnapshot => "runtime-snapshot",
            Self::HostEdge => "host-edge",
        }
    }
}

impl ReleaseBoundaryArtifactKind {
    fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::ExportDescription => "export-description",
            Self::ConformanceMatrix => "conformance-matrix",
            Self::PackagingManifest => "packaging-manifest",
            Self::Example => "example",
        }
    }
}

impl PackagingManifestInputKind {
    fn label(self) -> &'static str {
        match self {
            Self::Document => "document",
            Self::Descriptor => "descriptor",
            Self::ValidationTask => "validation-task",
            Self::Contract => "contract",
        }
    }
}

impl DownstreamAutomationFixtureKind {
    fn label(self) -> &'static str {
        match self {
            Self::AcceptanceTask => "acceptance-task",
            Self::Descriptor => "descriptor",
            Self::ScenarioExport => "scenario-export",
        }
    }
}

fn conformance_matrix_entries() -> &'static [ConformanceMatrixEntry] {
    &[
        ConformanceMatrixEntry {
            id: "runtime-public-contract-boundary",
            kind: ConformanceMatrixEntryKind::PublicBoundaryTest,
            crate_name: "signal-runtime",
            surface: "SignalRuntime, RuntimeObservationReport, RuntimeSupervisorReport public reexports",
            command:
                "cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports",
            rationale:
                "Proves a downstream-style consumer can capture runtime/export/plugin receipts without private internals.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-discovery-consumer",
            kind: ConformanceMatrixEntryKind::ExportConsumerTest,
            crate_name: "signal-supervisor-tools",
            surface: "signal.supervisor.export JSON carrying runtime-owned plugin discovery catalog",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_runtime_owned_plugin_discovery_catalog",
            rationale:
                "Proves the versioned supervisor export carries the widened discovery boundary without host-local reconstruction.",
        },
        ConformanceMatrixEntry {
            id: "plugin-backend-breadth-coverage",
            kind: ConformanceMatrixEntryKind::ExportConsumerTest,
            crate_name: "signal-runtime + signal-supervisor-tools",
            surface: "runtime reexports and supervisor export carrying backend-neutral plugin discovery coverage aggregates",
            command: "effigy acceptance:plugin-backend-breadth",
            rationale:
                "Proves widened multi-format discovery and capability coverage stays consumable through Signal-owned runtime and export surfaces.",
        },
        ConformanceMatrixEntry {
            id: "shared-host-edge-consumer",
            kind: ConformanceMatrixEntryKind::PublicBoundaryTest,
            crate_name: "signal-host-local + signal-host-server",
            surface: "shared-stable host constructors, RuntimeSupervisorApi, and supervisor_report()",
            command: "effigy acceptance:host-edge-consumer",
            rationale:
                "Proves the shared stable host edge remains consumable without private host internals or unstable summary helpers.",
        },
        ConformanceMatrixEntry {
            id: "runtime-supervisor-report-demo",
            kind: ConformanceMatrixEntryKind::Example,
            crate_name: "signal-runtime",
            surface: "supervisor_report_demo example",
            command: "cargo run -p signal-runtime --example supervisor_report_demo",
            rationale:
                "Provides a host-free runnable example that emits the stabilized supervisor report surface.",
        },
        ConformanceMatrixEntry {
            id: "supervisor-export-schema-description",
            kind: ConformanceMatrixEntryKind::Introspection,
            crate_name: "signal-supervisor-tools",
            surface: "signal-supervisor-tools export/conformance schema description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "Lets consumers inspect the runnable conformance matrix without reading private implementation detail.",
        },
    ]
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
            runtime_anchor: "RuntimeObservationReport, RuntimeHostObservationReport, RuntimeHostSupervisorReport",
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
            surface: "finalize_offline_render_with_local_delegated_executor() and render_offline_with_local_delegated_executor()",
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

fn interruption_boundary_surfaces() -> &'static [InterruptionBoundarySurface] {
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

fn interruption_boundary_validation_steps() -> &'static [InterruptionBoundaryValidationStep] {
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

fn fault_diagnostic_boundary_surfaces() -> &'static [FaultDiagnosticBoundarySurface] {
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

fn fault_diagnostic_boundary_validation_steps() -> &'static [FaultDiagnosticBoundaryValidationStep]
{
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

fn critical_path_boundary_surfaces() -> &'static [CriticalPathBoundarySurface] {
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

fn critical_path_boundary_validation_steps() -> &'static [CriticalPathBoundaryValidationStep] {
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

fn block_timing_boundary_surfaces() -> &'static [BlockTimingBoundarySurface] {
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

fn block_timing_boundary_validation_steps() -> &'static [BlockTimingBoundaryValidationStep] {
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

fn deferred_work_policy_boundary_surfaces() -> &'static [DeferredWorkPolicyBoundarySurface] {
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

fn deferred_work_policy_boundary_validation_steps(
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

fn recording_continuity_boundary_surfaces() -> &'static [RecordingContinuityBoundarySurface] {
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

fn recording_continuity_validation_steps() -> &'static [RecordingContinuityValidationStep] {
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

fn offline_render_continuity_boundary_surfaces() -> &'static [OfflineRenderContinuityBoundarySurface]
{
    &[
        OfflineRenderContinuityBoundarySurface {
            id: "runtime-offline-render-session-snapshot",
            kind: OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::offline_render_session_snapshot and RuntimeSupervisorReport::observation.offline_render_session_snapshot",
            runtime_anchor: "RuntimeOfflineRenderSessionSnapshot",
            rationale:
                "Carries active and last render-session continuity, checkpoints, cancellation, and purge truth directly on public runtime reports.",
        },
        OfflineRenderContinuityBoundarySurface {
            id: "runtime-offline-render-observation-api",
            kind: OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_offline_render_session_snapshot()",
            runtime_anchor: "RuntimeOfflineRenderSessionSnapshot",
            rationale:
                "Keeps render continuity inspectable without forcing consumers through filesystem artifacts or supervisor-only JSON parsing.",
        },
        OfflineRenderContinuityBoundarySurface {
            id: "shared-host-offline-render-supervisor-report",
            kind: OfflineRenderContinuityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures stable host edges forward resumable, restartable, and terminal render-session truth without host-local retry policy.",
        },
    ]
}

fn offline_render_continuity_validation_steps() -> &'static [OfflineRenderContinuityValidationStep]
{
    &[
        OfflineRenderContinuityValidationStep {
            id: "runtime-resumable-render-proof",
            command:
                "cargo test -p signal-runtime runtime_offline_render_session_snapshot_preserves_checkpoint_through_pause_and_recoverable_states",
            rationale:
                "Proves checkpoints survive pause and recoverable interruption under the same runtime-owned render-session identity.",
        },
        OfflineRenderContinuityValidationStep {
            id: "runtime-restartable-render-proof",
            command:
                "cargo test -p signal-runtime runtime_offline_render_session_snapshot_reports_restartable_state_across_stop_restart_and_resume",
            rationale:
                "Proves runtime stop and restart preserve render-session continuity as a restartable path instead of silently dropping active work.",
        },
        OfflineRenderContinuityValidationStep {
            id: "runtime-terminal-render-proof",
            command:
                "cargo test -p signal-runtime runtime_offline_render_session_snapshot_reports_failed_terminal_state_on_delivery_error",
            rationale:
                "Proves failed render delivery is exported as typed terminal session state rather than disappearing into a raw I/O error.",
        },
        OfflineRenderContinuityValidationStep {
            id: "runtime-public-boundary-proof",
            command:
                "cargo test -p signal-runtime public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states",
            rationale:
                "Proves a downstream-style runtime consumer can distinguish all three render continuity outcomes through public reexports.",
        },
        OfflineRenderContinuityValidationStep {
            id: "local-host-render-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_resumable_offline_render_session_truth",
            rationale:
                "Proves the local shared host edge preserves resumable render-session truth on supervisor export.",
        },
        OfflineRenderContinuityValidationStep {
            id: "server-host-render-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_restartable_and_terminal_offline_render_session_truth",
            rationale:
                "Proves the server shared host edge preserves restartable and terminal render-session truth on supervisor export.",
        },
        OfflineRenderContinuityValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-offline-render-continuity-boundary --format=json",
            rationale:
                "Lets consumers inspect the render continuity proof boundary without reading private runtime or host implementation detail.",
        },
    ]
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

fn vst3_boundary_surfaces() -> &'static [Vst3BoundarySurface] {
    &[
        Vst3BoundarySurface {
            id: "runtime-vst3-discovery-report",
            kind: Vst3BoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor: "RuntimePluginDiscoverySnapshot",
            rationale:
                "Keeps discovered VST3 types and format-filtered scan intent consumable through shared runtime reports rather than adapter-private catalogs.",
        },
        Vst3BoundarySurface {
            id: "runtime-vst3-lifecycle-snapshot",
            kind: Vst3BoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps VST3 sandbox lifecycle, readiness, and transport attachment truth on the existing runtime-owned lifecycle seam instead of a format-specific lifecycle shell.",
        },
        Vst3BoundarySurface {
            id: "shared-host-vst3-supervisor-report",
            kind: Vst3BoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward VST3 discovery and lifecycle truth without adapter-local reconstruction or host-private VST3 ledgers.",
        },
    ]
}

fn vst3_boundary_validation_steps() -> &'static [Vst3BoundaryValidationStep] {
    &[
        Vst3BoundaryValidationStep {
            id: "runtime-vst3-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect VST3 discovery and lifecycle truth through public runtime reexports alone.",
        },
        Vst3BoundaryValidationStep {
            id: "local-host-vst3-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_vst3_baseline_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned VST3 discovery and lifecycle state on supervisor export.",
        },
        Vst3BoundaryValidationStep {
            id: "server-host-vst3-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_vst3_baseline_truth",
            rationale:
                "Proves the server stable host edge forwards Linux-rooted runtime-owned VST3 discovery and lifecycle state on supervisor export.",
        },
        Vst3BoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-vst3-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared VST3 proof boundary without reading host or adapter implementation detail.",
        },
    ]
}

fn au_boundary_surfaces() -> &'static [AuBoundarySurface] {
    &[
        AuBoundarySurface {
            id: "runtime-au-discovery-report",
            kind: AuBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor: "RuntimePluginDiscoverySnapshot",
            rationale:
                "Keeps discovered AU types and format-filtered scan intent consumable through shared runtime reports rather than adapter-private catalogs.",
        },
        AuBoundarySurface {
            id: "runtime-au-lifecycle-snapshot",
            kind: AuBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps AU sandbox lifecycle, readiness, and transport attachment truth on the existing runtime-owned lifecycle seam instead of a format-specific lifecycle shell.",
        },
        AuBoundarySurface {
            id: "shared-host-au-supervisor-report",
            kind: AuBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward AU discovery and lifecycle truth without adapter-local reconstruction or host-private AU ledgers.",
        },
    ]
}

fn au_boundary_validation_steps() -> &'static [AuBoundaryValidationStep] {
    &[
        AuBoundaryValidationStep {
            id: "runtime-au-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect AU discovery and lifecycle truth through public runtime reexports alone.",
        },
        AuBoundaryValidationStep {
            id: "local-host-au-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_au_baseline_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned AU discovery and lifecycle state on supervisor export.",
        },
        AuBoundaryValidationStep {
            id: "server-host-au-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_au_baseline_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned AU discovery and lifecycle state on supervisor export.",
        },
        AuBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-au-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared AU proof boundary without reading host or adapter implementation detail.",
        },
    ]
}

fn cross_adapter_parity_boundary_surfaces() -> &'static [CrossAdapterParityBoundarySurface] {
    &[
        CrossAdapterParityBoundarySurface {
            id: "runtime-cross-adapter-discovery-report",
            kind: CrossAdapterParityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot",
            runtime_anchor: "RuntimePluginDiscoverySnapshot",
            rationale:
                "Keeps CLAP, VST3, and AU parity bands plus supported and unsupported platform scope consumable through the shared discovery report seam instead of a host-local portability matrix.",
        },
        CrossAdapterParityBoundarySurface {
            id: "runtime-cross-adapter-lifecycle-snapshot",
            kind: CrossAdapterParityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_lifecycle_snapshot()",
            runtime_anchor: "RuntimePluginLifecycleSnapshot",
            rationale:
                "Keeps cross-adapter parity counts for ready, degraded, faulted, and active-transport sandbox state on the existing runtime-owned lifecycle seam.",
        },
        CrossAdapterParityBoundarySurface {
            id: "shared-host-cross-adapter-supervisor-report",
            kind: CrossAdapterParityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward one cross-adapter parity vocabulary without private host portability tables or adapter-specific reconstruction.",
        },
    ]
}

fn cross_adapter_parity_boundary_validation_steps(
) -> &'static [CrossAdapterParityBoundaryValidationStep] {
    &[
        CrossAdapterParityBoundaryValidationStep {
            id: "runtime-cross-adapter-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect CLAP, VST3, and AU parity coverage through public runtime reexports alone.",
        },
        CrossAdapterParityBoundaryValidationStep {
            id: "local-host-cross-adapter-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_cross_adapter_parity_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned cross-adapter parity coverage on supervisor export.",
        },
        CrossAdapterParityBoundaryValidationStep {
            id: "server-host-cross-adapter-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_cross_adapter_parity_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned cross-adapter parity coverage on supervisor export.",
        },
        CrossAdapterParityBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-cross-adapter-parity-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared cross-adapter parity proof boundary without reading private host code or adapter internals.",
        },
    ]
}

fn generic_event_boundary_surfaces() -> &'static [GenericEventBoundarySurface] {
    &[
        GenericEventBoundarySurface {
            id: "runtime-generic-event-report",
            kind: GenericEventBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot",
            runtime_anchor: "RuntimePluginEventSnapshot",
            rationale:
                "Keeps parameter, note, note-expression, and MIDI event continuity on one runtime-owned report seam instead of host-private payload counters.",
        },
        GenericEventBoundarySurface {
            id: "runtime-generic-event-capability-coverage",
            kind: GenericEventBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationApi::get_plugin_discovery_snapshot() capability_coverage.supports_note_expression_count",
            runtime_anchor: "RuntimePluginCapabilityCoverageSummary",
            rationale:
                "Keeps note-expression breadth on runtime-owned discovery receipts instead of adapter-local inference from MIDI or note support alone.",
        },
        GenericEventBoundarySurface {
            id: "shared-host-generic-event-supervisor-report",
            kind: GenericEventBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges expose the widened event and capability truth without CLAP, VST3, or AU packet reconstruction.",
        },
    ]
}

fn generic_event_boundary_validation_steps() -> &'static [GenericEventBoundaryValidationStep] {
    &[
        GenericEventBoundaryValidationStep {
            id: "runtime-generic-event-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect widened generic event and note-expression capability truth through public runtime reexports alone.",
        },
        GenericEventBoundaryValidationStep {
            id: "local-host-generic-event-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_generic_event_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned generic event and note-expression capability receipts on supervisor export.",
        },
        GenericEventBoundaryValidationStep {
            id: "server-host-generic-event-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_generic_event_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned generic event and note-expression capability receipts on supervisor export.",
        },
        GenericEventBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools generic_event_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable boundary descriptor aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        GenericEventBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-generic-event-boundary --format=json",
            rationale:
                "Lets consumers inspect the widened generic event proof boundary without reading private host code or adapter packet translation logic.",
        },
    ]
}

fn recall_portability_boundary_surfaces() -> &'static [RecallPortabilityBoundarySurface] {
    &[
        RecallPortabilityBoundarySurface {
            id: "runtime-plugin-chain-recall-report",
            kind: RecallPortabilityBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::plugin_chain_snapshot and RuntimeSupervisorReport::observation.plugin_chain_snapshot",
            runtime_anchor: "RuntimePluginRecallPayload",
            rationale:
                "Keeps portable versus guarded, native-only, context-only, and unsupported recall truth on the shared plugin-chain report seam instead of adapter-native preset heuristics.",
        },
        RecallPortabilityBoundarySurface {
            id: "runtime-plugin-recall-handoff",
            kind: RecallPortabilityBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_plugin_recall_handoff_snapshot()",
            runtime_anchor: "RuntimePluginRecallHandoffSnapshot",
            rationale:
                "Lets offline, export, and downstream consumers inspect widened preset descriptor and bounded ARA-context transfer on a runtime-owned handoff snapshot instead of host-local blob planning.",
        },
        RecallPortabilityBoundarySurface {
            id: "shared-host-recall-supervisor-report",
            kind: RecallPortabilityBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward the same runtime-owned portability and ARA-context truth without adapter-local preset reconstruction or host-owned portability classes.",
        },
    ]
}

fn recall_portability_boundary_validation_steps(
) -> &'static [RecallPortabilityBoundaryValidationStep] {
    &[
        RecallPortabilityBoundaryValidationStep {
            id: "runtime-recall-portability-public-proof",
            command:
                "cargo test -p signal-runtime --test public_contract_boundary public_runtime_recall_interchange_and_ara_context_truth_is_consumable_from_reexports",
            rationale:
                "Proves a downstream-style runtime consumer can inspect portable versus non-portable recall outcomes and bounded ARA-context transfer through public runtime reexports alone.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "local-host-recall-portability-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_boundary local_shared_host_edge_exports_runtime_recall_portability_truth",
            rationale:
                "Proves the local stable host edge forwards runtime-owned recall portability and ARA-context truth on supervisor export.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "server-host-recall-portability-proof",
            command:
                "cargo test -p signal-host-server --test public_host_edge_boundary server_shared_host_edge_exports_runtime_recall_portability_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned recall portability and bounded ARA-context transfer on supervisor export.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools recall_portability_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable recall portability descriptor aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        RecallPortabilityBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-recall-portability-boundary --format=json",
            rationale:
                "Lets consumers inspect portable versus native-only recall outcomes and bounded ARA-context transfer without reading private host glue or adapter-native preset parsing code.",
        },
    ]
}

fn device_supervision_boundary_surfaces() -> &'static [DeviceSupervisionBoundarySurface] {
    &[
        DeviceSupervisionBoundarySurface {
            id: "runtime-device-supervision-report",
            kind: DeviceSupervisionBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::device_supervision_snapshot and RuntimeSupervisorReport::observation.device_supervision_snapshot",
            runtime_anchor: "RuntimeDeviceSupervisionSnapshot",
            rationale:
                "Keeps restart-state, exhaustion, and fault-boundary meaning on a shared runtime-owned report seam instead of host-private restart heuristics.",
        },
        DeviceSupervisionBoundarySurface {
            id: "runtime-supervision-fault-alignment",
            kind: DeviceSupervisionBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::fault_status and RuntimeObservationReport::interruption_summary",
            runtime_anchor: "RuntimeFaultStatusSnapshot + RuntimeInterruptionSummary",
            rationale:
                "Keeps device supervision classification aligned with shared runtime fault and interruption truth instead of a second hardware-only taxonomy.",
        },
        DeviceSupervisionBoundarySurface {
            id: "shared-host-device-supervision-supervisor-report",
            kind: DeviceSupervisionBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned device supervision truth without private restart-loop reconstruction or host-local fault classes.",
        },
    ]
}

fn device_supervision_boundary_validation_steps(
) -> &'static [DeviceSupervisionBoundaryValidationStep] {
    &[
        DeviceSupervisionBoundaryValidationStep {
            id: "runtime-device-supervision-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states",
            rationale:
                "Proves a downstream-style runtime consumer can inspect recovering and explicit faulted device supervision truth through public runtime reexports alone.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "local-host-device-supervision-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_device_supervision_truth",
            rationale:
                "Proves the local stable host edge forwards recovered, exhausted, and faulted device supervision outcomes on the shared supervisor report seam.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "server-host-device-supervision-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_device_supervision_truth",
            rationale:
                "Proves the server stable host edge forwards runtime-owned recovering and faulted device supervision outcomes without host-private restart policy.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools device_supervision_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable device supervision boundary aligned with the focused proof spine instead of drifting into prose-only documentation.",
        },
        DeviceSupervisionBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-device-supervision-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared recovery, exhaustion, and fault-boundary proof seam without reading private host restart code.",
        },
    ]
}

fn clock_topology_boundary_surfaces() -> &'static [ClockTopologyBoundarySurface] {
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
                "RuntimeObservationApi::get_external_io_snapshot() and RuntimeObservationReport::device_supervision_snapshot",
            runtime_anchor: "RuntimeExternalIoSnapshot + RuntimeDeviceSupervisionSnapshot",
            rationale:
                "Keeps clocking and endpoint-topology classification aligned with supervision and fault-boundary truth instead of a second host-local hardware taxonomy.",
        },
        ClockTopologyBoundarySurface {
            id: "shared-local-host-clock-topology-report",
            kind: ClockTopologyBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local",
            surface: "LocalRuntimeHost::host_supervisor_report() -> RuntimeHostSupervisorReport",
            runtime_anchor: "RuntimeHostSupervisorReport",
            rationale:
                "Proves the stable local host edge forwards runtime-owned clocking and endpoint-topology receipts without recomputing drift or duplex meaning in product code.",
        },
    ]
}

fn clock_topology_boundary_validation_steps() -> &'static [ClockTopologyBoundaryValidationStep] {
    &[
        ClockTopologyBoundaryValidationStep {
            id: "runtime-clock-topology-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts",
            rationale:
                "Proves a downstream-style runtime consumer can inspect drift, duplex-mismatch, and endpoint-topology truth through public runtime reexports and host-report DTOs alone.",
        },
        ClockTopologyBoundaryValidationStep {
            id: "local-host-clock-topology-public-proof",
            command:
                "cargo test -p signal-host-local --test public_host_edge_boundary local_shared_host_edge_exports_runtime_clock_topology_truth",
            rationale:
                "Proves the stable local host edge exposes runtime-owned steady and explicit faulted clock-topology receipts without private host helpers.",
        },
        ClockTopologyBoundaryValidationStep {
            id: "local-host-clock-topology-focused-proof",
            command:
                "cargo test -p signal-host-local local_host_shared_report_surfaces_duplex_ -- --nocapture",
            rationale:
                "Keeps the richer live duplex-mismatch and partial-availability cases on one focused host-owned proof spine even though the stable server edge does not yet expose live host-io receipts.",
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
                "Lets consumers inspect the shared drift, duplex-mismatch, and endpoint-topology seam without reading private host derivation code.",
        },
    ]
}

fn external_io_boundary_surfaces() -> &'static [ExternalIoBoundarySurface] {
    &[
        ExternalIoBoundarySurface {
            id: "runtime-external-io-report",
            kind: ExternalIoBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot",
            runtime_anchor: "RuntimeExternalIoSnapshot",
            rationale:
                "Keeps external-I/O role, monitor state, tap-point, and bounded loopback meaning on one runtime-owned observation seam instead of host-local monitor routing prose.",
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

fn external_io_boundary_validation_steps() -> &'static [ExternalIoBoundaryValidationStep] {
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

fn media_service_boundary_surfaces() -> &'static [MediaServiceBoundarySurface] {
    &[
        MediaServiceBoundarySurface {
            id: "runtime-media-service-report",
            kind: MediaServiceBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::media_pipeline_snapshot, RuntimeObservationReport::media_service_snapshot, and RuntimeSupervisorReport::observation.{media_pipeline_snapshot,media_service_snapshot}",
            runtime_anchor: "RuntimeMediaPipelineSnapshot + RuntimeMediaServiceSnapshot",
            rationale:
                "Keeps indexing, waveform readiness, preview readiness, and invalidation truth on one runtime-owned report seam instead of product-local preview or cache heuristics.",
        },
        MediaServiceBoundarySurface {
            id: "runtime-media-service-snapshot",
            kind: MediaServiceBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationApi::get_media_pipeline_snapshot() and RuntimeObservationApi::get_media_service_snapshot()",
            runtime_anchor: "RuntimeObservationApi media service accessors",
            rationale:
                "Lets downstream consumers inspect the same media indexing and service truth directly from runtime-owned snapshots instead of bespoke media-service facades.",
        },
        MediaServiceBoundarySurface {
            id: "shared-host-media-service-report",
            kind: MediaServiceBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned media readiness, invalidation, and preview state without product-local reconstruction.",
        },
    ]
}

fn media_service_boundary_validation_steps() -> &'static [MediaServiceBoundaryValidationStep] {
    &[
        MediaServiceBoundaryValidationStep {
            id: "runtime-media-service-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth",
            rationale:
                "Proves a downstream-style runtime consumer can inspect indexing, waveform readiness, preview state, and invalidation truth through public runtime reexports alone.",
        },
        MediaServiceBoundaryValidationStep {
            id: "local-host-media-service-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_media_service_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned media pipeline and media-service receipts on supervisor export.",
        },
        MediaServiceBoundaryValidationStep {
            id: "server-host-media-service-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_media_service_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned media readiness and invalidation receipt family on supervisor export.",
        },
        MediaServiceBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools media_service_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable media-service boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        MediaServiceBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-media-service-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared media indexing, waveform, preview, and invalidation seam without reading private product pipelines.",
        },
    ]
}

fn analysis_metadata_boundary_surfaces() -> &'static [AnalysisMetadataBoundarySurface] {
    &[
        AnalysisMetadataBoundarySurface {
            id: "runtime-analysis-metadata-report",
            kind: AnalysisMetadataBoundarySurfaceKind::RuntimeReport,
            crate_name: "signal-runtime",
            surface:
                "RuntimeObservationReport::media_library_snapshot and RuntimeSupervisorReport::observation.media_library_snapshot",
            runtime_anchor: "RuntimeMediaLibraryServiceSnapshot",
            rationale:
                "Keeps reusable loudness, character, and explicit deferred-family coverage on one runtime-owned report seam instead of product-local metadata caches.",
        },
        AnalysisMetadataBoundarySurface {
            id: "runtime-analysis-metadata-snapshot",
            kind: AnalysisMetadataBoundarySurfaceKind::RuntimeSnapshot,
            crate_name: "signal-runtime",
            surface: "RuntimeObservationApi::get_media_library_service_snapshot()",
            runtime_anchor: "RuntimeObservationApi media library accessor",
            rationale:
                "Lets downstream consumers inspect the same asset-analysis descriptor family directly from runtime-owned snapshots instead of reconstructing availability from media-service state alone.",
        },
        AnalysisMetadataBoundarySurface {
            id: "shared-host-analysis-metadata-report",
            kind: AnalysisMetadataBoundarySurfaceKind::HostEdge,
            crate_name: "signal-host-local + signal-host-server",
            surface: "supervisor_report() -> RuntimeSupervisorReport",
            runtime_anchor: "RuntimeSupervisorApi::supervisor_report()",
            rationale:
                "Ensures both stable host edges forward runtime-owned analysis-metadata and library-service receipts without product-local extraction or metadata forks.",
        },
    ]
}

fn analysis_metadata_boundary_validation_steps() -> &'static [AnalysisMetadataBoundaryValidationStep]
{
    &[
        AnalysisMetadataBoundaryValidationStep {
            id: "runtime-analysis-metadata-public-proof",
            command:
                "cargo test -p signal-runtime public_runtime_analysis_metadata_boundary_reports_runtime_owned_library_descriptors",
            rationale:
                "Proves a downstream-style runtime consumer can inspect the reusable library descriptor family, including ready and invalidated outcomes, through public runtime reexports alone.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "local-host-analysis-metadata-proof",
            command:
                "cargo test -p signal-host-local local_shared_host_edge_exports_runtime_analysis_metadata_truth",
            rationale:
                "Proves the stable local host edge forwards runtime-owned analysis metadata and library-service descriptors on supervisor export.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "server-host-analysis-metadata-proof",
            command:
                "cargo test -p signal-host-server server_shared_host_edge_exports_runtime_analysis_metadata_truth",
            rationale:
                "Proves the stable server host edge forwards the same runtime-owned analysis descriptor family without private metadata reconstruction.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "boundary-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools analysis_metadata_boundary_json_reports_runtime_and_host_edge_proofs",
            rationale:
                "Keeps the machine-readable analysis-metadata boundary aligned with the focused runtime and host proof spine instead of drifting into prose-only documentation.",
        },
        AnalysisMetadataBoundaryValidationStep {
            id: "boundary-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-analysis-metadata-boundary --format=json",
            rationale:
                "Lets consumers inspect the shared analysis-metadata and library-service seam without reading private product extraction code.",
        },
    ]
}

fn integrated_acceptance_families() -> &'static [IntegratedAcceptanceFamily] {
    &[
        IntegratedAcceptanceFamily {
            id: "recovery-and-fault-attribution",
            title: "Recovery And Fault Attribution",
            required_tasks: RECOVERY_AND_FAULT_REQUIRED_TASKS,
            advisory_tasks: RECOVERY_AND_FAULT_ADVISORY_TASKS,
            rationale:
                "Keeps interruption, fault diagnostics, and device supervision in the bounded lane while leaving broader continuity depth explicit but non-blocking.",
        },
        IntegratedAcceptanceFamily {
            id: "scheduling-and-execution-pressure",
            title: "Scheduling And Execution Pressure",
            required_tasks: SCHEDULING_AND_PRESSURE_REQUIRED_TASKS,
            advisory_tasks: SCHEDULING_AND_PRESSURE_ADVISORY_TASKS,
            rationale:
                "Pins execution pressure to bounded hot-path and deferred-work policy receipts without forcing every timing-adjacent proof into the required lane.",
        },
        IntegratedAcceptanceFamily {
            id: "adapter-and-portability-breadth",
            title: "Adapter And Portability Breadth",
            required_tasks: ADAPTER_AND_PORTABILITY_REQUIRED_TASKS,
            advisory_tasks: ADAPTER_AND_PORTABILITY_ADVISORY_TASKS,
            rationale:
                "Requires one shared plugin continuity and portability lane while keeping richer per-format and event-depth checks visible as advisory breadth.",
        },
        IntegratedAcceptanceFamily {
            id: "hardware-and-external-io-continuity",
            title: "Hardware And External-I/O Continuity",
            required_tasks: HARDWARE_AND_EXTERNAL_IO_REQUIRED_TASKS,
            advisory_tasks: HARDWARE_AND_EXTERNAL_IO_ADVISORY_TASKS,
            rationale:
                "Makes hardware restart, topology, and external-I/O truth part of the integrated lane instead of leaving them as isolated subsystem proofs.",
        },
        IntegratedAcceptanceFamily {
            id: "media-and-library-service-continuity",
            title: "Media And Library-Service Continuity",
            required_tasks: MEDIA_AND_LIBRARY_REQUIRED_TASKS,
            advisory_tasks: MEDIA_AND_LIBRARY_ADVISORY_TASKS,
            rationale:
                "Keeps reusable media readiness and analysis-metadata descriptors in the shared lane without expanding into product-local browser workflows.",
        },
    ]
}

fn integrated_acceptance_validation_steps() -> &'static [IntegratedAcceptanceValidationStep] {
    &[
        IntegratedAcceptanceValidationStep {
            id: "cross-family-export-proof",
            command:
                "cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_acceptance_evidence",
            rationale:
                "Proves one repo-owned supervisor export carries recovery, deferred-work, adapter breadth, hardware, and media/library evidence together instead of reducing the lane to a checklist of isolated boundary tasks.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor-proof",
            command:
                "cargo test -p signal-supervisor-tools integrated_acceptance_lane_json_reports_required_and_advisory_policy",
            rationale:
                "Keeps the machine-readable integrated acceptance descriptor aligned with the frozen required, advisory, and deferred policy.",
        },
        IntegratedAcceptanceValidationStep {
            id: "lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json",
            rationale:
                "Lets consumers inspect the integrated acceptance lane without reading contract prose or Effigy internals.",
        },
        IntegratedAcceptanceValidationStep {
            id: "required-lane-task",
            command: INTEGRATED_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded required acceptance lane is runnable as one repo-owned grouped task instead of a loose checklist.",
        },
    ]
}

fn g06_soak_lane_records() -> &'static [G06SoakLaneScenarioRecord] {
    &[
        G06SoakLaneScenarioRecord {
            id: "required-local-soak-export",
            status: "required",
            command: "cargo run -p signal-supervisor-tools -- --format=json local soak",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Provides one bounded long-session local-host soak artifact carrying runtime profiling, soak, and supervisor receipts together.",
        },
        G06SoakLaneScenarioRecord {
            id: "required-local-mixed-soak-export",
            status: "required",
            command: "cargo run -p signal-supervisor-tools -- --format=json local mixed",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Keeps mixed watchdog and recovery churn inside the bounded soak lane without depending on deferred server-host overlap behavior.",
        },
        G06SoakLaneScenarioRecord {
            id: "advisory-integrated-acceptance-base",
            status: "advisory",
            command: INTEGRATED_ACCEPTANCE_TASK,
            typed_output:
                "machine-readable integrated acceptance descriptors plus boundary proof outputs",
            rationale:
                "The bounded soak lane still depends on the fast integrated lane staying green, but that fast path remains a separate required base rather than the soak lane itself.",
        },
        G06SoakLaneScenarioRecord {
            id: "deferred-server-soak-export",
            status: "deferred",
            command: "cargo run -p signal-supervisor-tools -- --format=json server soak",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "The broader server-host soak path remains outside the bounded lane because the recovery-overlap attach limit still trips that scenario.",
        },
    ]
}

fn g06_soak_lane_validation_steps() -> &'static [G06SoakLaneValidationStep] {
    &[
        G06SoakLaneValidationStep {
            id: "g06-soak-lane-proof",
            command:
                "cargo test -p signal-supervisor-tools g06_soak_lane_json_reports_required_and_deferred_policy",
            rationale:
                "Keeps the machine-readable bounded soak descriptor aligned with the required, advisory, and deferred policy frozen in the closeout contract.",
        },
        G06SoakLaneValidationStep {
            id: "g06-soak-lane-descriptor",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-g06-soak-lane --format=json",
            rationale:
                "Lets maintainers inspect the bounded soak lane without reading closeout contract prose or Effigy internals.",
        },
        G06SoakLaneValidationStep {
            id: "g06-soak-lane-task",
            command: G06_SOAK_ACCEPTANCE_TASK,
            rationale:
                "Proves the bounded soak lane is runnable as one repo-owned Effigy task instead of a loose list of scenario commands.",
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

fn release_boundary_artifacts() -> &'static [ReleaseBoundaryArtifact] {
    &[
        ReleaseBoundaryArtifact {
            id: "workspace-changelog",
            kind: ReleaseBoundaryArtifactKind::Document,
            path_or_command: RELEASE_CHANGELOG_PATH,
            rationale:
                "Every consumer-facing release baseline must carry a human-readable change summary in the workspace changelog.",
        },
        ReleaseBoundaryArtifact {
            id: "supervisor-export-description",
            kind: ReleaseBoundaryArtifactKind::ExportDescription,
            path_or_command: "cargo run -p signal-supervisor-tools -- --describe-export --format=json",
            rationale:
                "The versioned export schema remains the machine-readable release contract for automation.",
        },
        ReleaseBoundaryArtifact {
            id: "consumer-conformance-matrix",
            kind: ReleaseBoundaryArtifactKind::ConformanceMatrix,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "Consumers need one inspectable list of the runnable proof surfaces included in the baseline.",
        },
        ReleaseBoundaryArtifact {
            id: "publication-packaging-manifest",
            kind: ReleaseBoundaryArtifactKind::PackagingManifest,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json",
            rationale:
                "Publication-grade packaging now has a repo-owned manifest descriptor instead of living only in prose around the baseline release boundary.",
        },
        ReleaseBoundaryArtifact {
            id: "runtime-supervisor-report-demo",
            kind: ReleaseBoundaryArtifactKind::Example,
            path_or_command: "cargo run -p signal-runtime --example supervisor_report_demo",
            rationale:
                "The human-readable report example remains part of the first shared release baseline for manual inspection.",
        },
    ]
}

fn release_boundary_validation_steps() -> &'static [ReleaseBoundaryValidationStep] {
    &[
        ReleaseBoundaryValidationStep {
            id: "consumer-conformance",
            command: RELEASE_CONFORMANCE_TASK,
            rationale:
                "The runnable consumer conformance matrix must pass before the packaging baseline is considered valid.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-health",
            command: "effigy health",
            rationale:
                "The repo-owned build baseline must stay healthy for a release-boundary claim to be credible.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-test",
            command: "effigy test",
            rationale:
                "The shared repo-owned test surface remains part of the packaging baseline rather than downstream-only policy.",
        },
        ReleaseBoundaryValidationStep {
            id: "workspace-validate",
            command: "effigy validate",
            rationale:
                "Validation must include the repo-owned configure/build/test chain before a release boundary is declared.",
        },
    ]
}

fn release_boundary_unstable_scopes() -> &'static [&'static str] {
    &[
        "backend breadth beyond the current CLAP-first plugin path",
        "host convenience APIs outside the frozen runtime/export boundary",
        "crates.io publication and downstream release orchestration",
        "publication packaging beyond the repo-owned manifest descriptor and receipt inventory",
    ]
}

fn packaging_manifest_inputs() -> &'static [PackagingManifestInput] {
    &[
        PackagingManifestInput {
            id: "workspace-changelog",
            kind: PackagingManifestInputKind::Document,
            path_or_command: RELEASE_CHANGELOG_PATH,
            rationale:
                "The publication bundle still anchors human-readable release notes in the workspace changelog.",
        },
        PackagingManifestInput {
            id: "export-boundary-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command: "cargo run -p signal-supervisor-tools -- --describe-export --format=json",
            rationale:
                "The versioned supervisor export descriptor remains the canonical machine-readable schema source.",
        },
        PackagingManifestInput {
            id: "consumer-conformance-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json",
            rationale:
                "The packaging manifest must include the repo-owned consumer-proof boundary rather than a private release matrix.",
        },
        PackagingManifestInput {
            id: "host-edge-boundary-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json",
            rationale:
                "Stable shared host edges must remain explicit in the publication bundle instead of being inferred from host crate internals.",
        },
        PackagingManifestInput {
            id: "release-boundary-descriptor",
            kind: PackagingManifestInputKind::Descriptor,
            path_or_command:
                "cargo run -p signal-supervisor-tools -- --describe-release-boundary --format=json",
            rationale:
                "The publication manifest aggregates the existing host-free release boundary rather than replacing it.",
        },
        PackagingManifestInput {
            id: "plugin-backend-breadth-acceptance",
            kind: PackagingManifestInputKind::ValidationTask,
            path_or_command: "effigy acceptance:plugin-backend-breadth",
            rationale:
                "Release packaging claims about backend-neutral breadth must point back to the repo-owned acceptance task that proves them.",
        },
        PackagingManifestInput {
            id: "host-edge-consumer-acceptance",
            kind: PackagingManifestInputKind::ValidationTask,
            path_or_command: "effigy acceptance:host-edge-consumer",
            rationale:
                "The manifest includes the stable shared host-edge proof rather than assuming it from release prose.",
        },
        PackagingManifestInput {
            id: "packaging-contract",
            kind: PackagingManifestInputKind::Contract,
            path_or_command: PACKAGING_MANIFEST_CONTRACT_PATH,
            rationale:
                "The packaging manifest stays anchored to the frozen contract instead of an ad hoc release script.",
        },
    ]
}

fn packaging_receipt_surfaces() -> &'static [PackagingReceiptSurface] {
    &[
        PackagingReceiptSurface {
            id: "manifest-generation-receipt",
            surface:
                "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json",
            rationale:
                "The packaging manifest descriptor is the repo-owned receipt for what Signal currently considers packageable.",
        },
        PackagingReceiptSurface {
            id: "validation-receipt",
            surface: PACKAGING_MANIFEST_ACCEPTANCE_TASK,
            rationale:
                "The packaging acceptance task is the repo-owned receipt that the declared bundle and validation spine stay runnable together.",
        },
    ]
}

fn packaging_manifest_validation_steps() -> &'static [PackagingManifestValidationStep] {
    &[
        PackagingManifestValidationStep {
            id: "release-boundary-baseline",
            command: "effigy acceptance:release-boundary",
            rationale:
                "Publication packaging builds on the existing release-boundary baseline instead of replacing it.",
        },
        PackagingManifestValidationStep {
            id: "packaging-manifest-description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json",
            rationale:
                "Consumers and automation need one machine-readable publication manifest descriptor.",
        },
        PackagingManifestValidationStep {
            id: "workspace-health",
            command: "effigy health",
            rationale:
                "Publication packaging claims still depend on the repo-owned build baseline staying healthy.",
        },
        PackagingManifestValidationStep {
            id: "workspace-docs",
            command: "effigy qa:docs",
            rationale:
                "The publication manifest depends on docs and index surfaces staying aligned with the declared release bundle.",
        },
    ]
}

fn packaging_manifest_unsupported_paths() -> &'static [&'static str] {
    &[
        "crates.io publication and registry upload automation",
        "signed installers, notarization, and platform distribution packaging",
        "downstream application-specific release wrappers or private CI pipelines",
        "generation closeout bundling and post-release promotion policy beyond the current g05 milestone",
    ]
}

fn downstream_automation_mandatory_fixtures() -> &'static [DownstreamAutomationFixture] {
    &[
        DownstreamAutomationFixture {
            id: "consumer-conformance",
            kind: DownstreamAutomationFixtureKind::AcceptanceTask,
            command: RELEASE_CONFORMANCE_TASK,
            typed_output:
                "conformance matrix descriptor plus task-local test/example receipts",
            rationale:
                "The bounded release fast path still starts from the shared consumer conformance matrix.",
        },
        DownstreamAutomationFixture {
            id: "release-packaging-consumer",
            kind: DownstreamAutomationFixtureKind::AcceptanceTask,
            command: PACKAGING_MANIFEST_ACCEPTANCE_TASK,
            typed_output:
                "release-boundary and packaging-manifest descriptors plus public binary-facing proof",
            rationale:
                "The mandatory release path must prove packaging claims remain consumable without private scripts.",
        },
        DownstreamAutomationFixture {
            id: "downstream-automation-descriptor",
            kind: DownstreamAutomationFixtureKind::Descriptor,
            command:
                "cargo run -p signal-supervisor-tools -- --describe-downstream-automation --format=json",
            typed_output: "machine-readable downstream automation boundary descriptor",
            rationale:
                "Mandatory release automation must stay inspectable as one repo-owned boundary description.",
        },
    ]
}

fn downstream_automation_optional_fixtures() -> &'static [DownstreamAutomationFixture] {
    &[
        DownstreamAutomationFixture {
            id: "local-mixed-watchdog-export",
            kind: DownstreamAutomationFixtureKind::ScenarioExport,
            command: "cargo run -p signal-supervisor-tools -- --format=json local mixed",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Optional depth should exercise richer mixed watchdog/fault scenarios through typed export rather than log-only review.",
        },
        DownstreamAutomationFixture {
            id: "local-soak-export",
            kind: DownstreamAutomationFixtureKind::ScenarioExport,
            command: "cargo run -p signal-supervisor-tools -- --format=json local soak",
            typed_output:
                "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report",
            rationale:
                "Optional depth should include a broader watchdog-soak path while keeping the output typed and inspectable.",
        },
        DownstreamAutomationFixture {
            id: "analysis-acceptance",
            kind: DownstreamAutomationFixtureKind::AcceptanceTask,
            command: "effigy acceptance:analysis",
            typed_output: "analysis harness task receipts across the shared analysis crates",
            rationale:
                "Longer-running shared confidence can extend into broader analysis acceptance without becoming a release prerequisite yet.",
        },
    ]
}

fn downstream_fail_gate_rules() -> &'static [DownstreamFailGateRule] {
    &[
        DownstreamFailGateRule {
            id: "mandatory-release-gate",
            gate: "required",
            command: DOWNSTREAM_AUTOMATION_MANDATORY_TASK,
            blocks_release: true,
            rationale:
                "The bounded downstream release task is the current mandatory gate for widened consumer and packaging claims.",
        },
        DownstreamFailGateRule {
            id: "automation-boundary-descriptor",
            gate: "required",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-downstream-fail-gates --format=json",
            blocks_release: true,
            rationale:
                "The fail-gate policy itself must remain inspectable as a machine-readable repo-owned surface.",
        },
        DownstreamFailGateRule {
            id: "optional-depth-lane",
            gate: "advisory",
            command: DOWNSTREAM_AUTOMATION_OPTIONAL_TASK,
            blocks_release: false,
            rationale:
                "Optional depth broadens confidence, but it does not currently block the fast release path.",
        },
    ]
}

fn downstream_deferred_depth_records() -> &'static [DownstreamDeferredDepthRecord] {
    &[
        DownstreamDeferredDepthRecord {
            id: "server-soak-export",
            command: "cargo run -p signal-supervisor-tools -- --format=json server soak",
            status: "deferred",
            rationale:
                "The current server-host soak path is not yet stable enough to gate release because the recovery-overlap attach limit still trips this fixture.",
        },
        DownstreamDeferredDepthRecord {
            id: "analysis-acceptance-promotion",
            command: "effigy acceptance:analysis",
            status: "deferred",
            rationale:
                "Analysis acceptance remains useful optional depth, but it is not yet part of the bounded shared release gate.",
        },
    ]
}

fn generation_closeout_validation_steps() -> &'static [GenerationCloseoutValidationStep] {
    &[
        GenerationCloseoutValidationStep {
            id: "integrated-acceptance-base",
            command: INTEGRATED_ACCEPTANCE_TASK,
            rationale:
                "The final g06 closeout gate must build on the already-closed integrated acceptance lane instead of replacing it with a prose-only summary.",
        },
        GenerationCloseoutValidationStep {
            id: "bounded-soak-lane",
            command: G06_SOAK_ACCEPTANCE_TASK,
            rationale:
                "The final gate must include one bounded long-session soak lane that stays repo-owned and typed rather than relying on manual endurance folklore.",
        },
        GenerationCloseoutValidationStep {
            id: "generation-closeout-description",
            command:
                "cargo run -p signal-supervisor-tools -- --describe-generation-closeout --format=json",
            rationale:
                "Consumers and maintainers need one machine-readable g06 closeout record tying together integrated acceptance, soak, and provisional readiness status.",
        },
        GenerationCloseoutValidationStep {
            id: "repo-validation",
            command: "effigy validate",
            rationale:
                "The closeout gate still requires the repo-owned configure/build/test chain to stay green.",
        },
    ]
}

fn generation_closeout_residual_risks() -> &'static [&'static str] {
    &[
        "the broader server-host soak path remains deferred because the recovery-overlap attach limit still trips that fixture",
        "wider rerun counts and advisory continuity lanes still remain outside the bounded required closeout gate",
        "the g06 closeout verdict is sufficient to promote g07, but it is still a reusable substrate verdict rather than a Loophole product-launch verdict",
    ]
}

fn generation_closeout_next_queue_summary() -> &'static str {
    "g06 now closes cleanly enough to promote g07. Residual unstable soak and broader advisory confidence depth stay explicitly deferred instead of blocking the next generation."
}

fn generation_closeout_readiness_areas() -> &'static [GenerationReadinessArea] {
    &[
        GenerationReadinessArea {
            id: "runtime-hardening-and-recovery",
            status: "sufficient-for-promotion",
            rationale:
                "Integrated acceptance plus the bounded soak lane now give Signal enough reusable runtime-hardening and recovery evidence to stop treating Loophole's hardening concerns as a blocker for the next generation.",
        },
        GenerationReadinessArea {
            id: "adapter-and-portability-breadth",
            status: "sufficient-for-promotion",
            rationale:
                "CLAP, VST3, AU, parity, generic-event, and recall boundaries now give Loophole a materially broader shared plugin substrate, even though richer per-format depth is still later work.",
        },
        GenerationReadinessArea {
            id: "hardware-and-external-io-substrate",
            status: "sufficient-for-promotion",
            rationale:
                "Device supervision, clock-topology, and external-I/O boundaries now form a reusable hardware substrate strong enough to move feature expansion forward without reopening host-local recovery ownership.",
        },
        GenerationReadinessArea {
            id: "media-and-analysis-service-substrate",
            status: "sufficient-for-promotion",
            rationale:
                "Media-service and analysis-metadata boundaries now provide a bounded shared service baseline, which is enough for g07 feature work even though richer metadata and library workflows remain deferred.",
        },
    ]
}

fn render_host_summary_sections_text(debug: ExportDebugOptions) -> String {
    let mut sections = DEFAULT_HOST_SUMMARY_SECTIONS.join(",");
    if debug.supports(HostSummaryDebugSection::Payload) {
        sections.push(',');
        sections.push_str(HostSummaryDebugSection::Payload.label());
    }
    format!("sections: {sections}\n")
}

fn render_supported_debug_sections_text() -> String {
    let sections = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .map(|section| section.label())
        .collect::<Vec<_>>()
        .join(",");
    format!("debug_sections_supported: {sections}\n")
}

fn render_enabled_debug_sections_text(debug: ExportDebugOptions) -> String {
    let enabled = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .copied()
        .filter(|section| debug.supports(*section))
        .map(|section| section.label())
        .collect::<Vec<_>>();
    if enabled.is_empty() {
        "debug_sections_enabled: none\n".into()
    } else {
        format!("debug_sections_enabled: {}\n", enabled.join(","))
    }
}

fn render_host_summary_sections_json(debug: ExportDebugOptions) -> String {
    let mut sections: Vec<String> = DEFAULT_HOST_SUMMARY_SECTIONS
        .iter()
        .map(|section| json_string(section))
        .collect();
    if debug.supports(HostSummaryDebugSection::Payload) {
        sections.push(json_string(HostSummaryDebugSection::Payload.label()));
    }
    format!("[{}]", sections.join(","))
}

fn render_supported_debug_sections_json() -> String {
    let sections = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .map(|section| json_string(section.label()))
        .collect::<Vec<_>>();
    format!("[{}]", sections.join(","))
}

fn render_enabled_debug_sections_json(debug: ExportDebugOptions) -> String {
    let sections = SUPPORTED_DEBUG_SECTIONS
        .iter()
        .copied()
        .filter(|section| debug.supports(*section))
        .map(|section| json_string(section.label()))
        .collect::<Vec<_>>();
    format!("[{}]", sections.join(","))
}

fn render_local_payload_text(summary: &LocalRuntimeHostSummary) -> String {
    format!(
        "\npayload: events={} parameter_events={} parameter_gestures={} parameter_modulations={} note_events={} note_expression_events={} midi_events={} generated_event_bytes={} first_output_sample={:?}",
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        summary.last_payload.first_output_sample,
    )
}

fn render_server_payload_text(summary: &ServerRuntimeHostSummary) -> String {
    format!(
        "\npayload: events={} parameter_events={} parameter_gestures={} parameter_modulations={} note_events={} note_expression_events={} midi_events={} generated_event_bytes={} first_output_sample={:?}",
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        summary.last_payload.first_output_sample,
    )
}

fn render_local_summary(summary: &LocalRuntimeHostSummary, debug: ExportDebugOptions) -> String {
    let mut rendered = format!(
        "profile=Local backend={}\n{}{}{}execution: sandbox={:?} processed_blocks={} completion={:?} last_block={} control_requests={} control_responses={} heartbeat_responses={} last_control_message={:?} epoch={} restarts={} teardowns={} last_recovery_intent={:?} last_stop_reason={:?}\ntransport: lease_id={:?} region_id={:?} shared_memory_bytes={}\nfaults: deadline_misses={} heartbeat_misses={} watchdog_triggered={} watchdog_reason={:?}",
        summary.backend_name,
        render_host_summary_sections_text(debug),
        render_supported_debug_sections_text(),
        render_enabled_debug_sections_text(debug),
        summary.transport.sandbox_id,
        summary.execution.processed_blocks,
        summary.execution.last_completion_state,
        summary.execution.last_block_sequence,
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.last_control_message,
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        summary.execution.last_recovery_intent,
        summary.execution.last_stop_reason,
        summary.transport.shared_memory_lease_id,
        summary.transport.shared_memory_region_id,
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        summary.faults.watchdog_trigger_reason,
    );
    rendered.push_str(&format!(
        "\nengine: processed_blocks={} graph_id={:?} output_peak={:?} output_rms={:?}",
        summary.execution.engine_processed_blocks,
        summary.execution.last_engine_graph_id,
        summary.execution.last_engine_output_peak,
        summary.execution.last_engine_output_rms,
    ));
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push_str(&render_local_payload_text(summary));
    }
    rendered
}

fn render_server_summary(summary: &ServerRuntimeHostSummary, debug: ExportDebugOptions) -> String {
    let mut rendered = format!(
        "profile=Server\n{}{}{}execution: sandbox={:?} processed_blocks={} completion={:?} last_block={} control_requests={} control_responses={} heartbeat_responses={} last_control_message={:?} epoch={} restarts={} teardowns={} last_recovery_intent={:?} last_stop_reason={:?}\ntransport: lease_id={:?} region_id={:?} shared_memory_bytes={}\nfaults: deadline_misses={} heartbeat_misses={} watchdog_triggered={} watchdog_reason={:?}",
        render_host_summary_sections_text(debug),
        render_supported_debug_sections_text(),
        render_enabled_debug_sections_text(debug),
        summary.transport.sandbox_id,
        summary.execution.processed_blocks,
        summary.execution.last_completion_state,
        summary.execution.last_block_sequence,
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.last_control_message,
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        summary.execution.last_recovery_intent,
        summary.execution.last_stop_reason,
        summary.transport.shared_memory_lease_id,
        summary.transport.shared_memory_region_id,
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        summary.faults.watchdog_trigger_reason,
    );
    rendered.push_str(&format!(
        "\nengine: processed_blocks={} graph_id={:?} output_peak={:?} output_rms={:?}",
        summary.execution.engine_processed_blocks,
        summary.execution.last_engine_graph_id,
        summary.execution.last_engine_output_peak,
        summary.execution.last_engine_output_rms,
    ));
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push_str(&render_server_payload_text(summary));
    }
    rendered
}

fn render_local_payload_json(summary: &LocalRuntimeHostSummary) -> String {
    format!(
        concat!(
            "\"payload\":{{",
            "\"events\":{},",
            "\"parameter_events\":{},",
            "\"parameter_gestures\":{},",
            "\"parameter_modulations\":{},",
            "\"note_events\":{},",
            "\"note_expression_events\":{},",
            "\"midi_events\":{},",
            "\"generated_event_bytes\":{},",
            "\"first_output_sample\":{}",
            "}}"
        ),
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        json_option_f32(summary.last_payload.first_output_sample),
    )
}

fn render_server_payload_json(summary: &ServerRuntimeHostSummary) -> String {
    format!(
        concat!(
            "\"payload\":{{",
            "\"events\":{},",
            "\"parameter_events\":{},",
            "\"parameter_gestures\":{},",
            "\"parameter_modulations\":{},",
            "\"note_events\":{},",
            "\"note_expression_events\":{},",
            "\"midi_events\":{},",
            "\"generated_event_bytes\":{},",
            "\"first_output_sample\":{}",
            "}}"
        ),
        summary.last_payload.event_count,
        summary.last_payload.parameter_event_count,
        summary.last_payload.parameter_gesture_event_count,
        summary.last_payload.parameter_modulation_event_count,
        summary.last_payload.note_event_count,
        summary.last_payload.note_expression_event_count,
        summary.last_payload.midi_event_count,
        summary.last_payload.generated_event_bytes,
        json_option_f32(summary.last_payload.first_output_sample),
    )
}

fn render_local_summary_json(
    summary: &LocalRuntimeHostSummary,
    debug: ExportDebugOptions,
) -> String {
    let mut rendered = format!(
        concat!(
            "{{",
            "\"profile\":\"Local\",",
            "\"backend\":{},",
            "\"sections\":{},",
            "\"debug_sections_supported\":{},",
            "\"debug_sections_enabled\":{},",
            "\"execution\":{{",
            "\"sandbox_id\":{},",
            "\"control_requests\":{},",
            "\"control_responses\":{},",
            "\"heartbeat_responses\":{},",
            "\"processed_blocks\":{},",
            "\"engine_processed_blocks\":{},",
            "\"last_completion_state\":{},",
            "\"last_block_sequence\":{},",
            "\"last_control_message\":{},",
            "\"last_engine_graph_id\":{},",
            "\"last_engine_output_peak\":{},",
            "\"last_engine_output_rms\":{},",
            "\"processing_epoch\":{},",
            "\"restart_count\":{},",
            "\"teardown_count\":{},",
            "\"last_recovery_intent\":{},",
            "\"last_stop_reason\":{}",
            "}},",
            "\"transport\":{{",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"shared_memory_path\":{},",
            "\"shared_memory_bytes\":{}",
            "}},",
            "\"faults\":{{",
            "\"deadline_misses\":{},",
            "\"heartbeat_misses\":{},",
            "\"watchdog_triggered\":{},",
            "\"watchdog_trigger_reason\":{}",
            "}}"
        ),
        json_string(summary.backend_name),
        render_host_summary_sections_json(debug),
        render_supported_debug_sections_json(),
        render_enabled_debug_sections_json(debug),
        json_string(&summary.transport.sandbox_id),
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.processed_blocks,
        summary.execution.engine_processed_blocks,
        json_string(&format!("{:?}", summary.execution.last_completion_state)),
        summary.execution.last_block_sequence,
        json_string(&summary.execution.last_control_message),
        summary
            .execution
            .last_engine_graph_id
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        json_option_f32(summary.execution.last_engine_output_peak),
        json_option_f32(summary.execution.last_engine_output_rms),
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        json_option_debug(summary.execution.last_recovery_intent),
        json_option_debug(summary.execution.last_stop_reason),
        json_string(&summary.transport.shared_memory_lease_id),
        json_string(&summary.transport.shared_memory_region_id),
        json_string(&summary.transport.shared_memory_path),
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        json_option_debug(summary.faults.watchdog_trigger_reason),
    );
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push(',');
        rendered.push_str(&render_local_payload_json(summary));
    }
    rendered.push('}');
    rendered
}

fn render_server_summary_json(
    summary: &ServerRuntimeHostSummary,
    debug: ExportDebugOptions,
) -> String {
    let mut rendered = format!(
        concat!(
            "{{",
            "\"profile\":\"Server\",",
            "\"sections\":{},",
            "\"debug_sections_supported\":{},",
            "\"debug_sections_enabled\":{},",
            "\"execution\":{{",
            "\"sandbox_id\":{},",
            "\"control_requests\":{},",
            "\"control_responses\":{},",
            "\"heartbeat_responses\":{},",
            "\"processed_blocks\":{},",
            "\"engine_processed_blocks\":{},",
            "\"last_completion_state\":{},",
            "\"last_block_sequence\":{},",
            "\"last_control_message\":{},",
            "\"last_engine_graph_id\":{},",
            "\"last_engine_output_peak\":{},",
            "\"last_engine_output_rms\":{},",
            "\"processing_epoch\":{},",
            "\"restart_count\":{},",
            "\"teardown_count\":{},",
            "\"last_recovery_intent\":{},",
            "\"last_stop_reason\":{}",
            "}},",
            "\"transport\":{{",
            "\"lease_id\":{},",
            "\"region_id\":{},",
            "\"shared_memory_path\":{},",
            "\"shared_memory_bytes\":{}",
            "}},",
            "\"faults\":{{",
            "\"deadline_misses\":{},",
            "\"heartbeat_misses\":{},",
            "\"watchdog_triggered\":{},",
            "\"watchdog_trigger_reason\":{}",
            "}}"
        ),
        render_host_summary_sections_json(debug),
        render_supported_debug_sections_json(),
        render_enabled_debug_sections_json(debug),
        json_string(&summary.transport.sandbox_id),
        summary.execution.control_requests,
        summary.execution.control_responses,
        summary.execution.heartbeat_responses,
        summary.execution.processed_blocks,
        summary.execution.engine_processed_blocks,
        json_string(&format!("{:?}", summary.execution.last_completion_state)),
        summary.execution.last_block_sequence,
        json_string(&summary.execution.last_control_message),
        summary
            .execution
            .last_engine_graph_id
            .as_deref()
            .map(json_string)
            .unwrap_or_else(|| "null".into()),
        json_option_f32(summary.execution.last_engine_output_peak),
        json_option_f32(summary.execution.last_engine_output_rms),
        summary.execution.processing_epoch,
        summary.execution.restart_count,
        summary.execution.teardown_count,
        json_option_debug(summary.execution.last_recovery_intent),
        json_option_debug(summary.execution.last_stop_reason),
        json_string(&summary.transport.shared_memory_lease_id),
        json_string(&summary.transport.shared_memory_region_id),
        json_string(&summary.transport.shared_memory_path),
        summary.transport.shared_memory_bytes,
        summary.faults.deadline_misses,
        summary.faults.heartbeat_misses,
        summary.faults.watchdog_triggered,
        json_option_debug(summary.faults.watchdog_trigger_reason),
    );
    if debug.supports(HostSummaryDebugSection::Payload) {
        rendered.push(',');
        rendered.push_str(&render_server_payload_json(summary));
    }
    rendered.push('}');
    rendered
}

fn json_option_f32(value: Option<f32>) -> String {
    match value {
        Some(value) => value.to_string(),
        None => "null".into(),
    }
}

fn render_supervisor_export_json(
    profile: HostProfile,
    scenario: Scenario,
    host_summary: String,
    profiling: &RuntimeProfilingReceipt,
    soak: &RuntimeSoakReceipt,
    supervisor_report: &RuntimeSupervisorReport,
) -> String {
    format!(
        concat!(
            "{{",
            "\"schema\":{},",
            "\"schema_version\":{},",
            "\"profile\":{},",
            "\"scenario\":{},",
            "\"host_summary\":{},",
            "\"profiling_receipt\":{},",
            "\"soak_receipt\":{},",
            "\"supervisor_report\":{}",
            "}}"
        ),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        json_string(&format!("{profile:?}")),
        json_string(&format!("{scenario:?}")),
        host_summary,
        profiling.render_json(),
        soak.render_json(),
        supervisor_report.render_json(),
    )
}

fn render_export_description_text() -> String {
    format!(
        "schema: {EXPORT_SCHEMA}\nschema_version: {EXPORT_SCHEMA_VERSION}\ndefault_host_summary_sections: {}\nsupported_debug_sections: {}",
        DEFAULT_HOST_SUMMARY_SECTIONS.join(","),
        SUPPORTED_DEBUG_SECTIONS
            .iter()
            .map(|section| section.label())
            .collect::<Vec<_>>()
            .join(",")
    )
}

fn render_export_description_json() -> String {
    format!(
        concat!(
            "{{",
            "\"schema\":{},",
            "\"schema_version\":{},",
            "\"default_host_summary_sections\":{},",
            "\"supported_debug_sections\":{}",
            "}}"
        ),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        format!(
            "[{}]",
            DEFAULT_HOST_SUMMARY_SECTIONS
                .iter()
                .map(|section| json_string(section))
                .collect::<Vec<_>>()
                .join(",")
        ),
        render_supported_debug_sections_json(),
    )
}

fn print_export_description(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_export_description_text()),
        OutputFormat::Json => println!("{}", render_export_description_json()),
    }
}

fn render_conformance_matrix_text() -> String {
    let mut rendered = String::from("consumer_conformance_matrix:\n");
    for entry in conformance_matrix_entries() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  command: {}\n  rationale: {}\n",
            entry.id,
            entry.kind.label(),
            entry.crate_name,
            entry.surface,
            entry.command,
            entry.rationale,
        ));
    }
    rendered
}

fn render_conformance_matrix_json() -> String {
    let entries = conformance_matrix_entries()
        .iter()
        .map(|entry| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"crate\":{},",
                    "\"surface\":{},",
                    "\"command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(entry.id),
                json_string(entry.kind.label()),
                json_string(entry.crate_name),
                json_string(entry.surface),
                json_string(entry.command),
                json_string(entry.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"matrix\":\"signal.consumer.conformance\",",
            "\"entry_count\":{},",
            "\"entries\":[{}]",
            "}}"
        ),
        conformance_matrix_entries().len(),
        entries,
    )
}

fn print_conformance_matrix(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_conformance_matrix_text()),
        OutputFormat::Json => println!("{}", render_conformance_matrix_json()),
    }
}

fn render_interruption_boundary_text() -> String {
    let mut rendered = format!(
        "interruption_boundary: {INTERRUPTION_BOUNDARY}\ncontract_path: {INTERRUPTION_CONTRACT_PATH}\nacceptance_task: {INTERRUPTION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in interruption_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in interruption_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "device-loss-specific fault truth is still stronger on broader host I/O surfaces than on a dedicated runtime-owned device-loss snapshot",
        "subsystem-specific recording, plugin transport, and offline render recovery depth still belong to later g06 milestones",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_interruption_boundary_json() -> String {
    let surfaces = interruption_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = interruption_boundary_validation_steps()
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
    let deferred_scope = [
        "device-loss-specific fault truth is still stronger on broader host I/O surfaces than on a dedicated runtime-owned device-loss snapshot",
        "subsystem-specific recording, plugin transport, and offline render recovery depth still belong to later g06 milestones",
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
            "\"surfaces\":[{}],",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(INTERRUPTION_BOUNDARY),
        json_string(INTERRUPTION_CONTRACT_PATH),
        json_string(INTERRUPTION_ACCEPTANCE_TASK),
        surfaces,
        validation_steps,
        deferred_scope,
    )
}

fn render_recording_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "recording_continuity_boundary: {RECORDING_CONTINUITY_BOUNDARY}\ncontract_path: {RECORDING_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {RECORDING_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in recording_continuity_boundary_surfaces() {
        let kind = match surface.kind {
            RecordingContinuityBoundarySurfaceKind::RuntimeReceipt => "runtime-receipt",
            RecordingContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
            RecordingContinuityBoundarySurfaceKind::HostEdge => "host-edge",
        };
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            kind,
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in recording_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "concrete MIDI capture and commit DTOs are still deferred, so the continuity family is typed but not yet format-complete",
        "same-identity resumable capture is currently proven through safe-mode degradation rather than a richer dedicated capture pause or resume API",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_recording_continuity_boundary_json() -> String {
    let surfaces = recording_continuity_boundary_surfaces()
        .iter()
        .map(|surface| {
            let kind = match surface.kind {
                RecordingContinuityBoundarySurfaceKind::RuntimeReceipt => "runtime-receipt",
                RecordingContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
                RecordingContinuityBoundarySurfaceKind::HostEdge => "host-edge",
            };
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
                json_string(kind),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = recording_continuity_validation_steps()
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
    let deferred_scope = [
        "concrete MIDI capture and commit DTOs are still deferred, so the continuity family is typed but not yet format-complete",
        "same-identity resumable capture is currently proven through safe-mode degradation rather than a richer dedicated capture pause or resume API",
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
        json_string(RECORDING_CONTINUITY_BOUNDARY),
        json_string(RECORDING_CONTINUITY_CONTRACT_PATH),
        json_string(RECORDING_CONTINUITY_ACCEPTANCE_TASK),
        recording_continuity_boundary_surfaces().len(),
        surfaces,
        recording_continuity_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_interruption_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_interruption_boundary_text()),
        OutputFormat::Json => println!("{}", render_interruption_boundary_json()),
    }
}

fn render_fault_diagnostic_boundary_text() -> String {
    let mut rendered = format!(
        "fault_diagnostic_boundary: {FAULT_DIAGNOSTIC_BOUNDARY}\ncontract_path: {FAULT_DIAGNOSTIC_CONTRACT_PATH}\nacceptance_task: {FAULT_DIAGNOSTIC_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in fault_diagnostic_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in fault_diagnostic_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "callback pressure remains advisory host evidence rather than a stronger canonical runtime family",
        "per-event traces, remote diagnostics pipelines, and product-specific diagnostic UX remain out of scope for this shared boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_fault_diagnostic_boundary_json() -> String {
    let surfaces = fault_diagnostic_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = fault_diagnostic_boundary_validation_steps()
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
    let deferred_scope = [
        "callback pressure remains advisory host evidence rather than a stronger canonical runtime family",
        "per-event traces, remote diagnostics pipelines, and product-specific diagnostic UX remain out of scope for this shared boundary",
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
        json_string(FAULT_DIAGNOSTIC_BOUNDARY),
        json_string(FAULT_DIAGNOSTIC_CONTRACT_PATH),
        json_string(FAULT_DIAGNOSTIC_ACCEPTANCE_TASK),
        fault_diagnostic_boundary_surfaces().len(),
        surfaces,
        fault_diagnostic_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_fault_diagnostic_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_fault_diagnostic_boundary_text()),
        OutputFormat::Json => println!("{}", render_fault_diagnostic_boundary_json()),
    }
}

fn render_critical_path_boundary_text() -> String {
    let mut rendered = format!(
        "critical_path_boundary: {CRITICAL_PATH_BOUNDARY}\ncontract_path: {CRITICAL_PATH_CONTRACT_PATH}\nacceptance_task: {CRITICAL_PATH_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in critical_path_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in critical_path_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "deeper scheduler attribution beyond the current bounded hot-node, hot-group, and critical-path lane receipts remains deferred to later profiling work",
        "node-by-node elapsed-time traces, flamegraph exports, and host thread telemetry remain outside this bounded consumer surface",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_critical_path_boundary_json() -> String {
    let surfaces = critical_path_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = critical_path_boundary_validation_steps()
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
    let deferred_scope = [
        "deeper scheduler attribution beyond the current bounded hot-node, hot-group, and critical-path lane receipts remains deferred to later profiling work",
        "node-by-node elapsed-time traces, flamegraph exports, and host thread telemetry remain outside this bounded consumer surface",
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
        json_string(CRITICAL_PATH_BOUNDARY),
        json_string(CRITICAL_PATH_CONTRACT_PATH),
        json_string(CRITICAL_PATH_ACCEPTANCE_TASK),
        critical_path_boundary_surfaces().len(),
        surfaces,
        critical_path_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_critical_path_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_critical_path_boundary_text()),
        OutputFormat::Json => println!("{}", render_critical_path_boundary_json()),
    }
}

fn render_block_timing_boundary_text() -> String {
    let mut rendered = format!(
        "block_timing_boundary: {BLOCK_TIMING_BOUNDARY}\ncontract_path: {BLOCK_TIMING_CONTRACT_PATH}\nacceptance_task: {BLOCK_TIMING_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in block_timing_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in block_timing_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "critical-path, hot-node, and worker-lane attribution are still deferred to g06.007 instead of being inferred from block timing alone",
        "host callback cadence remains advisory evidence and does not outrank the runtime-owned per-block timing seam",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_block_timing_boundary_json() -> String {
    let surfaces = block_timing_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = block_timing_boundary_validation_steps()
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
    let deferred_scope = [
        "critical-path, hot-node, and worker-lane attribution are still deferred to g06.007 instead of being inferred from block timing alone",
        "host callback cadence remains advisory evidence and does not outrank the runtime-owned per-block timing seam",
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
        json_string(BLOCK_TIMING_BOUNDARY),
        json_string(BLOCK_TIMING_CONTRACT_PATH),
        json_string(BLOCK_TIMING_ACCEPTANCE_TASK),
        block_timing_boundary_surfaces().len(),
        surfaces,
        block_timing_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_block_timing_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_block_timing_boundary_text()),
        OutputFormat::Json => println!("{}", render_block_timing_boundary_json()),
    }
}

fn render_deferred_work_policy_boundary_text() -> String {
    let mut rendered = format!(
        "deferred_work_policy_boundary: {DEFERRED_WORK_POLICY_BOUNDARY}\ncontract_path: {DEFERRED_WORK_POLICY_CONTRACT_PATH}\nacceptance_task: {DEFERRED_WORK_POLICY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in deferred_work_policy_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in deferred_work_policy_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "consumer-facing proof is limited to the current bounded deferred-service family rather than a generic future job scheduler",
        "distributed or remote deferred-work ownership remains deferred beyond this shared local runtime policy boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_deferred_work_policy_boundary_json() -> String {
    let surfaces = deferred_work_policy_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = deferred_work_policy_boundary_validation_steps()
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
    let deferred_scope = [
        "consumer-facing proof is limited to the current bounded deferred-service family rather than a generic future job scheduler",
        "distributed or remote deferred-work ownership remains deferred beyond this shared local runtime policy boundary",
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
        json_string(DEFERRED_WORK_POLICY_BOUNDARY),
        json_string(DEFERRED_WORK_POLICY_CONTRACT_PATH),
        json_string(DEFERRED_WORK_POLICY_ACCEPTANCE_TASK),
        deferred_work_policy_boundary_surfaces().len(),
        surfaces,
        deferred_work_policy_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_deferred_work_policy_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_deferred_work_policy_boundary_text()),
        OutputFormat::Json => println!("{}", render_deferred_work_policy_boundary_json()),
    }
}

fn print_recording_continuity_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_recording_continuity_boundary_text()),
        OutputFormat::Json => println!("{}", render_recording_continuity_boundary_json()),
    }
}

fn render_offline_render_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "offline_render_continuity_boundary: {OFFLINE_RENDER_CONTINUITY_BOUNDARY}\ncontract_path: {OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in offline_render_continuity_boundary_surfaces() {
        let kind = match surface.kind {
            OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot => "runtime-snapshot",
            OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
            OfflineRenderContinuityBoundarySurfaceKind::HostEdge => "host-edge",
        };
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            kind,
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in offline_render_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "restart-survival across full process restart still needs later deeper render recovery work beyond the current runtime stop/restart proof",
        "dedicated durable queue ownership and remote render job orchestration remain out of scope for this continuity boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_offline_render_continuity_boundary_json() -> String {
    let surfaces = offline_render_continuity_boundary_surfaces()
        .iter()
        .map(|surface| {
            let kind = match surface.kind {
                OfflineRenderContinuityBoundarySurfaceKind::RuntimeSnapshot => "runtime-snapshot",
                OfflineRenderContinuityBoundarySurfaceKind::RuntimeReport => "runtime-report",
                OfflineRenderContinuityBoundarySurfaceKind::HostEdge => "host-edge",
            };
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
                json_string(kind),
                json_string(surface.crate_name),
                json_string(surface.surface),
                json_string(surface.runtime_anchor),
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = offline_render_continuity_validation_steps()
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
    let deferred_scope = [
        "restart-survival across full process restart still needs later deeper render recovery work beyond the current runtime stop/restart proof",
        "dedicated durable queue ownership and remote render job orchestration remain out of scope for this continuity boundary",
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
        json_string(OFFLINE_RENDER_CONTINUITY_BOUNDARY),
        json_string(OFFLINE_RENDER_CONTINUITY_CONTRACT_PATH),
        json_string(OFFLINE_RENDER_CONTINUITY_ACCEPTANCE_TASK),
        offline_render_continuity_boundary_surfaces().len(),
        surfaces,
        offline_render_continuity_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_offline_render_continuity_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_offline_render_continuity_boundary_text()),
        OutputFormat::Json => println!("{}", render_offline_render_continuity_boundary_json()),
    }
}

fn render_plugin_continuity_boundary_text() -> String {
    let mut rendered = format!(
        "plugin_continuity_boundary: {PLUGIN_CONTINUITY_BOUNDARY}\ncontract_path: {PLUGIN_CONTINUITY_CONTRACT_PATH}\nacceptance_task: {PLUGIN_CONTINUITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in plugin_continuity_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in plugin_continuity_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
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

fn render_plugin_continuity_boundary_json() -> String {
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
                json_string(surface.rationale),
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
                json_string(step.rationale),
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
        deferred_scope,
    )
}

fn print_plugin_continuity_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_plugin_continuity_boundary_text()),
        OutputFormat::Json => println!("{}", render_plugin_continuity_boundary_json()),
    }
}

fn render_vst3_boundary_text() -> String {
    let mut rendered = format!(
        "vst3_boundary: {VST3_BOUNDARY}\ncontract_path: {VST3_CONTRACT_PATH}\nacceptance_task: {VST3_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in vst3_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in vst3_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared VST3 discovery and lifecycle truth is now public, but richer event, unit, and program-list depth still remain later cross-adapter work",
        "the current boundary proves adapter realization through runtime and stable host edges, not wider cross-format parity or publication-grade plugin breadth",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_vst3_boundary_json() -> String {
    let surfaces = vst3_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = vst3_boundary_validation_steps()
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
    let deferred_scope = [
        "shared VST3 discovery and lifecycle truth is now public, but richer event, unit, and program-list depth still remain later cross-adapter work",
        "the current boundary proves adapter realization through runtime and stable host edges, not wider cross-format parity or publication-grade plugin breadth",
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
        json_string(VST3_BOUNDARY),
        json_string(VST3_CONTRACT_PATH),
        json_string(VST3_ACCEPTANCE_TASK),
        vst3_boundary_surfaces().len(),
        surfaces,
        vst3_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_vst3_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_vst3_boundary_text()),
        OutputFormat::Json => println!("{}", render_vst3_boundary_json()),
    }
}

fn render_au_boundary_text() -> String {
    let mut rendered = format!(
        "au_boundary: {AU_BOUNDARY}\ncontract_path: {AU_CONTRACT_PATH}\nacceptance_task: {AU_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in au_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in au_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared AU discovery and lifecycle truth is now public, but richer parameter-tree, preset, editor, and event-model depth still remain later cross-adapter work",
        "the current boundary proves adapter realization through runtime and stable host edges, not wider cross-format parity or publication-grade plugin breadth",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_au_boundary_json() -> String {
    let surfaces = au_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = au_boundary_validation_steps()
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
    let deferred_scope = [
        "shared AU discovery and lifecycle truth is now public, but richer parameter-tree, preset, editor, and event-model depth still remain later cross-adapter work",
        "the current boundary proves adapter realization through runtime and stable host edges, not wider cross-format parity or publication-grade plugin breadth",
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
        json_string(AU_BOUNDARY),
        json_string(AU_CONTRACT_PATH),
        json_string(AU_ACCEPTANCE_TASK),
        au_boundary_surfaces().len(),
        surfaces,
        au_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_au_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_au_boundary_text()),
        OutputFormat::Json => println!("{}", render_au_boundary_json()),
    }
}

fn render_cross_adapter_parity_boundary_text() -> String {
    let mut rendered = format!(
        "cross_adapter_parity_boundary: {CROSS_ADAPTER_PARITY_BOUNDARY}\ncontract_path: {CROSS_ADAPTER_PARITY_CONTRACT_PATH}\nacceptance_task: {CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in cross_adapter_parity_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in cross_adapter_parity_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared CLAP, VST3, and AU parity truth is now public, but richer event-model, preset, editor, and unit-tree parity still remain later cross-adapter work",
        "the current boundary proves bounded platform coverage and lifecycle parity through runtime and stable host edges, not publication-grade capability marketing or deeper adapter internals",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_cross_adapter_parity_boundary_json() -> String {
    let surfaces = cross_adapter_parity_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = cross_adapter_parity_boundary_validation_steps()
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
    let deferred_scope = [
        "shared CLAP, VST3, and AU parity truth is now public, but richer event-model, preset, editor, and unit-tree parity still remain later cross-adapter work",
        "the current boundary proves bounded platform coverage and lifecycle parity through runtime and stable host edges, not publication-grade capability marketing or deeper adapter internals",
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
        json_string(CROSS_ADAPTER_PARITY_BOUNDARY),
        json_string(CROSS_ADAPTER_PARITY_CONTRACT_PATH),
        json_string(CROSS_ADAPTER_PARITY_ACCEPTANCE_TASK),
        cross_adapter_parity_boundary_surfaces().len(),
        surfaces,
        cross_adapter_parity_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_cross_adapter_parity_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_cross_adapter_parity_boundary_text()),
        OutputFormat::Json => println!("{}", render_cross_adapter_parity_boundary_json()),
    }
}

fn render_generic_event_boundary_text() -> String {
    let mut rendered = format!(
        "generic_event_boundary: {GENERIC_EVENT_BOUNDARY}\ncontract_path: {GENERIC_EVENT_CONTRACT_PATH}\nacceptance_task: {GENERIC_EVENT_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in generic_event_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in generic_event_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared generic event truth is now consumable, but richer per-format packet families, SysEx, controller mapping, and editor semantics remain later work",
        "the current boundary proves bounded event and note-expression capability receipts through runtime and stable host edges, not full CLAP, VST3, and AU packet-model parity",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_generic_event_boundary_json() -> String {
    let surfaces = generic_event_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = generic_event_boundary_validation_steps()
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
    let deferred_scope = [
        "shared generic event truth is now consumable, but richer per-format packet families, SysEx, controller mapping, and editor semantics remain later work",
        "the current boundary proves bounded event and note-expression capability receipts through runtime and stable host edges, not full CLAP, VST3, and AU packet-model parity",
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
        json_string(GENERIC_EVENT_BOUNDARY),
        json_string(GENERIC_EVENT_CONTRACT_PATH),
        json_string(GENERIC_EVENT_ACCEPTANCE_TASK),
        generic_event_boundary_surfaces().len(),
        surfaces,
        generic_event_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_generic_event_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_generic_event_boundary_text()),
        OutputFormat::Json => println!("{}", render_generic_event_boundary_json()),
    }
}

fn render_recall_portability_boundary_text() -> String {
    let mut rendered = format!(
        "recall_portability_boundary: {RECALL_PORTABILITY_BOUNDARY}\ncontract_path: {RECALL_PORTABILITY_CONTRACT_PATH}\nacceptance_task: {RECALL_PORTABILITY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in recall_portability_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in recall_portability_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared preset-state portability truth is now consumable, but lossless cross-adapter preset interchange, richer preset families, and adapter-native document models remain later work",
        "the current boundary proves bounded ARA document, source, and region context transfer through runtime and stable host edges, not fuller ARA editor workflow or persistent product document semantics",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_recall_portability_boundary_json() -> String {
    let surfaces = recall_portability_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = recall_portability_boundary_validation_steps()
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
    let deferred_scope = [
        "shared preset-state portability truth is now consumable, but lossless cross-adapter preset interchange, richer preset families, and adapter-native document models remain later work",
        "the current boundary proves bounded ARA document, source, and region context transfer through runtime and stable host edges, not fuller ARA editor workflow or persistent product document semantics",
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
        json_string(RECALL_PORTABILITY_BOUNDARY),
        json_string(RECALL_PORTABILITY_CONTRACT_PATH),
        json_string(RECALL_PORTABILITY_ACCEPTANCE_TASK),
        recall_portability_boundary_surfaces().len(),
        surfaces,
        recall_portability_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_recall_portability_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_recall_portability_boundary_text()),
        OutputFormat::Json => println!("{}", render_recall_portability_boundary_json()),
    }
}

fn render_device_supervision_boundary_text() -> String {
    let mut rendered = format!(
        "device_supervision_boundary: {DEVICE_SUPERVISION_BOUNDARY}\ncontract_path: {DEVICE_SUPERVISION_CONTRACT_PATH}\nacceptance_task: {DEVICE_SUPERVISION_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in device_supervision_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in device_supervision_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "shared device supervision truth is now public, but broader backend-matrix breadth and endpoint-topology depth still remain later hardware work",
        "the current boundary proves recovering, exhausted, and faulted device outcomes on shared runtime and host edges, not product-local recovery UX or remote hardware orchestration",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_device_supervision_boundary_json() -> String {
    let surfaces = device_supervision_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = device_supervision_boundary_validation_steps()
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
    let deferred_scope = [
        "shared device supervision truth is now public, but broader backend-matrix breadth and endpoint-topology depth still remain later hardware work",
        "the current boundary proves recovering, exhausted, and faulted device outcomes on shared runtime and host edges, not product-local recovery UX or remote hardware orchestration",
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
        json_string(DEVICE_SUPERVISION_BOUNDARY),
        json_string(DEVICE_SUPERVISION_CONTRACT_PATH),
        json_string(DEVICE_SUPERVISION_ACCEPTANCE_TASK),
        device_supervision_boundary_surfaces().len(),
        surfaces,
        device_supervision_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_device_supervision_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_device_supervision_boundary_text()),
        OutputFormat::Json => println!("{}", render_device_supervision_boundary_json()),
    }
}

fn render_clock_topology_boundary_text() -> String {
    let mut rendered = format!(
        "clock_topology_boundary: {CLOCK_TOPOLOGY_BOUNDARY}\ncontract_path: {CLOCK_TOPOLOGY_CONTRACT_PATH}\nacceptance_task: {CLOCK_TOPOLOGY_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in clock_topology_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in clock_topology_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned drift, discontinuity, duplex-mismatch, and endpoint-topology meaning, but broader external-I/O, monitoring, and loopback depth still belongs to g06.016",
        "the stable local host edge exposes live host-io receipts directly, while the stable server host edge still omits that live clocking seam and remains outside this focused boundary",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_clock_topology_boundary_json() -> String {
    let surfaces = clock_topology_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = clock_topology_boundary_validation_steps()
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
    let deferred_scope = [
        "the shared boundary now proves runtime-owned drift, discontinuity, duplex-mismatch, and endpoint-topology meaning, but broader external-I/O, monitoring, and loopback depth still belongs to g06.016",
        "the stable local host edge exposes live host-io receipts directly, while the stable server host edge still omits that live clocking seam and remains outside this focused boundary",
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
        json_string(CLOCK_TOPOLOGY_BOUNDARY),
        json_string(CLOCK_TOPOLOGY_CONTRACT_PATH),
        json_string(CLOCK_TOPOLOGY_ACCEPTANCE_TASK),
        clock_topology_boundary_surfaces().len(),
        surfaces,
        clock_topology_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_clock_topology_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_clock_topology_boundary_text()),
        OutputFormat::Json => println!("{}", render_clock_topology_boundary_json()),
    }
}

fn render_external_io_boundary_text() -> String {
    let mut rendered = format!(
        "external_io_boundary: {EXTERNAL_IO_BOUNDARY}\ncontract_path: {EXTERNAL_IO_CONTRACT_PATH}\nacceptance_task: {EXTERNAL_IO_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in external_io_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in external_io_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned external-I/O role, monitor state, tap-point, and bounded loopback meaning, but richer measurement-session and calibration workflows still belong to later g06.016 and media-service work",
        "the stable server host edge currently proves explicit unavailable monitoring and loopback state rather than a live server-host hardware seam, so broader live server-side external-I/O depth remains deferred",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_external_io_boundary_json() -> String {
    let surfaces = external_io_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = external_io_boundary_validation_steps()
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
    let deferred_scope = [
        "the shared boundary now proves runtime-owned external-I/O role, monitor state, tap-point, and bounded loopback meaning, but richer measurement-session and calibration workflows still belong to later g06.016 and media-service work",
        "the stable server host edge currently proves explicit unavailable monitoring and loopback state rather than a live server-host hardware seam, so broader live server-side external-I/O depth remains deferred",
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
        json_string(EXTERNAL_IO_BOUNDARY),
        json_string(EXTERNAL_IO_CONTRACT_PATH),
        json_string(EXTERNAL_IO_ACCEPTANCE_TASK),
        external_io_boundary_surfaces().len(),
        surfaces,
        external_io_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_external_io_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_external_io_boundary_text()),
        OutputFormat::Json => println!("{}", render_external_io_boundary_json()),
    }
}

fn render_media_service_boundary_text() -> String {
    let mut rendered = format!(
        "media_service_boundary: {MEDIA_SERVICE_BOUNDARY}\ncontract_path: {MEDIA_SERVICE_CONTRACT_PATH}\nacceptance_task: {MEDIA_SERVICE_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in media_service_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in media_service_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned media indexing, waveform readiness, preview state, and invalidation receipts, but richer metadata extraction and broader library-service depth still belong to later g06.018 work",
        "this closes the bounded consumer seam for shared media-service state, not product-local browser, collection, or editorial media-management workflows",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_media_service_boundary_json() -> String {
    let surfaces = media_service_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = media_service_boundary_validation_steps()
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
    let deferred_scope = [
        "the shared boundary now proves runtime-owned media indexing, waveform readiness, preview state, and invalidation receipts, but richer metadata extraction and broader library-service depth still belong to later g06.018 work",
        "this closes the bounded consumer seam for shared media-service state, not product-local browser, collection, or editorial media-management workflows",
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
        json_string(MEDIA_SERVICE_BOUNDARY),
        json_string(MEDIA_SERVICE_CONTRACT_PATH),
        json_string(MEDIA_SERVICE_ACCEPTANCE_TASK),
        media_service_boundary_surfaces().len(),
        surfaces,
        media_service_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_media_service_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_media_service_boundary_text()),
        OutputFormat::Json => println!("{}", render_media_service_boundary_json()),
    }
}

fn render_analysis_metadata_boundary_text() -> String {
    let mut rendered = format!(
        "analysis_metadata_boundary: {ANALYSIS_METADATA_BOUNDARY}\ncontract_path: {ANALYSIS_METADATA_CONTRACT_PATH}\nacceptance_task: {ANALYSIS_METADATA_ACCEPTANCE_TASK}\nsurfaces:\n"
    );
    for surface in analysis_metadata_boundary_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  crate: {}\n  surface: {}\n  runtime_anchor: {}\n  rationale: {}\n",
            surface.id,
            surface.kind.label(),
            surface.crate_name,
            surface.surface,
            surface.runtime_anchor,
            surface.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in analysis_metadata_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the shared boundary now proves runtime-owned reusable loudness and character descriptors plus explicit deferred-family coverage, but broader rhythm, tonal, and embedding payload depth still belongs to later work",
        "this closes the bounded consumer seam for analysis-metadata and library-service truth, not product-local browser, collection, tagging, or recommendation workflows",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_analysis_metadata_boundary_json() -> String {
    let surfaces = analysis_metadata_boundary_surfaces()
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
                json_string(surface.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = analysis_metadata_boundary_validation_steps()
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
    let deferred_scope = [
        "the shared boundary now proves runtime-owned reusable loudness and character descriptors plus explicit deferred-family coverage, but broader rhythm, tonal, and embedding payload depth still belongs to later work",
        "this closes the bounded consumer seam for analysis-metadata and library-service truth, not product-local browser, collection, tagging, or recommendation workflows",
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
        json_string(ANALYSIS_METADATA_BOUNDARY),
        json_string(ANALYSIS_METADATA_CONTRACT_PATH),
        json_string(ANALYSIS_METADATA_ACCEPTANCE_TASK),
        analysis_metadata_boundary_surfaces().len(),
        surfaces,
        analysis_metadata_boundary_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_analysis_metadata_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_analysis_metadata_boundary_text()),
        OutputFormat::Json => println!("{}", render_analysis_metadata_boundary_json()),
    }
}

fn render_integrated_acceptance_lane_text() -> String {
    let mut rendered = format!(
        "integrated_acceptance_lane: {INTEGRATED_ACCEPTANCE_LANE}\ncontract_path: {INTEGRATED_ACCEPTANCE_CONTRACT_PATH}\nacceptance_task: {INTEGRATED_ACCEPTANCE_TASK}\nrequired_tasks:\n"
    );
    for task in INTEGRATED_ACCEPTANCE_REQUIRED_TASKS {
        rendered.push_str(&format!("- {task}\n"));
    }
    rendered.push_str("advisory_tasks:\n");
    for task in INTEGRATED_ACCEPTANCE_ADVISORY_TASKS {
        rendered.push_str(&format!("- {task}\n"));
    }
    rendered.push_str("families:\n");
    for family in integrated_acceptance_families() {
        rendered.push_str(&format!(
            "- id: {}\n  title: {}\n  rationale: {}\n  required_tasks:\n",
            family.id, family.title, family.rationale
        ));
        for task in family.required_tasks {
            rendered.push_str(&format!("    - {task}\n"));
        }
        rendered.push_str("  advisory_tasks:\n");
        for task in family.advisory_tasks {
            rendered.push_str(&format!("    - {task}\n"));
        }
    }
    rendered.push_str("validation_steps:\n");
    for step in integrated_acceptance_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the bounded lane now groups one required cross-family acceptance path, but long-session soak thresholds and promotion policy still belong to g06.020",
        "unstable broader server-host recovery-overlap scenarios remain explicitly deferred until the integrated lane is real and bounded",
        "product-local QA dashboards, browser workflows, and exhaustive environment certification remain outside the shared Signal acceptance lane",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_integrated_acceptance_lane_json() -> String {
    let required_tasks = INTEGRATED_ACCEPTANCE_REQUIRED_TASKS
        .iter()
        .map(|task| json_string(task))
        .collect::<Vec<_>>()
        .join(",");
    let advisory_tasks = INTEGRATED_ACCEPTANCE_ADVISORY_TASKS
        .iter()
        .map(|task| json_string(task))
        .collect::<Vec<_>>()
        .join(",");
    let families = integrated_acceptance_families()
        .iter()
        .map(|family| {
            let required = family
                .required_tasks
                .iter()
                .map(|task| json_string(task))
                .collect::<Vec<_>>()
                .join(",");
            let advisory = family
                .advisory_tasks
                .iter()
                .map(|task| json_string(task))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"title\":{},",
                    "\"rationale\":{},",
                    "\"required_tasks\":[{}],",
                    "\"advisory_tasks\":[{}]",
                    "}}"
                ),
                json_string(family.id),
                json_string(family.title),
                json_string(family.rationale),
                required,
                advisory,
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = integrated_acceptance_validation_steps()
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
    let deferred_scope = [
        "the bounded lane now groups one required cross-family acceptance path, but long-session soak thresholds and promotion policy still belong to g06.020",
        "unstable broader server-host recovery-overlap scenarios remain explicitly deferred until the integrated lane is real and bounded",
        "product-local QA dashboards, browser workflows, and exhaustive environment certification remain outside the shared Signal acceptance lane",
    ]
    .iter()
    .map(|scope| json_string(scope))
    .collect::<Vec<_>>()
    .join(",");
    format!(
        concat!(
            "{{",
            "\"lane\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"required_task_count\":{},",
            "\"required_tasks\":[{}],",
            "\"advisory_task_count\":{},",
            "\"advisory_tasks\":[{}],",
            "\"family_count\":{},",
            "\"families\":[{}],",
            "\"validation_step_count\":{},",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(INTEGRATED_ACCEPTANCE_LANE),
        json_string(INTEGRATED_ACCEPTANCE_CONTRACT_PATH),
        json_string(INTEGRATED_ACCEPTANCE_TASK),
        INTEGRATED_ACCEPTANCE_REQUIRED_TASKS.len(),
        required_tasks,
        INTEGRATED_ACCEPTANCE_ADVISORY_TASKS.len(),
        advisory_tasks,
        integrated_acceptance_families().len(),
        families,
        integrated_acceptance_validation_steps().len(),
        validation_steps,
        deferred_scope,
    )
}

fn print_integrated_acceptance_lane(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_integrated_acceptance_lane_text()),
        OutputFormat::Json => println!("{}", render_integrated_acceptance_lane_json()),
    }
}

fn render_g06_soak_lane_text() -> String {
    let mut rendered = format!(
        "g06_soak_lane: {G06_SOAK_LANE}\ncontract_path: {G06_SOAK_CONTRACT_PATH}\nacceptance_task: {G06_SOAK_ACCEPTANCE_TASK}\nrecords:\n"
    );
    for record in g06_soak_lane_records() {
        rendered.push_str(&format!(
            "- id: {}\n  status: {}\n  command: {}\n  typed_output: {}\n  rationale: {}\n",
            record.id, record.status, record.command, record.typed_output, record.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in g06_soak_lane_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("deferred_scope:\n");
    for scope in [
        "the broader server-host soak path remains deferred because the recovery-overlap attach limit still trips that scenario",
        "wider rerun counts and promotion thresholds still belong to later g06.020 closeout review work",
        "remote or distributed soak orchestration remains outside the shared bounded lane",
    ] {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_g06_soak_lane_json() -> String {
    let records = g06_soak_lane_records()
        .iter()
        .map(|record| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"status\":{},",
                    "\"command\":{},",
                    "\"typed_output\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(record.id),
                json_string(record.status),
                json_string(record.command),
                json_string(record.typed_output),
                json_string(record.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = g06_soak_lane_validation_steps()
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
    let deferred_scope = [
        "the broader server-host soak path remains deferred because the recovery-overlap attach limit still trips that scenario",
        "wider rerun counts and promotion thresholds still belong to later g06.020 closeout review work",
        "remote or distributed soak orchestration remains outside the shared bounded lane",
    ]
    .iter()
    .map(|scope| json_string(scope))
    .collect::<Vec<_>>()
    .join(",");
    format!(
        concat!(
            "{{",
            "\"lane\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"record_count\":{},",
            "\"records\":[{}],",
            "\"validation_steps\":[{}],",
            "\"deferred_scope\":[{}]",
            "}}"
        ),
        json_string(G06_SOAK_LANE),
        json_string(G06_SOAK_CONTRACT_PATH),
        json_string(G06_SOAK_ACCEPTANCE_TASK),
        g06_soak_lane_records().len(),
        records,
        validation_steps,
        deferred_scope,
    )
}

fn print_g06_soak_lane(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_g06_soak_lane_text()),
        OutputFormat::Json => println!("{}", render_g06_soak_lane_json()),
    }
}

fn render_host_edge_boundary_text() -> String {
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

fn render_host_edge_boundary_json() -> String {
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

fn print_host_edge_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_host_edge_boundary_text()),
        OutputFormat::Json => println!("{}", render_host_edge_boundary_json()),
    }
}

fn render_release_boundary_text() -> String {
    let mut rendered = format!(
        "release_boundary: {RELEASE_BOUNDARY}\nrelease_version: {}\nversion_source: {RELEASE_VERSION_SOURCE}\nchangelog_path: {RELEASE_CHANGELOG_PATH}\nexport_schema: {EXPORT_SCHEMA}\nexport_schema_version: {EXPORT_SCHEMA_VERSION}\nconformance_task: {RELEASE_CONFORMANCE_TASK}\nartifacts:\n",
        env!("CARGO_PKG_VERSION")
    );
    for artifact in release_boundary_artifacts() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  path_or_command: {}\n  rationale: {}\n",
            artifact.id,
            artifact.kind.label(),
            artifact.path_or_command,
            artifact.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in release_boundary_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("intentionally_unstable:\n");
    for scope in release_boundary_unstable_scopes() {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_release_boundary_json() -> String {
    let artifacts = release_boundary_artifacts()
        .iter()
        .map(|artifact| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"path_or_command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(artifact.id),
                json_string(artifact.kind.label()),
                json_string(artifact.path_or_command),
                json_string(artifact.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = release_boundary_validation_steps()
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
    let unstable = release_boundary_unstable_scopes()
        .iter()
        .map(|scope| json_string(scope))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"release_version\":{},",
            "\"version_source\":{},",
            "\"changelog_path\":{},",
            "\"export_schema\":{},",
            "\"export_schema_version\":{},",
            "\"conformance_task\":{},",
            "\"artifacts\":[{}],",
            "\"validation_steps\":[{}],",
            "\"intentionally_unstable\":[{}]",
            "}}"
        ),
        json_string(RELEASE_BOUNDARY),
        json_string(env!("CARGO_PKG_VERSION")),
        json_string(RELEASE_VERSION_SOURCE),
        json_string(RELEASE_CHANGELOG_PATH),
        json_string(EXPORT_SCHEMA),
        EXPORT_SCHEMA_VERSION,
        json_string(RELEASE_CONFORMANCE_TASK),
        artifacts,
        validation_steps,
        unstable,
    )
}

fn print_release_boundary(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_release_boundary_text()),
        OutputFormat::Json => println!("{}", render_release_boundary_json()),
    }
}

fn render_packaging_manifest_text() -> String {
    let mut rendered = format!(
        "packaging_manifest: {PACKAGING_MANIFEST}\nrelease_version: {}\nversion_source: {RELEASE_VERSION_SOURCE}\ncontract_path: {PACKAGING_MANIFEST_CONTRACT_PATH}\nacceptance_task: {PACKAGING_MANIFEST_ACCEPTANCE_TASK}\ninputs:\n",
        env!("CARGO_PKG_VERSION")
    );
    for input in packaging_manifest_inputs() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  path_or_command: {}\n  rationale: {}\n",
            input.id,
            input.kind.label(),
            input.path_or_command,
            input.rationale,
        ));
    }
    rendered.push_str("receipt_surfaces:\n");
    for receipt in packaging_receipt_surfaces() {
        rendered.push_str(&format!(
            "- id: {}\n  surface: {}\n  rationale: {}\n",
            receipt.id, receipt.surface, receipt.rationale,
        ));
    }
    rendered.push_str("validation_steps:\n");
    for step in packaging_manifest_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("unsupported_publication_paths:\n");
    for scope in packaging_manifest_unsupported_paths() {
        rendered.push_str(&format!("- {scope}\n"));
    }
    rendered
}

fn render_packaging_manifest_json() -> String {
    let inputs = packaging_manifest_inputs()
        .iter()
        .map(|input| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"path_or_command\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(input.id),
                json_string(input.kind.label()),
                json_string(input.path_or_command),
                json_string(input.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let receipts = packaging_receipt_surfaces()
        .iter()
        .map(|receipt| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"surface\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(receipt.id),
                json_string(receipt.surface),
                json_string(receipt.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let validation_steps = packaging_manifest_validation_steps()
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
    let unsupported = packaging_manifest_unsupported_paths()
        .iter()
        .map(|scope| json_string(scope))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"manifest\":{},",
            "\"release_version\":{},",
            "\"version_source\":{},",
            "\"contract_path\":{},",
            "\"acceptance_task\":{},",
            "\"inputs\":[{}],",
            "\"receipt_surfaces\":[{}],",
            "\"validation_steps\":[{}],",
            "\"unsupported_publication_paths\":[{}]",
            "}}"
        ),
        json_string(PACKAGING_MANIFEST),
        json_string(env!("CARGO_PKG_VERSION")),
        json_string(RELEASE_VERSION_SOURCE),
        json_string(PACKAGING_MANIFEST_CONTRACT_PATH),
        json_string(PACKAGING_MANIFEST_ACCEPTANCE_TASK),
        inputs,
        receipts,
        validation_steps,
        unsupported,
    )
}

fn print_packaging_manifest(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_packaging_manifest_text()),
        OutputFormat::Json => println!("{}", render_packaging_manifest_json()),
    }
}

fn render_downstream_automation_text() -> String {
    let mut rendered = format!(
        "downstream_automation_boundary: {DOWNSTREAM_AUTOMATION_BOUNDARY}\ncontract_path: {DOWNSTREAM_AUTOMATION_CONTRACT_PATH}\nmandatory_release_task: {DOWNSTREAM_AUTOMATION_MANDATORY_TASK}\noptional_depth_task: {DOWNSTREAM_AUTOMATION_OPTIONAL_TASK}\ncombined_task: {DOWNSTREAM_AUTOMATION_COMBINED_TASK}\nmandatory_release_acceptance:\n"
    );
    for fixture in downstream_automation_mandatory_fixtures() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  command: {}\n  typed_output: {}\n  rationale: {}\n",
            fixture.id,
            fixture.kind.label(),
            fixture.command,
            fixture.typed_output,
            fixture.rationale,
        ));
    }
    rendered.push_str("optional_confidence_depth:\n");
    for fixture in downstream_automation_optional_fixtures() {
        rendered.push_str(&format!(
            "- id: {}\n  kind: {}\n  command: {}\n  typed_output: {}\n  rationale: {}\n",
            fixture.id,
            fixture.kind.label(),
            fixture.command,
            fixture.typed_output,
            fixture.rationale,
        ));
    }
    rendered
}

fn render_downstream_automation_json() -> String {
    let mandatory = downstream_automation_mandatory_fixtures()
        .iter()
        .map(|fixture| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"command\":{},",
                    "\"typed_output\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(fixture.id),
                json_string(fixture.kind.label()),
                json_string(fixture.command),
                json_string(fixture.typed_output),
                json_string(fixture.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let optional = downstream_automation_optional_fixtures()
        .iter()
        .map(|fixture| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"kind\":{},",
                    "\"command\":{},",
                    "\"typed_output\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(fixture.id),
                json_string(fixture.kind.label()),
                json_string(fixture.command),
                json_string(fixture.typed_output),
                json_string(fixture.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"mandatory_release_task\":{},",
            "\"optional_depth_task\":{},",
            "\"combined_task\":{},",
            "\"mandatory_release_acceptance\":[{}],",
            "\"optional_confidence_depth\":[{}]",
            "}}"
        ),
        json_string(DOWNSTREAM_AUTOMATION_BOUNDARY),
        json_string(DOWNSTREAM_AUTOMATION_CONTRACT_PATH),
        json_string(DOWNSTREAM_AUTOMATION_MANDATORY_TASK),
        json_string(DOWNSTREAM_AUTOMATION_OPTIONAL_TASK),
        json_string(DOWNSTREAM_AUTOMATION_COMBINED_TASK),
        mandatory,
        optional,
    )
}

fn print_downstream_automation(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_downstream_automation_text()),
        OutputFormat::Json => println!("{}", render_downstream_automation_json()),
    }
}

fn render_downstream_fail_gates_text() -> String {
    let mut rendered = format!(
        "downstream_fail_gates: {DOWNSTREAM_FAIL_GATES}\ncontract_path: {DOWNSTREAM_AUTOMATION_CONTRACT_PATH}\nfail_gate_task: {DOWNSTREAM_FAIL_GATE_TASK}\nmandatory_release_task: {DOWNSTREAM_AUTOMATION_MANDATORY_TASK}\noptional_depth_task: {DOWNSTREAM_AUTOMATION_OPTIONAL_TASK}\nrules:\n"
    );
    for rule in downstream_fail_gate_rules() {
        rendered.push_str(&format!(
            "- id: {}\n  gate: {}\n  command: {}\n  blocks_release: {}\n  rationale: {}\n",
            rule.id, rule.gate, rule.command, rule.blocks_release, rule.rationale,
        ));
    }
    rendered.push_str("deferred_depth:\n");
    for record in downstream_deferred_depth_records() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  status: {}\n  rationale: {}\n",
            record.id, record.command, record.status, record.rationale,
        ));
    }
    rendered
}

fn render_downstream_fail_gates_json() -> String {
    let rules = downstream_fail_gate_rules()
        .iter()
        .map(|rule| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"gate\":{},",
                    "\"command\":{},",
                    "\"blocks_release\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(rule.id),
                json_string(rule.gate),
                json_string(rule.command),
                rule.blocks_release,
                json_string(rule.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let deferred = downstream_deferred_depth_records()
        .iter()
        .map(|record| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"command\":{},",
                    "\"status\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(record.id),
                json_string(record.command),
                json_string(record.status),
                json_string(record.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"boundary\":{},",
            "\"contract_path\":{},",
            "\"fail_gate_task\":{},",
            "\"mandatory_release_task\":{},",
            "\"optional_depth_task\":{},",
            "\"rules\":[{}],",
            "\"deferred_depth\":[{}]",
            "}}"
        ),
        json_string(DOWNSTREAM_FAIL_GATES),
        json_string(DOWNSTREAM_AUTOMATION_CONTRACT_PATH),
        json_string(DOWNSTREAM_FAIL_GATE_TASK),
        json_string(DOWNSTREAM_AUTOMATION_MANDATORY_TASK),
        json_string(DOWNSTREAM_AUTOMATION_OPTIONAL_TASK),
        rules,
        deferred,
    )
}

fn print_downstream_fail_gates(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_downstream_fail_gates_text()),
        OutputFormat::Json => println!("{}", render_downstream_fail_gates_json()),
    }
}

fn render_generation_closeout_text() -> String {
    let mut rendered = format!(
        "generation_closeout: {GENERATION_CLOSEOUT}\ngeneration: {GENERATION_CLOSEOUT_GENERATION}\ncontract_path: {GENERATION_CLOSEOUT_CONTRACT_PATH}\ncloseout_task: {GENERATION_CLOSEOUT_TASK}\npromotion_decision: {GENERATION_CLOSEOUT_PROMOTION_DECISION}\nintegrated_acceptance_command: cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json\ng06_soak_lane_command: cargo run -p signal-supervisor-tools -- --describe-g06-soak-lane --format=json\nnext_generation_path: {G07_README_PATH}\nnext_generation_status: {GENERATION_CLOSEOUT_NEXT_GENERATION_STATUS}\nnext_queue_status: {GENERATION_CLOSEOUT_NEXT_QUEUE_STATUS}\nvalidation_steps:\n"
    );
    for step in generation_closeout_validation_steps() {
        rendered.push_str(&format!(
            "- id: {}\n  command: {}\n  rationale: {}\n",
            step.id, step.command, step.rationale,
        ));
    }
    rendered.push_str("loophole_readiness_areas:\n");
    for area in generation_closeout_readiness_areas() {
        rendered.push_str(&format!(
            "- id: {}\n  status: {}\n  rationale: {}\n",
            area.id, area.status, area.rationale,
        ));
    }
    rendered.push_str("residual_risks:\n");
    for risk in generation_closeout_residual_risks() {
        rendered.push_str(&format!("- {risk}\n"));
    }
    rendered.push_str(&format!(
        "next_queue_summary: {}\n",
        generation_closeout_next_queue_summary()
    ));
    rendered
}

fn render_generation_closeout_json() -> String {
    let validation_steps = generation_closeout_validation_steps()
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
    let readiness_areas = generation_closeout_readiness_areas()
        .iter()
        .map(|area| {
            format!(
                concat!(
                    "{{",
                    "\"id\":{},",
                    "\"status\":{},",
                    "\"rationale\":{}",
                    "}}"
                ),
                json_string(area.id),
                json_string(area.status),
                json_string(area.rationale),
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let residual_risks = generation_closeout_residual_risks()
        .iter()
        .map(|risk| json_string(risk))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        concat!(
            "{{",
            "\"closeout\":{},",
            "\"generation\":{},",
            "\"contract_path\":{},",
            "\"closeout_task\":{},",
            "\"promotion_decision\":{},",
            "\"integrated_acceptance_command\":{},",
            "\"g06_soak_lane_command\":{},",
            "\"next_generation_path\":{},",
            "\"next_generation_status\":{},",
            "\"next_queue_status\":{},",
            "\"validation_steps\":[{}],",
            "\"loophole_readiness_areas\":[{}],",
            "\"residual_risks\":[{}],",
            "\"next_queue_summary\":{}",
            "}}"
        ),
        json_string(GENERATION_CLOSEOUT),
        json_string(GENERATION_CLOSEOUT_GENERATION),
        json_string(GENERATION_CLOSEOUT_CONTRACT_PATH),
        json_string(GENERATION_CLOSEOUT_TASK),
        json_string(GENERATION_CLOSEOUT_PROMOTION_DECISION),
        json_string(
            "cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json",
        ),
        json_string(
            "cargo run -p signal-supervisor-tools -- --describe-g06-soak-lane --format=json",
        ),
        json_string(G07_README_PATH),
        json_string(GENERATION_CLOSEOUT_NEXT_GENERATION_STATUS),
        json_string(GENERATION_CLOSEOUT_NEXT_QUEUE_STATUS),
        validation_steps,
        readiness_areas,
        residual_risks,
        json_string(generation_closeout_next_queue_summary()),
    )
}

fn print_generation_closeout(format: OutputFormat) {
    match format {
        OutputFormat::Text => println!("{}", render_generation_closeout_text()),
        OutputFormat::Json => println!("{}", render_generation_closeout_json()),
    }
}

fn json_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn json_option_debug<T: std::fmt::Debug>(value: Option<T>) -> String {
    match value {
        Some(value) => json_string(&format!("{value:?}")),
        None => "null".into(),
    }
}

fn print_report(
    format: OutputFormat,
    profile: HostProfile,
    scenario: Scenario,
    summary: String,
    profiling: &RuntimeProfilingReceipt,
    soak: &RuntimeSoakReceipt,
    report: RuntimeSupervisorReport,
) {
    match format {
        OutputFormat::Text => println!(
            "signal-supervisor-tools profile={profile:?} scenario={scenario:?}\n{summary}\nprofiling:\n{}\nsoak:\n{}\nsupervisor:\n{}",
            profiling.render_multiline(),
            soak.render_multiline(),
            report.render_multiline()
        ),
        OutputFormat::Json => println!(
            "{}",
            render_supervisor_export_json(profile, scenario, summary, profiling, soak, &report)
        ),
    }
}

fn run_local(
    format: OutputFormat,
    debug: ExportDebugOptions,
    scenario: Scenario,
) -> Result<(), String> {
    let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
    let mut host = LocalRuntimeHost::new(runtime);
    let summary = match scenario {
        Scenario::Default => host.boot_default(),
        Scenario::Timeout => host.boot_with_timeout_recovery(),
        Scenario::Crash => host.boot_with_crash_recovery(),
        Scenario::Heartbeat => host.boot_with_heartbeat_miss_recovery(),
        Scenario::Soak => host.boot_with_watchdog_soak(),
        Scenario::Mixed => host.boot_with_mixed_watchdog_soak(),
    }
    .map_err(|error| error.message)?;
    let report = host.supervisor_report();
    let host_report = host.host_supervisor_report();
    let profiling = host_report.profiling_receipt();
    let soak = host_report.soak_receipt();
    print_report(
        format,
        HostProfile::Local,
        scenario,
        match format {
            OutputFormat::Text => render_local_summary(&summary, debug),
            OutputFormat::Json => render_local_summary_json(&summary, debug),
        },
        &profiling,
        &soak,
        report,
    );
    Ok(())
}

fn run_server(
    format: OutputFormat,
    debug: ExportDebugOptions,
    scenario: Scenario,
) -> Result<(), String> {
    let runtime = SignalRuntime::new(RuntimeConfig::server(48_000, 512));
    let mut host = ServerRuntimeHost::new(runtime);
    let summary = match scenario {
        Scenario::Default => host.boot_default(),
        Scenario::Timeout => host.boot_with_timeout_recovery(),
        Scenario::Crash => host.boot_with_crash_recovery(),
        Scenario::Heartbeat => host.boot_with_heartbeat_miss_recovery(),
        Scenario::Soak => host.boot_with_watchdog_soak(),
        Scenario::Mixed => host.boot_with_mixed_watchdog_soak(),
    }
    .map_err(|error| error.message)?;
    let report = host.supervisor_report();
    let profiling = report.profiling_receipt();
    let soak = report.soak_receipt();
    print_report(
        format,
        HostProfile::Server,
        scenario,
        match format {
            OutputFormat::Text => render_server_summary(&summary, debug),
            OutputFormat::Json => render_server_summary_json(&summary, debug),
        },
        &profiling,
        &soak,
        report,
    );
    Ok(())
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliArgs, String> {
    let mut format = OutputFormat::Text;
    let mut debug = ExportDebugOptions::default();
    let mut describe_export = false;
    let mut describe_conformance_matrix = false;
    let mut describe_interruption_boundary = false;
    let mut describe_fault_diagnostic_boundary = false;
    let mut describe_critical_path_boundary = false;
    let mut describe_block_timing_boundary = false;
    let mut describe_deferred_work_policy_boundary = false;
    let mut describe_recording_continuity_boundary = false;
    let mut describe_offline_render_continuity_boundary = false;
    let mut describe_plugin_continuity_boundary = false;
    let mut describe_vst3_boundary = false;
    let mut describe_au_boundary = false;
    let mut describe_cross_adapter_parity_boundary = false;
    let mut describe_generic_event_boundary = false;
    let mut describe_recall_portability_boundary = false;
    let mut describe_device_supervision_boundary = false;
    let mut describe_clock_topology_boundary = false;
    let mut describe_external_io_boundary = false;
    let mut describe_media_service_boundary = false;
    let mut describe_analysis_metadata_boundary = false;
    let mut describe_integrated_acceptance_lane = false;
    let mut describe_g06_soak_lane = false;
    let mut describe_host_edge_boundary = false;
    let mut describe_release_boundary = false;
    let mut describe_packaging_manifest = false;
    let mut describe_downstream_automation = false;
    let mut describe_downstream_fail_gates = false;
    let mut describe_generation_closeout = false;
    let mut positional = Vec::new();

    for arg in args {
        if arg == "--json" {
            format = OutputFormat::Json;
            continue;
        }
        if arg == "--text" {
            format = OutputFormat::Text;
            continue;
        }
        if arg == "--include-payload" {
            debug.payload = true;
            continue;
        }
        if arg == "--describe-export" {
            describe_export = true;
            continue;
        }
        if arg == "--describe-conformance-matrix" {
            describe_conformance_matrix = true;
            continue;
        }
        if arg == "--describe-interruption-boundary" {
            describe_interruption_boundary = true;
            continue;
        }
        if arg == "--describe-fault-diagnostic-boundary" {
            describe_fault_diagnostic_boundary = true;
            continue;
        }
        if arg == "--describe-critical-path-boundary" {
            describe_critical_path_boundary = true;
            continue;
        }
        if arg == "--describe-block-timing-boundary" {
            describe_block_timing_boundary = true;
            continue;
        }
        if arg == "--describe-deferred-work-policy-boundary" {
            describe_deferred_work_policy_boundary = true;
            continue;
        }
        if arg == "--describe-recording-continuity-boundary" {
            describe_recording_continuity_boundary = true;
            continue;
        }
        if arg == "--describe-offline-render-continuity-boundary" {
            describe_offline_render_continuity_boundary = true;
            continue;
        }
        if arg == "--describe-plugin-continuity-boundary" {
            describe_plugin_continuity_boundary = true;
            continue;
        }
        if arg == "--describe-vst3-boundary" {
            describe_vst3_boundary = true;
            continue;
        }
        if arg == "--describe-au-boundary" {
            describe_au_boundary = true;
            continue;
        }
        if arg == "--describe-cross-adapter-parity-boundary" {
            describe_cross_adapter_parity_boundary = true;
            continue;
        }
        if arg == "--describe-generic-event-boundary" {
            describe_generic_event_boundary = true;
            continue;
        }
        if arg == "--describe-recall-portability-boundary" {
            describe_recall_portability_boundary = true;
            continue;
        }
        if arg == "--describe-device-supervision-boundary" {
            describe_device_supervision_boundary = true;
            continue;
        }
        if arg == "--describe-clock-topology-boundary" {
            describe_clock_topology_boundary = true;
            continue;
        }
        if arg == "--describe-external-io-boundary" {
            describe_external_io_boundary = true;
            continue;
        }
        if arg == "--describe-media-service-boundary" {
            describe_media_service_boundary = true;
            continue;
        }
        if arg == "--describe-analysis-metadata-boundary" {
            describe_analysis_metadata_boundary = true;
            continue;
        }
        if arg == "--describe-integrated-acceptance-lane" {
            describe_integrated_acceptance_lane = true;
            continue;
        }
        if arg == "--describe-g06-soak-lane" {
            describe_g06_soak_lane = true;
            continue;
        }
        if arg == "--describe-host-edge-boundary" {
            describe_host_edge_boundary = true;
            continue;
        }
        if arg == "--describe-release-boundary" {
            describe_release_boundary = true;
            continue;
        }
        if arg == "--describe-packaging-manifest" {
            describe_packaging_manifest = true;
            continue;
        }
        if arg == "--describe-downstream-automation" {
            describe_downstream_automation = true;
            continue;
        }
        if arg == "--describe-downstream-fail-gates" {
            describe_downstream_fail_gates = true;
            continue;
        }
        if arg == "--describe-generation-closeout" {
            describe_generation_closeout = true;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--format=") {
            format = OutputFormat::parse(value)?;
            continue;
        }
        positional.push(arg);
    }

    let describe_mode_count = [
        describe_export,
        describe_conformance_matrix,
        describe_interruption_boundary,
        describe_fault_diagnostic_boundary,
        describe_critical_path_boundary,
        describe_block_timing_boundary,
        describe_deferred_work_policy_boundary,
        describe_recording_continuity_boundary,
        describe_offline_render_continuity_boundary,
        describe_plugin_continuity_boundary,
        describe_vst3_boundary,
        describe_au_boundary,
        describe_cross_adapter_parity_boundary,
        describe_generic_event_boundary,
        describe_recall_portability_boundary,
        describe_device_supervision_boundary,
        describe_clock_topology_boundary,
        describe_external_io_boundary,
        describe_media_service_boundary,
        describe_analysis_metadata_boundary,
        describe_integrated_acceptance_lane,
        describe_g06_soak_lane,
        describe_host_edge_boundary,
        describe_release_boundary,
        describe_packaging_manifest,
        describe_downstream_automation,
        describe_downstream_fail_gates,
        describe_generation_closeout,
    ]
    .into_iter()
    .filter(|enabled| *enabled)
    .count();
    if describe_mode_count > 1 {
        return Err("describe modes are mutually exclusive".into());
    }

    if describe_export {
        if !positional.is_empty() {
            return Err(
                "`--describe-export` does not accept <profile> <scenario> positionals".into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeExport,
        });
    }

    if describe_conformance_matrix {
        if !positional.is_empty() {
            return Err(
                "`--describe-conformance-matrix` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeConformanceMatrix,
        });
    }

    if describe_interruption_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-interruption-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeInterruptionBoundary,
        });
    }

    if describe_fault_diagnostic_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-fault-diagnostic-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeFaultDiagnosticBoundary,
        });
    }

    if describe_critical_path_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-critical-path-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeCriticalPathBoundary,
        });
    }

    if describe_block_timing_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-block-timing-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeBlockTimingBoundary,
        });
    }

    if describe_deferred_work_policy_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-deferred-work-policy-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDeferredWorkPolicyBoundary,
        });
    }

    if describe_recording_continuity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-recording-continuity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeRecordingContinuityBoundary,
        });
    }

    if describe_offline_render_continuity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-offline-render-continuity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeOfflineRenderContinuityBoundary,
        });
    }

    if describe_plugin_continuity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-plugin-continuity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribePluginContinuityBoundary,
        });
    }

    if describe_vst3_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-vst3-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeVst3Boundary,
        });
    }

    if describe_au_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-au-boundary` does not accept <profile> <scenario> positionals".into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeAuBoundary,
        });
    }

    if describe_cross_adapter_parity_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-cross-adapter-parity-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeCrossAdapterParityBoundary,
        });
    }

    if describe_generic_event_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-generic-event-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeGenericEventBoundary,
        });
    }

    if describe_recall_portability_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-recall-portability-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeRecallPortabilityBoundary,
        });
    }

    if describe_device_supervision_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-device-supervision-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDeviceSupervisionBoundary,
        });
    }

    if describe_clock_topology_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-clock-topology-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeClockTopologyBoundary,
        });
    }

    if describe_external_io_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-external-io-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeExternalIoBoundary,
        });
    }

    if describe_media_service_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-media-service-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeMediaServiceBoundary,
        });
    }

    if describe_analysis_metadata_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-analysis-metadata-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeAnalysisMetadataBoundary,
        });
    }

    if describe_integrated_acceptance_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-integrated-acceptance-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeIntegratedAcceptanceLane,
        });
    }

    if describe_g06_soak_lane {
        if !positional.is_empty() {
            return Err(
                "`--describe-g06-soak-lane` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeG06SoakLane,
        });
    }

    if describe_host_edge_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-host-edge-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeHostEdgeBoundary,
        });
    }

    if describe_release_boundary {
        if !positional.is_empty() {
            return Err(
                "`--describe-release-boundary` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeReleaseBoundary,
        });
    }

    if describe_downstream_automation {
        if !positional.is_empty() {
            return Err(
                "`--describe-downstream-automation` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDownstreamAutomation,
        });
    }

    if describe_downstream_fail_gates {
        if !positional.is_empty() {
            return Err(
                "`--describe-downstream-fail-gates` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeDownstreamFailGates,
        });
    }

    if describe_generation_closeout {
        if !positional.is_empty() {
            return Err(
                "`--describe-generation-closeout` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribeGenerationCloseout,
        });
    }

    if describe_packaging_manifest {
        if !positional.is_empty() {
            return Err(
                "`--describe-packaging-manifest` does not accept <profile> <scenario> positionals"
                    .into(),
            );
        }
        return Ok(CliArgs {
            format,
            debug,
            mode: CliMode::DescribePackagingManifest,
        });
    }

    if positional.len() != 2 {
        return Err("expected <profile> <scenario>".into());
    }

    Ok(CliArgs {
        format,
        debug,
        mode: CliMode::Run {
            profile: HostProfile::parse(&positional[0])?,
            scenario: Scenario::parse(&positional[1])?,
        },
    })
}

fn main() {
    let args = match parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            print_usage();
            std::process::exit(2);
        }
    };

    let result = match args.mode {
        CliMode::Run { profile, scenario } => match profile {
            HostProfile::Local => run_local(args.format, args.debug, scenario),
            HostProfile::Server => run_server(args.format, args.debug, scenario),
        },
        CliMode::DescribeExport => {
            print_export_description(args.format);
            Ok(())
        }
        CliMode::DescribeConformanceMatrix => {
            print_conformance_matrix(args.format);
            Ok(())
        }
        CliMode::DescribeInterruptionBoundary => {
            print_interruption_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeFaultDiagnosticBoundary => {
            print_fault_diagnostic_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeCriticalPathBoundary => {
            print_critical_path_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeBlockTimingBoundary => {
            print_block_timing_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeDeferredWorkPolicyBoundary => {
            print_deferred_work_policy_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeRecordingContinuityBoundary => {
            print_recording_continuity_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeOfflineRenderContinuityBoundary => {
            print_offline_render_continuity_boundary(args.format);
            Ok(())
        }
        CliMode::DescribePluginContinuityBoundary => {
            print_plugin_continuity_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeVst3Boundary => {
            print_vst3_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeAuBoundary => {
            print_au_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeCrossAdapterParityBoundary => {
            print_cross_adapter_parity_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeGenericEventBoundary => {
            print_generic_event_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeRecallPortabilityBoundary => {
            print_recall_portability_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeDeviceSupervisionBoundary => {
            print_device_supervision_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeClockTopologyBoundary => {
            print_clock_topology_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeExternalIoBoundary => {
            print_external_io_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeMediaServiceBoundary => {
            print_media_service_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeAnalysisMetadataBoundary => {
            print_analysis_metadata_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeIntegratedAcceptanceLane => {
            print_integrated_acceptance_lane(args.format);
            Ok(())
        }
        CliMode::DescribeG06SoakLane => {
            print_g06_soak_lane(args.format);
            Ok(())
        }
        CliMode::DescribeHostEdgeBoundary => {
            print_host_edge_boundary(args.format);
            Ok(())
        }
        CliMode::DescribeReleaseBoundary => {
            print_release_boundary(args.format);
            Ok(())
        }
        CliMode::DescribePackagingManifest => {
            print_packaging_manifest(args.format);
            Ok(())
        }
        CliMode::DescribeDownstreamAutomation => {
            print_downstream_automation(args.format);
            Ok(())
        }
        CliMode::DescribeDownstreamFailGates => {
            print_downstream_fail_gates(args.format);
            Ok(())
        }
        CliMode::DescribeGenerationCloseout => {
            print_generation_closeout(args.format);
            Ok(())
        }
    };

    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        parse_args, render_analysis_metadata_boundary_json, render_analysis_metadata_boundary_text,
        render_au_boundary_json, render_au_boundary_text, render_block_timing_boundary_json,
        render_block_timing_boundary_text, render_clock_topology_boundary_json,
        render_clock_topology_boundary_text, render_conformance_matrix_json,
        render_conformance_matrix_text, render_critical_path_boundary_json,
        render_critical_path_boundary_text, render_cross_adapter_parity_boundary_json,
        render_cross_adapter_parity_boundary_text, render_deferred_work_policy_boundary_json,
        render_deferred_work_policy_boundary_text, render_device_supervision_boundary_json,
        render_device_supervision_boundary_text, render_downstream_automation_json,
        render_downstream_automation_text, render_downstream_fail_gates_json,
        render_downstream_fail_gates_text, render_export_description_json,
        render_export_description_text, render_external_io_boundary_json,
        render_external_io_boundary_text, render_fault_diagnostic_boundary_json,
        render_fault_diagnostic_boundary_text, render_g06_soak_lane_json,
        render_g06_soak_lane_text, render_generation_closeout_json,
        render_generation_closeout_text, render_generic_event_boundary_json,
        render_generic_event_boundary_text, render_host_edge_boundary_json,
        render_host_edge_boundary_text, render_integrated_acceptance_lane_json,
        render_integrated_acceptance_lane_text, render_interruption_boundary_json,
        render_interruption_boundary_text, render_media_service_boundary_json,
        render_media_service_boundary_text, render_offline_render_continuity_boundary_json,
        render_offline_render_continuity_boundary_text, render_packaging_manifest_json,
        render_packaging_manifest_text, render_plugin_continuity_boundary_json,
        render_plugin_continuity_boundary_text, render_recall_portability_boundary_json,
        render_recall_portability_boundary_text, render_recording_continuity_boundary_json,
        render_recording_continuity_boundary_text, render_release_boundary_json,
        render_release_boundary_text, render_supervisor_export_json, render_vst3_boundary_json,
        render_vst3_boundary_text, CliArgs, CliMode, ExportDebugOptions, HostProfile,
        HostSummaryDebugSection, OutputFormat, Scenario,
    };
    use signal_hardware::{
        AudioSampleFormat, BackendHealth, HardwareDiagnosticsSnapshot, HardwareLifecycleContract,
        HardwareLifecycleOwnership, HardwareRestartPolicy,
    };
    use signal_host_local::host::{
        LocalAudioPumpSummary, LocalAudioStreamState, LocalAudioTransferPolicy,
        LocalExecutionSummary, LocalFaultSummary, LocalHardwareSummary, LocalPayloadSummary,
        LocalTransportSummary,
    };
    use signal_host_local::{LocalRuntimeHostSummary, RecoveryRestartIntent};
    use signal_plugin::{
        CompletionState, PluginFeature, PluginFormat, PluginIoLayout, PluginLifecycleContract,
        PluginProcessingContract, PluginStateContract, WatchdogTriggerReason,
    };
    use signal_runtime::{
        BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
        HandshakeRequest, HeartbeatCycleStage, PluginSandboxLifecycleStage, PluginSandboxSpec,
        PluginSandboxTransportStage, PluginScanRequest, RuntimeConfig, RuntimeConfigRequest,
        RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink, RuntimeExecutionTopologySummary,
        RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
        RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
        RuntimeHostClockFallbackState, RuntimeHostClockSource, RuntimeHostClockTransitionState,
        RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
        RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
        RuntimeHostLifecycleOwnership, RuntimeHostRestartPolicy, RuntimeLifecycleApi,
        RuntimeMediaAssetRegistration, RuntimeObservationApi, RuntimeOfflineRenderPurgeRequest,
        RuntimeOfflineRenderRequest, RuntimePluginDiscoveredTypeRecord,
        RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
        RuntimeSupervisorReport, RuntimeWatchdogTrigger, SafeModeRequest,
        SandboxOperationFailureStage, SignalRuntime, StopReason, TransportDispatchState,
        TransportHeartbeatFreshness, TransportSessionState, WatchdogRestartRecord,
    };

    fn sample_discovered_type_record() -> RuntimePluginDiscoveredTypeRecord {
        RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:clap:export-consumer".into(),
            plugin_id: "com.signal.export-consumer".into(),
            vendor: "Signal".into(),
            name: "Signal Export Consumer".into(),
            format: PluginFormat::Clap,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::AudioEffect, PluginFeature::Utility],
            default_io_layout: PluginIoLayout {
                audio_inputs: 2,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 1,
            },
            audio_bus_count: 2,
            parameter_count: 12,
            state_contract: PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: true,
                exposes_tail: true,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 4096,
                sample_accurate_automation: true,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: true,
                silence_aware: true,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: false,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: true,
            },
            summary: "supervisor export discovered plugin".into(),
        }
    }

    fn sample_backend_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
        RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:vst3:export-instrument".into(),
            plugin_id: "com.signal.export-instrument".into(),
            vendor: "Signal".into(),
            name: "Signal Export Instrument".into(),
            format: PluginFormat::Vst3,
            version: Some("2.0.0".into()),
            features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
            default_io_layout: PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            audio_bus_count: 1,
            parameter_count: 24,
            state_contract: PluginStateContract {
                supports_snapshot: false,
                supports_reset: true,
                supports_bypass: false,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 2048,
                sample_accurate_automation: false,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: true,
                supports_prepare: true,
                supports_activate: false,
                supports_reset_while_active: false,
            },
            summary: "supervisor export backend breadth plugin".into(),
        }
    }

    fn sample_au_breadth_record() -> RuntimePluginDiscoveredTypeRecord {
        RuntimePluginDiscoveredTypeRecord {
            plugin_type_id: "plugin:au:export-au".into(),
            plugin_id: "com.signal.export-au".into(),
            vendor: "Signal".into(),
            name: "Signal Export AU".into(),
            format: PluginFormat::Au,
            version: Some("1.0.0".into()),
            features: vec![PluginFeature::Instrument, PluginFeature::Analyzer],
            default_io_layout: PluginIoLayout {
                audio_inputs: 0,
                audio_outputs: 2,
                midi_inputs: 1,
                midi_outputs: 0,
            },
            audio_bus_count: 1,
            parameter_count: 10,
            state_contract: PluginStateContract {
                supports_snapshot: true,
                supports_reset: true,
                supports_bypass: true,
                exposes_latency: false,
                exposes_tail: true,
            },
            processing_contract: PluginProcessingContract {
                max_block_frames: 2048,
                sample_accurate_automation: false,
                accepts_midi: true,
                accepts_note_events: true,
                supports_note_expression: true,
                produces_midi: false,
                silence_aware: false,
            },
            lifecycle_contract: PluginLifecycleContract {
                requires_main_thread_for_state: true,
                supports_prepare: true,
                supports_activate: true,
                supports_reset_while_active: false,
            },
            summary: "supervisor export au breadth plugin".into(),
        }
    }

    fn sample_integrated_acceptance_host_io() -> RuntimeHostIoSummary {
        RuntimeHostIoSummary {
            hardware: RuntimeHostHardwareSummary {
                backend_name: "coreaudio".into(),
                device_id: "device:integrated-acceptance".into(),
                device_name: "Integrated Acceptance Device".into(),
                sample_rate: 48_000,
                buffer_size: 256,
                output_channels: 2,
                sample_format: AudioSampleFormat::F32,
                simulated: false,
                backend_health: BackendHealth::Healthy,
                xrun_count: 0,
                callback_overrun_count: 0,
                device_loss_count: 0,
                restart_attempt_count: 0,
                restart_failure_count: 0,
            },
            audio_pump: RuntimeHostAudioPumpSummary {
                stream_state: RuntimeHostAudioStreamState::Running,
                transfer_policy: RuntimeHostAudioTransferPolicy {
                    max_callback_frames: 256,
                    max_transfer_channels: 2,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 12,
                total_callback_frames: 3_072,
                total_runtime_output_frames: 3_072,
                copied_output_samples: 6_144,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: Some(0.35),
                last_runtime_graph_id: Some("graph:integrated-acceptance".into()),
            },
            clocking: RuntimeHostClockingSummary {
                clock_source: RuntimeHostClockSource::Internal,
                ownership: RuntimeHostLifecycleOwnership::HostDrivenCallback,
                restart_policy: RuntimeHostRestartPolicy::HostMustRestart,
                processing_sample_rate_hz: 44_100,
                hardware_sample_rate_hz: 48_000,
                clock_domain: RuntimeHostClockDomain::CrossClock,
                fallback_state: RuntimeHostClockFallbackState::RuntimeResampled,
                transition_state: RuntimeHostClockTransitionState::EnteredCrossClockFallback,
                drift_state: RuntimeHostClockDriftState::CrossClockManaged,
                discontinuity_state: RuntimeHostClockDiscontinuityState::Reconfigured,
                duplex_mismatch_state: RuntimeHostDuplexMismatchState::CrossClockDiverged,
                endpoint_topology: RuntimeHostEndpointTopology::Duplex,
                partial_availability: false,
                crossing_required: true,
                callback_interval_ms: 5.333,
            },
            latency: RuntimeHostLatencySummary {
                input_latency_samples: Some(128),
                output_latency_samples: 256,
                round_trip_latency_samples: Some(384),
                graph_latency_samples: 24,
                estimated_output_latency_samples: 280,
                estimated_round_trip_latency_samples: Some(408),
                output_latency_ms: 5.333,
                graph_latency_ms: 0.5,
                estimated_output_latency_ms: 5.833,
                estimated_round_trip_latency_ms: Some(8.5),
            },
            runtime_graph_id_matches_pump: true,
        }
    }

    fn integrated_acceptance_media_fixture_path(label: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for media fixture paths")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "signal-supervisor-tools-integrated-{label}-{}-{unique}.wav",
            std::process::id()
        ))
    }

    fn write_integrated_acceptance_test_wav(path: &Path) {
        let channels = 1u16;
        let sample_rate = 48_000u32;
        let bits_per_sample = 16u16;
        let frame_count = 128u32;
        let block_align = channels * (bits_per_sample / 8);
        let byte_rate = sample_rate * block_align as u32;
        let data_size = frame_count * block_align as u32;
        let riff_size = 36 + data_size;
        let mut bytes = Vec::with_capacity((44 + data_size) as usize);
        bytes.extend_from_slice(b"RIFF");
        bytes.extend_from_slice(&riff_size.to_le_bytes());
        bytes.extend_from_slice(b"WAVE");
        bytes.extend_from_slice(b"fmt ");
        bytes.extend_from_slice(&16u32.to_le_bytes());
        bytes.extend_from_slice(&1u16.to_le_bytes());
        bytes.extend_from_slice(&channels.to_le_bytes());
        bytes.extend_from_slice(&sample_rate.to_le_bytes());
        bytes.extend_from_slice(&byte_rate.to_le_bytes());
        bytes.extend_from_slice(&block_align.to_le_bytes());
        bytes.extend_from_slice(&bits_per_sample.to_le_bytes());
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&data_size.to_le_bytes());
        for index in 0..frame_count {
            let sample =
                (((index as f32 / (frame_count - 1) as f32) * 2.0) - 1.0) * i16::MAX as f32 * 0.5;
            bytes.extend_from_slice(&(sample as i16).to_le_bytes());
        }
        fs::write(path, bytes).expect("integrated acceptance media fixture should be written");
    }

    fn sample_local_summary() -> LocalRuntimeHostSummary {
        LocalRuntimeHostSummary {
            backend_name: "coreaudio",
            hardware: LocalHardwareSummary {
                device_id: "coreaudio:default-output".into(),
                device_name: "CoreAudio Default Output".into(),
                sample_rate: 48_000,
                buffer_size: 512,
                output_channels: 2,
                sample_format: AudioSampleFormat::F32,
                lifecycle: HardwareLifecycleContract {
                    ownership: HardwareLifecycleOwnership::HostDrivenCallback,
                    restart_policy: HardwareRestartPolicy::HostMustRestart,
                },
                simulated: false,
                backend_diagnostics: HardwareDiagnosticsSnapshot::healthy(),
            },
            audio_pump: LocalAudioPumpSummary {
                stream_state: LocalAudioStreamState::Running,
                transfer_policy: LocalAudioTransferPolicy {
                    max_callback_frames: 512,
                    max_transfer_channels: 2,
                    zero_fill_unwritten_output: true,
                },
                callback_count: 3,
                last_callback_index: Some(2),
                total_callback_frames: 1536,
                total_runtime_output_frames: 1536,
                copied_output_samples: 3072,
                zero_filled_output_samples: 0,
                dropped_output_samples: 0,
                last_callback_output_peak: Some(0.8),
                last_runtime_graph_id: Some("signal.host.local.demo".into()),
            },
            scan_roots: vec!["/plugins".into()],
            execution: LocalExecutionSummary {
                control_requests: 4,
                control_responses: 4,
                heartbeat_responses: 2,
                processed_blocks: 3,
                engine_processed_blocks: 3,
                last_control_message: "activateInstance".into(),
                last_completion_state: CompletionState::Completed,
                last_block_sequence: 7,
                last_engine_graph_id: Some("signal.host.local.demo".into()),
                last_engine_output_peak: Some(0.8),
                last_engine_output_rms: Some(0.42),
                processing_epoch: 2,
                restart_count: 1,
                teardown_count: 1,
                last_recovery_intent: Some(RecoveryRestartIntent::WatchdogRecovery),
                last_stop_reason: Some(StopReason::DegradedModeRecovery),
                last_plugin_state: None,
            },
            transport: LocalTransportSummary {
                sandbox_id: "sandbox-1".into(),
                shared_memory_lease_id: "lease-1".into(),
                shared_memory_region_id: "region-1".into(),
                shared_memory_path: "/tmp/signal-region-1".into(),
                shared_memory_bytes: 4096,
            },
            topology: RuntimeExecutionTopologySummary::default(),
            plugin_dispatch: None,
            last_payload: LocalPayloadSummary {
                event_count: 6,
                parameter_event_count: 2,
                parameter_gesture_event_count: 2,
                parameter_modulation_event_count: 1,
                note_event_count: 1,
                note_expression_event_count: 1,
                midi_event_count: 1,
                generated_event_bytes: 128,
                first_output_sample: Some(0.5),
            },
            faults: LocalFaultSummary {
                deadline_misses: 1,
                heartbeat_misses: 0,
                watchdog_triggered: true,
                watchdog_trigger_reason: Some(WatchdogTriggerReason::DeadlineMisses),
            },
        }
    }

    #[test]
    fn parses_profiles() {
        assert_eq!(HostProfile::parse("local"), Ok(HostProfile::Local));
        assert_eq!(HostProfile::parse("server"), Ok(HostProfile::Server));
    }

    #[test]
    fn parses_scenarios() {
        assert_eq!(Scenario::parse("default"), Ok(Scenario::Default));
        assert_eq!(Scenario::parse("mixed"), Ok(Scenario::Mixed));
        assert_eq!(Scenario::parse("soak"), Ok(Scenario::Soak));
    }

    #[test]
    fn parses_json_flag_and_positionals() {
        assert_eq!(
            parse_args(["--format=json".into(), "local".into(), "mixed".into(),]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::Run {
                    profile: HostProfile::Local,
                    scenario: Scenario::Mixed,
                },
            })
        );
    }

    #[test]
    fn rejects_missing_positionals() {
        let error = parse_args(["local".into()]).unwrap_err();
        assert!(error.contains("expected"));
    }

    #[test]
    fn parse_args_supports_short_json_flag() {
        assert_eq!(
            parse_args(["--json".into(), "server".into(), "soak".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::Run {
                    profile: HostProfile::Server,
                    scenario: Scenario::Soak,
                },
            })
        );
    }

    #[test]
    fn parse_args_supports_include_payload_flag() {
        assert_eq!(
            parse_args([
                "--json".into(),
                "--include-payload".into(),
                "local".into(),
                "default".into(),
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: true },
                mode: CliMode::Run {
                    profile: HostProfile::Local,
                    scenario: Scenario::Default,
                },
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_export_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-export".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeExport,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_export() {
        let error =
            parse_args(["--describe-export".into(), "local".into(), "default".into()]).unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_conformance_matrix_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-conformance-matrix".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeConformanceMatrix,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_conformance_matrix() {
        let error = parse_args([
            "--describe-conformance-matrix".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_interruption_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-interruption-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeInterruptionBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_interruption_boundary() {
        let error = parse_args([
            "--describe-interruption-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_fault_diagnostic_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-fault-diagnostic-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeFaultDiagnosticBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_fault_diagnostic_boundary() {
        let error = parse_args([
            "--describe-fault-diagnostic-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_critical_path_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-critical-path-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeCriticalPathBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_critical_path_boundary() {
        let error = parse_args([
            "--describe-critical-path-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_block_timing_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-block-timing-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeBlockTimingBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_block_timing_boundary() {
        let error = parse_args([
            "--describe-block-timing-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_deferred_work_policy_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-deferred-work-policy-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDeferredWorkPolicyBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_deferred_work_policy_boundary() {
        let error = parse_args([
            "--describe-deferred-work-policy-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_recording_continuity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-recording-continuity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeRecordingContinuityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_recording_continuity_boundary() {
        let error = parse_args([
            "--describe-recording-continuity-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_offline_render_continuity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-offline-render-continuity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeOfflineRenderContinuityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_offline_render_continuity_boundary() {
        let error = parse_args([
            "--describe-offline-render-continuity-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_plugin_continuity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-plugin-continuity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribePluginContinuityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_vst3_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-vst3-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeVst3Boundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_au_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-au-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeAuBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_cross_adapter_parity_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-cross-adapter-parity-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeCrossAdapterParityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_generic_event_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-generic-event-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeGenericEventBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_recall_portability_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-recall-portability-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeRecallPortabilityBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_device_supervision_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-device-supervision-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDeviceSupervisionBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_clock_topology_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-clock-topology-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeClockTopologyBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_external_io_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-external-io-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeExternalIoBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_media_service_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-media-service-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeMediaServiceBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_analysis_metadata_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-analysis-metadata-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeAnalysisMetadataBoundary,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_integrated_acceptance_lane_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-integrated-acceptance-lane".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeIntegratedAcceptanceLane,
            })
        );
    }

    #[test]
    fn parse_args_supports_describe_g06_soak_lane_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-g06-soak-lane".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeG06SoakLane,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_device_supervision_boundary() {
        let error = parse_args([
            "--describe-device-supervision-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_clock_topology_boundary() {
        let error = parse_args([
            "--describe-clock-topology-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_external_io_boundary() {
        let error = parse_args([
            "--describe-external-io-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_media_service_boundary() {
        let error = parse_args([
            "--describe-media-service-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_analysis_metadata_boundary() {
        let error = parse_args([
            "--describe-analysis-metadata-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_integrated_acceptance_lane() {
        let error = parse_args([
            "--describe-integrated-acceptance-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_g06_soak_lane() {
        let error = parse_args([
            "--describe-g06-soak-lane".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_plugin_continuity_boundary() {
        let error = parse_args([
            "--describe-plugin-continuity-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_rejects_multiple_describe_modes() {
        let error = parse_args([
            "--describe-export".into(),
            "--describe-conformance-matrix".into(),
        ])
        .unwrap_err();
        assert!(error.contains("mutually exclusive"));
    }

    #[test]
    fn parse_args_supports_describe_host_edge_boundary_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-host-edge-boundary".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeHostEdgeBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_host_edge_boundary() {
        let error = parse_args([
            "--describe-host-edge-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_release_boundary_mode() {
        assert_eq!(
            parse_args(["--format=json".into(), "--describe-release-boundary".into()]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeReleaseBoundary,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_release_boundary() {
        let error = parse_args([
            "--describe-release-boundary".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_packaging_manifest_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-packaging-manifest".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribePackagingManifest,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_packaging_manifest() {
        let error = parse_args([
            "--describe-packaging-manifest".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_downstream_automation_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-downstream-automation".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDownstreamAutomation,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_downstream_automation() {
        let error = parse_args([
            "--describe-downstream-automation".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_downstream_fail_gates_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-downstream-fail-gates".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeDownstreamFailGates,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_downstream_fail_gates() {
        let error = parse_args([
            "--describe-downstream-fail-gates".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn parse_args_supports_describe_generation_closeout_mode() {
        assert_eq!(
            parse_args([
                "--format=json".into(),
                "--describe-generation-closeout".into()
            ]),
            Ok(CliArgs {
                format: OutputFormat::Json,
                debug: ExportDebugOptions { payload: false },
                mode: CliMode::DescribeGenerationCloseout,
            })
        );
    }

    #[test]
    fn parse_args_rejects_positionals_with_describe_generation_closeout() {
        let error = parse_args([
            "--describe-generation-closeout".into(),
            "local".into(),
            "default".into(),
        ])
        .unwrap_err();
        assert!(error.contains("does not accept"));
    }

    #[test]
    fn only_payload_is_currently_supported_as_debug_section() {
        assert!(ExportDebugOptions { payload: true }.supports(HostSummaryDebugSection::Payload));
        assert_eq!(HostSummaryDebugSection::Payload.label(), "payload");
    }

    #[test]
    fn export_description_text_reports_frozen_policy() {
        let rendered = render_export_description_text();
        assert!(rendered.contains("schema: signal.supervisor.export"));
        assert!(rendered.contains("schema_version: 1"));
        assert!(rendered.contains("default_host_summary_sections: execution,transport,faults"));
        assert!(rendered.contains("supported_debug_sections: payload"));
    }

    #[test]
    fn export_description_json_reports_frozen_policy() {
        let rendered = render_export_description_json();
        assert!(rendered.contains("\"schema\":\"signal.supervisor.export\""));
        assert!(rendered.contains("\"schema_version\":1"));
        assert!(rendered.contains(
            "\"default_host_summary_sections\":[\"execution\",\"transport\",\"faults\"]"
        ));
        assert!(rendered.contains("\"supported_debug_sections\":[\"payload\"]"));
    }

    #[test]
    fn conformance_matrix_text_reports_runnable_consumer_boundary() {
        let rendered = render_conformance_matrix_text();
        assert!(rendered.contains("consumer_conformance_matrix:"));
        assert!(rendered.contains("runtime-public-contract-boundary"));
        assert!(rendered.contains("supervisor-export-discovery-consumer"));
        assert!(rendered.contains("plugin-backend-breadth-coverage"));
        assert!(rendered.contains("shared-host-edge-consumer"));
        assert!(rendered.contains("runtime-supervisor-report-demo"));
        assert!(rendered.contains("supervisor-export-schema-description"));
        assert!(rendered.contains("cargo test -p signal-runtime public_runtime_contract_boundary_is_consumable_from_reexports"));
        assert!(rendered.contains("effigy acceptance:plugin-backend-breadth"));
        assert!(rendered.contains("effigy acceptance:host-edge-consumer"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-conformance-matrix --format=json"
        ));
    }

    #[test]
    fn conformance_matrix_json_reports_runnable_consumer_boundary() {
        let rendered = render_conformance_matrix_json();
        assert!(rendered.contains("\"matrix\":\"signal.consumer.conformance\""));
        assert!(rendered.contains("\"entry_count\":6"));
        assert!(rendered.contains("\"id\":\"runtime-public-contract-boundary\""));
        assert!(rendered.contains("\"kind\":\"export-consumer-test\""));
        assert!(rendered.contains("\"crate\":\"signal-supervisor-tools\""));
        assert!(rendered.contains("\"id\":\"plugin-backend-breadth-coverage\""));
        assert!(rendered.contains("\"id\":\"shared-host-edge-consumer\""));
        assert!(rendered.contains(
            "\"command\":\"cargo run -p signal-runtime --example supervisor_report_demo\""
        ));
    }

    #[test]
    fn interruption_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_interruption_boundary_text();
        assert!(rendered.contains("interruption_boundary: signal.runtime.interruption-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:interruption-boundary"));
        assert!(rendered.contains("surface: RuntimeObservationReport::fault_status"));
        assert!(rendered.contains("surface: RuntimeDeferredServiceReceipt::interruption_class"));
        assert!(rendered.contains("surface: supervisor_report() -> RuntimeSupervisorReport"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_interruption_boundary_reports_restartable_runtime_state"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-interruption-boundary --format=json"
        ));
    }

    #[test]
    fn interruption_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_interruption_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.interruption-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/012-runtime-interruption-taxonomy-and-resumability-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:interruption-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-fault-status\""));
        assert!(rendered.contains("\"id\":\"offline-render-execution-interruption-receipt\""));
        assert!(rendered.contains("\"id\":\"shared-host-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-resumable-deferred-proof\""));
    }

    #[test]
    fn fault_diagnostic_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_fault_diagnostic_boundary_text();
        assert!(rendered
            .contains("fault_diagnostic_boundary: signal.runtime.fault-diagnostic-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:fault-diagnostic-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::fault_diagnostic_receipt and RuntimeSupervisorReport::observation.fault_diagnostic_receipt"
        ));
        assert!(rendered.contains("surface: RuntimeProfilingReceipt::fault_diagnostic_receipt"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_fault_diagnostic_boundary_reports_canonical_runtime_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-fault-diagnostic-boundary --format=json"
        ));
    }

    #[test]
    fn fault_diagnostic_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_fault_diagnostic_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.fault-diagnostic-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/016-runtime-fault-cause-attribution-and-diagnostic-receipt-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:fault-diagnostic-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-observation-fault-diagnostic\""));
        assert!(rendered.contains("\"id\":\"runtime-profiling-fault-diagnostic\""));
        assert!(rendered.contains("\"id\":\"shared-host-fault-diagnostic-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-fault-diagnostic-proof\""));
    }

    #[test]
    fn critical_path_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_critical_path_boundary_text();
        assert!(rendered.contains("critical_path_boundary: signal.runtime.critical-path-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:critical-path-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::performance_snapshot() and RuntimeSupervisorReport::performance_snapshot()"
        ));
        assert!(rendered.contains("surface: RuntimePerformanceTraceReceipt"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_critical_path_boundary_reports_bounded_hotspot_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-critical-path-boundary --format=json"
        ));
    }

    #[test]
    fn critical_path_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_critical_path_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.critical-path-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/018-graph-critical-path-hot-node-and-worker-lane-instrumentation-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:critical-path-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-performance-hotspot-report\""));
        assert!(rendered.contains("\"id\":\"runtime-performance-trace-digest\""));
        assert!(rendered.contains("\"id\":\"shared-host-critical-path-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-critical-path-proof\""));
    }

    #[test]
    fn block_timing_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_block_timing_boundary_text();
        assert!(rendered.contains("block_timing_boundary: signal.runtime.block-timing-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:block-timing-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::engine_block_snapshot and RuntimeSupervisorReport::observation.engine_block_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_block_timing_boundary_reports_bounded_runtime_measurements"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-block-timing-boundary --format=json"
        ));
    }

    #[test]
    fn block_timing_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_block_timing_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.block-timing-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/017-per-block-execution-timing-and-pressure-snapshot-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:block-timing-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-engine-block-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-performance-digests\""));
        assert!(rendered.contains("\"id\":\"shared-host-block-timing-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-block-timing-proof\""));
    }

    #[test]
    fn deferred_work_policy_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_deferred_work_policy_boundary_text();
        assert!(rendered.contains(
            "deferred_work_policy_boundary: signal.runtime.deferred-work-policy-boundary"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:deferred-work-policy-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::last_deferred_service_receipt and RuntimeSupervisorReport::observation.last_deferred_service_receipt"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::performance_snapshot(), RuntimeSupervisorReport::performance_snapshot(), and RuntimePerformanceTraceReceipt"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_deferred_work_policy_boundary_reports_runtime_owned_scheduler_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-deferred-work-policy-boundary --format=json"
        ));
    }

    #[test]
    fn deferred_work_policy_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_deferred_work_policy_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.deferred-work-policy-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/019-deferred-work-scheduler-priority-backpressure-and-cancellation-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:deferred-work-policy-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-deferred-service-policy-receipt\""));
        assert!(rendered.contains("\"id\":\"runtime-performance-policy-digests\""));
        assert!(rendered.contains("\"id\":\"shared-host-deferred-policy-report\""));
        assert!(rendered.contains("\"id\":\"runtime-public-deferred-policy-proof\""));
    }

    #[test]
    fn recording_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recording_continuity_boundary_text();
        assert!(rendered.contains(
            "recording_continuity_boundary: signal.runtime.recording-continuity-boundary"
        ));
        assert!(rendered.contains("acceptance_task: effigy acceptance:recording-continuity"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::recording_capture_snapshot and RuntimeSupervisorReport::observation.recording_capture_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeRecordingCaptureCommitReceipt::committed_checkpoint"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_recording_continuity_boundary_reports_resumable_restartable_and_terminal_states"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-recording-continuity-boundary --format=json"
        ));
    }

    #[test]
    fn recording_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recording_continuity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.recording-continuity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/013-recording-continuity-midi-capture-and-checkpoint-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:recording-continuity\""));
        assert!(rendered.contains("\"id\":\"runtime-recording-capture-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-recording-capture-commit-receipt\""));
        assert!(rendered.contains("\"id\":\"shared-host-recording-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-terminal-capture-proof\""));
    }

    #[test]
    fn offline_render_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_offline_render_continuity_boundary_text();
        assert!(rendered.contains(
            "offline_render_continuity_boundary: signal.runtime.offline-render-continuity-boundary"
        ));
        assert!(rendered.contains("acceptance_task: effigy acceptance:offline-render-continuity"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::offline_render_session_snapshot and RuntimeSupervisorReport::observation.offline_render_session_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeObservationApi::get_offline_render_session_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_offline_render_continuity_boundary_reports_resumable_restartable_and_terminal_states"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-offline-render-continuity-boundary --format=json"
        ));
    }

    #[test]
    fn offline_render_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_offline_render_continuity_boundary_json();
        assert!(
            rendered.contains("\"boundary\":\"signal.runtime.offline-render-continuity-boundary\"")
        );
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/015-offline-render-recovery-and-resumability-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:offline-render-continuity\""));
        assert!(rendered.contains("\"id\":\"runtime-offline-render-session-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-offline-render-observation-api\""));
        assert!(rendered.contains("\"id\":\"shared-host-offline-render-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-terminal-render-proof\""));
    }

    #[test]
    fn plugin_continuity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_plugin_continuity_boundary_text();
        assert!(rendered
            .contains("plugin_continuity_boundary: signal.runtime.plugin-continuity-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:plugin-continuity"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_lifecycle_snapshot and RuntimeSupervisorReport::observation.plugin_lifecycle_snapshot"
        ));
        assert!(rendered.contains("surface: RuntimeObservationApi::get_plugin_chain_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_plugin_continuity_boundary_reports_shared_boundary_and_policy_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-plugin-continuity-boundary --format=json"
        ));
    }

    #[test]
    fn plugin_continuity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_plugin_continuity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.plugin-continuity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/014-plugin-isolation-policy-transport-rebind-and-shared-sandbox-continuity-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:plugin-continuity\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-chain-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-plugin-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-placement-policy-proof\""));
    }

    #[test]
    fn vst3_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_vst3_boundary_text();
        assert!(rendered.contains("vst3_boundary: signal.runtime.vst3-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:vst3-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_vst3_boundary_reports_runtime_owned_discovery_and_lifecycle_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-vst3-boundary --format=json"
        ));
    }

    #[test]
    fn vst3_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_vst3_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.vst3-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/020-vst3-adapter-baseline-and-runtime-owned-lifecycle-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:vst3-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-vst3-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-vst3-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-vst3-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-vst3-proof\""));
    }

    #[test]
    fn au_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_au_boundary_text();
        assert!(rendered.contains("au_boundary: signal.runtime.au-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:au-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_au_boundary_reports_runtime_owned_discovery_and_lifecycle_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-au-boundary --format=json"
        ));
    }

    #[test]
    fn au_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_au_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.au-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/021-au-adapter-baseline-and-runtime-owned-lifecycle-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:au-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-au-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-au-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-au-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-au-proof\""));
    }

    #[test]
    fn cross_adapter_parity_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_cross_adapter_parity_boundary_text();
        assert!(rendered.contains(
            "cross_adapter_parity_boundary: signal.runtime.cross-adapter-parity-boundary"
        ));
        assert!(
            rendered.contains("acceptance_task: effigy acceptance:cross-adapter-parity-boundary")
        );
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_discovery_snapshot and RuntimeSupervisorReport::observation.plugin_discovery_snapshot"
        ));
        assert!(
            rendered.contains("surface: RuntimeObservationApi::get_plugin_lifecycle_snapshot()")
        );
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_cross_adapter_parity_boundary_reports_runtime_owned_portability_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-cross-adapter-parity-boundary --format=json"
        ));
    }

    #[test]
    fn cross_adapter_parity_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_cross_adapter_parity_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.cross-adapter-parity-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/022-backend-capability-parity-linux-plugin-support-and-cross-adapter-conformance-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:cross-adapter-parity-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-cross-adapter-discovery-report\""));
        assert!(rendered.contains("\"id\":\"runtime-cross-adapter-lifecycle-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-cross-adapter-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"server-host-cross-adapter-proof\""));
    }

    #[test]
    fn generic_event_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_generic_event_boundary_text();
        assert!(rendered.contains("generic_event_boundary: signal.runtime.generic-event-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:generic-event-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_event_snapshot and RuntimeSupervisorReport::observation.plugin_event_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationApi::get_plugin_discovery_snapshot() capability_coverage.supports_note_expression_count"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_generic_event_boundary_reports_runtime_owned_event_and_capability_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-generic-event-boundary --format=json"
        ));
    }

    #[test]
    fn generic_event_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_generic_event_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.generic-event-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/023-generic-midi-note-expression-and-plugin-event-model-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:generic-event-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-generic-event-report\""));
        assert!(rendered.contains("\"id\":\"runtime-generic-event-capability-coverage\""));
        assert!(rendered.contains("\"id\":\"shared-host-generic-event-supervisor-report\""));
    }

    #[test]
    fn recall_portability_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recall_portability_boundary_text();
        assert!(rendered
            .contains("recall_portability_boundary: signal.runtime.recall-portability-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:recall-portability-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::plugin_chain_snapshot and RuntimeSupervisorReport::observation.plugin_chain_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeObservationApi::get_plugin_recall_handoff_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime --test public_contract_boundary public_runtime_recall_interchange_and_ara_context_truth_is_consumable_from_reexports"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-recall-portability-boundary --format=json"
        ));
    }

    #[test]
    fn recall_portability_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_recall_portability_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.recall-portability-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/024-plugin-preset-state-interchange-portable-recall-and-ara-context-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:recall-portability-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-chain-recall-report\""));
        assert!(rendered.contains("\"id\":\"runtime-plugin-recall-handoff\""));
        assert!(rendered.contains("\"id\":\"shared-host-recall-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-recall-portability-public-proof\""));
    }

    #[test]
    fn device_supervision_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_device_supervision_boundary_text();
        assert!(rendered
            .contains("device_supervision_boundary: signal.runtime.device-supervision-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:device-supervision-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::device_supervision_snapshot and RuntimeSupervisorReport::observation.device_supervision_snapshot"
        ));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::fault_status and RuntimeObservationReport::interruption_summary"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_device_supervision_boundary_reports_recovering_and_faulted_runtime_states"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-device-supervision-boundary --format=json"
        ));
    }

    #[test]
    fn device_supervision_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_device_supervision_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.device-supervision-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/025-device-supervision-restart-state-machine-and-fault-boundary-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:device-supervision-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-device-supervision-report\""));
        assert!(rendered.contains("\"id\":\"runtime-supervision-fault-alignment\""));
        assert!(rendered.contains("\"id\":\"shared-host-device-supervision-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"runtime-device-supervision-public-proof\""));
    }

    #[test]
    fn clock_topology_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_clock_topology_boundary_text();
        assert!(
            rendered.contains("clock_topology_boundary: signal.runtime.clock-topology-boundary")
        );
        assert!(rendered.contains("acceptance_task: effigy acceptance:clock-topology-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeHostObservationReport::host_io and RuntimeHostSupervisorReport::observation.host_io"
        ));
        assert!(rendered.contains(
            "surface: LocalRuntimeHost::host_supervisor_report() -> RuntimeHostSupervisorReport"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_clock_topology_boundary_reports_drift_duplex_and_endpoint_receipts"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-clock-topology-boundary --format=json"
        ));
    }

    #[test]
    fn clock_topology_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_clock_topology_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.clock-topology-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/026-clock-domain-drift-duplex-mismatch-and-endpoint-topology-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:clock-topology-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-host-clocking-report\""));
        assert!(rendered.contains("\"id\":\"runtime-external-io-alignment\""));
        assert!(rendered.contains("\"id\":\"shared-local-host-clock-topology-report\""));
        assert!(rendered.contains("\"id\":\"runtime-clock-topology-public-proof\""));
    }

    #[test]
    fn external_io_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_io_boundary_text();
        assert!(rendered.contains("external_io_boundary: signal.runtime.external-io-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:external-io-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::external_io_snapshot and RuntimeSupervisorReport::observation.external_io_snapshot"
        ));
        assert!(rendered.contains(
            "surface: ServerRuntimeHost::supervisor_report() -> RuntimeSupervisorReport"
        ));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_external_io_boundary_reports_runtime_owned_monitor_and_loopback_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-external-io-boundary --format=json"
        ));
    }

    #[test]
    fn external_io_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_external_io_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.external-io-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/027-external-io-monitoring-tap-point-and-loopback-measurement-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:external-io-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-external-io-report\""));
        assert!(rendered.contains("\"id\":\"runtime-host-external-io-report\""));
        assert!(rendered.contains("\"id\":\"shared-local-host-external-io-report\""));
        assert!(rendered.contains("\"id\":\"shared-server-host-external-io-report\""));
        assert!(rendered.contains("\"id\":\"runtime-external-io-public-proof\""));
    }

    #[test]
    fn media_service_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_media_service_boundary_text();
        assert!(rendered.contains("media_service_boundary: signal.runtime.media-service-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:media-service-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::media_pipeline_snapshot, RuntimeObservationReport::media_service_snapshot, and RuntimeSupervisorReport::observation.{media_pipeline_snapshot,media_service_snapshot}"
        ));
        assert!(rendered.contains("surface: supervisor_report() -> RuntimeSupervisorReport"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_media_service_boundary_reports_runtime_owned_readiness_and_invalidation_truth"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-media-service-boundary --format=json"
        ));
    }

    #[test]
    fn media_service_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_media_service_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.media-service-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/028-media-indexing-waveform-analysis-and-preview-service-contract.md\""
        ));
        assert!(
            rendered.contains("\"acceptance_task\":\"effigy acceptance:media-service-boundary\"")
        );
        assert!(rendered.contains("\"id\":\"runtime-media-service-report\""));
        assert!(rendered.contains("\"id\":\"runtime-media-service-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-media-service-report\""));
        assert!(rendered.contains("\"id\":\"runtime-media-service-public-proof\""));
    }

    #[test]
    fn analysis_metadata_boundary_text_reports_runtime_and_host_edge_proofs() {
        let rendered = render_analysis_metadata_boundary_text();
        assert!(rendered
            .contains("analysis_metadata_boundary: signal.runtime.analysis-metadata-boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:analysis-metadata-boundary"));
        assert!(rendered.contains(
            "surface: RuntimeObservationReport::media_library_snapshot and RuntimeSupervisorReport::observation.media_library_snapshot"
        ));
        assert!(rendered
            .contains("surface: RuntimeObservationApi::get_media_library_service_snapshot()"));
        assert!(rendered.contains(
            "cargo test -p signal-runtime public_runtime_analysis_metadata_boundary_reports_runtime_owned_library_descriptors"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-analysis-metadata-boundary --format=json"
        ));
    }

    #[test]
    fn analysis_metadata_boundary_json_reports_runtime_and_host_edge_proofs() {
        let rendered = render_analysis_metadata_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.runtime.analysis-metadata-boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/029-analysis-metadata-extraction-and-library-service-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:analysis-metadata-boundary\""));
        assert!(rendered.contains("\"id\":\"runtime-analysis-metadata-report\""));
        assert!(rendered.contains("\"id\":\"runtime-analysis-metadata-snapshot\""));
        assert!(rendered.contains("\"id\":\"shared-host-analysis-metadata-report\""));
        assert!(rendered.contains("\"id\":\"runtime-analysis-metadata-public-proof\""));
    }

    #[test]
    fn integrated_acceptance_lane_text_reports_required_and_advisory_policy() {
        let rendered = render_integrated_acceptance_lane_text();
        assert!(rendered
            .contains("integrated_acceptance_lane: signal.runtime.integrated-acceptance-lane"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:integrated-acceptance-lane"));
        assert!(rendered.contains("- effigy acceptance:interruption-boundary"));
        assert!(rendered.contains("- effigy acceptance:analysis-metadata-boundary"));
        assert!(rendered.contains("- effigy acceptance:recording-continuity"));
        assert!(rendered.contains("- effigy acceptance:vst3-boundary"));
        assert!(rendered.contains("title: Adapter And Portability Breadth"));
        assert!(rendered.contains("id: cross-family-export-proof"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json"
        ));
    }

    #[test]
    fn integrated_acceptance_lane_json_reports_required_and_advisory_policy() {
        let rendered = render_integrated_acceptance_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.runtime.integrated-acceptance-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/030-fault-injection-harness-and-multi-backend-acceptance-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:integrated-acceptance-lane\""));
        assert!(rendered.contains("\"required_task_count\":11"));
        assert!(rendered.contains("\"advisory_task_count\":6"));
        assert!(rendered.contains("\"id\":\"recovery-and-fault-attribution\""));
        assert!(rendered.contains("\"id\":\"adapter-and-portability-breadth\""));
        assert!(rendered.contains("\"id\":\"cross-family-export-proof\""));
        assert!(rendered.contains(
            "\"command\":\"cargo test -p signal-supervisor-tools export_json_carries_cross_family_integrated_acceptance_evidence\""
        ));
        assert!(rendered.contains("\"command\":\"effigy acceptance:integrated-acceptance-lane\""));
    }

    #[test]
    fn g06_soak_lane_text_reports_required_and_deferred_policy() {
        let rendered = render_g06_soak_lane_text();
        assert!(rendered.contains("g06_soak_lane: signal.g06.long-session-soak-lane"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:g06-soak-lane"));
        assert!(rendered.contains("id: required-local-soak-export"));
        assert!(rendered.contains("status: required"));
        assert!(rendered.contains("id: deferred-server-soak-export"));
        assert!(rendered.contains("status: deferred"));
        assert!(rendered.contains("id: g06-soak-lane-task"));
    }

    #[test]
    fn g06_soak_lane_json_reports_required_and_deferred_policy() {
        let rendered = render_g06_soak_lane_json();
        assert!(rendered.contains("\"lane\":\"signal.g06.long-session-soak-lane\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:g06-soak-lane\""));
        assert!(rendered.contains("\"id\":\"required-local-soak-export\""));
        assert!(rendered.contains("\"status\":\"required\""));
        assert!(rendered.contains("\"id\":\"deferred-server-soak-export\""));
        assert!(rendered.contains("\"status\":\"deferred\""));
        assert!(rendered.contains("\"id\":\"g06-soak-lane-proof\""));
    }

    #[test]
    fn host_edge_boundary_text_reports_stable_and_unstable_edges() {
        let rendered = render_host_edge_boundary_text();
        assert!(rendered.contains("host_edge_boundary: signal.host.edge.boundary"));
        assert!(rendered.contains("acceptance_task: effigy acceptance:host-edge-consumer"));
        assert!(rendered.contains("surface: RuntimeSupervisorApi implemented by both hosts"));
        assert!(rendered.contains("surface: supervisor_report() -> RuntimeSupervisorReport"));
        assert!(rendered.contains("tier: consumer-facing-but-unstable"));
        assert!(rendered.contains("surface: boot_* fault, recovery, watchdog, and soak helpers"));
    }

    #[test]
    fn host_edge_boundary_json_reports_stable_and_unstable_edges() {
        let rendered = render_host_edge_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.host.edge.boundary\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/009-shared-host-convenience-api-and-consumer-edge-contract.md\""
        ));
        assert!(rendered.contains("\"acceptance_task\":\"effigy acceptance:host-edge-consumer\""));
        assert!(rendered.contains("\"id\":\"shared-runtime-supervisor-api\""));
        assert!(rendered.contains("\"id\":\"shared-supervisor-report\""));
        assert!(rendered.contains("\"id\":\"host-summary-dtos\""));
        assert!(rendered.contains("\"tier\":\"scenario-only\""));
    }

    #[test]
    fn release_boundary_text_reports_packaging_baseline() {
        let rendered = render_release_boundary_text();
        assert!(rendered.contains("release_boundary: signal.release.boundary"));
        assert!(rendered.contains("release_version: 0.1.0"));
        assert!(rendered.contains("version_source: workspace.package.version"));
        assert!(rendered.contains("changelog_path: CHANGELOG.md"));
        assert!(rendered.contains("conformance_task: effigy acceptance:conformance"));
        assert!(rendered
            .contains("cargo run -p signal-supervisor-tools -- --describe-export --format=json"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-packaging-manifest --format=json"
        ));
        assert!(rendered.contains(
            "publication packaging beyond the repo-owned manifest descriptor and receipt inventory"
        ));
    }

    #[test]
    fn release_boundary_json_reports_packaging_baseline() {
        let rendered = render_release_boundary_json();
        assert!(rendered.contains("\"boundary\":\"signal.release.boundary\""));
        assert!(rendered.contains("\"release_version\":\"0.1.0\""));
        assert!(rendered.contains("\"version_source\":\"workspace.package.version\""));
        assert!(rendered.contains("\"changelog_path\":\"CHANGELOG.md\""));
        assert!(rendered.contains("\"conformance_task\":\"effigy acceptance:conformance\""));
        assert!(rendered.contains("\"id\":\"workspace-changelog\""));
        assert!(rendered.contains("\"id\":\"consumer-conformance\""));
        assert!(rendered.contains("\"id\":\"supervisor-export-description\""));
        assert!(rendered.contains("\"id\":\"publication-packaging-manifest\""));
    }

    #[test]
    fn packaging_manifest_text_reports_release_bundle_and_receipts() {
        let rendered = render_packaging_manifest_text();
        assert!(rendered.contains("packaging_manifest: signal.release.packaging-manifest"));
        assert!(rendered.contains("release_version: 0.1.0"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md"
        ));
        assert!(rendered.contains("acceptance_task: effigy acceptance:release-packaging-consumer"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-host-edge-boundary --format=json"
        ));
        assert!(rendered.contains("id: manifest-generation-receipt"));
        assert!(rendered.contains("id: validation-receipt"));
        assert!(rendered.contains("crates.io publication and registry upload automation"));
    }

    #[test]
    fn packaging_manifest_json_reports_release_bundle_and_receipts() {
        let rendered = render_packaging_manifest_json();
        assert!(rendered.contains("\"manifest\":\"signal.release.packaging-manifest\""));
        assert!(rendered.contains("\"release_version\":\"0.1.0\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/010-publication-grade-packaging-manifest-and-release-receipt-contract.md\""
        ));
        assert!(rendered
            .contains("\"acceptance_task\":\"effigy acceptance:release-packaging-consumer\""));
        assert!(rendered.contains("\"id\":\"release-boundary-descriptor\""));
        assert!(rendered.contains("\"id\":\"manifest-generation-receipt\""));
        assert!(rendered.contains("\"id\":\"validation-receipt\""));
        assert!(rendered.contains("\"id\":\"release-boundary-baseline\""));
    }

    #[test]
    fn downstream_automation_text_reports_mandatory_and_optional_fixtures() {
        let rendered = render_downstream_automation_text();
        assert!(rendered.contains("downstream_automation_boundary: signal.downstream.automation"));
        assert!(rendered.contains("mandatory_release_task: effigy acceptance:downstream-release"));
        assert!(rendered.contains("optional_depth_task: effigy acceptance:downstream-depth"));
        assert!(rendered.contains("id: release-packaging-consumer"));
        assert!(rendered.contains("id: local-mixed-watchdog-export"));
        assert!(rendered.contains(
            "signal.supervisor.export JSON with profiling_receipt, soak_receipt, and supervisor_report"
        ));
    }

    #[test]
    fn downstream_automation_json_reports_mandatory_and_optional_fixtures() {
        let rendered = render_downstream_automation_json();
        assert!(rendered.contains("\"boundary\":\"signal.downstream.automation\""));
        assert!(rendered
            .contains("\"mandatory_release_task\":\"effigy acceptance:downstream-release\""));
        assert!(rendered.contains("\"optional_depth_task\":\"effigy acceptance:downstream-depth\""));
        assert!(rendered.contains("\"combined_task\":\"effigy acceptance:downstream-automation\""));
        assert!(rendered.contains("\"id\":\"downstream-automation-descriptor\""));
        assert!(rendered.contains("\"id\":\"local-soak-export\""));
        assert!(rendered.contains("\"id\":\"analysis-acceptance\""));
    }

    #[test]
    fn downstream_fail_gates_text_reports_required_and_deferred_policy() {
        let rendered = render_downstream_fail_gates_text();
        assert!(rendered.contains("downstream_fail_gates: signal.downstream.fail-gates"));
        assert!(rendered.contains("fail_gate_task: effigy acceptance:downstream-gate"));
        assert!(rendered.contains("id: mandatory-release-gate"));
        assert!(rendered.contains("blocks_release: true"));
        assert!(rendered.contains("id: optional-depth-lane"));
        assert!(rendered.contains("blocks_release: false"));
        assert!(rendered.contains("id: server-soak-export"));
    }

    #[test]
    fn downstream_fail_gates_json_reports_required_and_deferred_policy() {
        let rendered = render_downstream_fail_gates_json();
        assert!(rendered.contains("\"boundary\":\"signal.downstream.fail-gates\""));
        assert!(rendered.contains("\"fail_gate_task\":\"effigy acceptance:downstream-gate\""));
        assert!(rendered.contains("\"id\":\"mandatory-release-gate\""));
        assert!(rendered.contains("\"blocks_release\":true"));
        assert!(rendered.contains("\"id\":\"optional-depth-lane\""));
        assert!(rendered.contains("\"blocks_release\":false"));
        assert!(rendered.contains("\"id\":\"server-soak-export\""));
        assert!(rendered.contains("\"status\":\"deferred\""));
    }

    #[test]
    fn generation_closeout_text_reports_combined_boundary_and_next_queue() {
        let rendered = render_generation_closeout_text();
        assert!(rendered.contains("generation_closeout: signal.generation.closeout"));
        assert!(rendered.contains("generation: g06"));
        assert!(rendered.contains(
            "contract_path: docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md"
        ));
        assert!(rendered.contains("closeout_task: effigy acceptance:g06-closeout"));
        assert!(rendered.contains("promotion_decision: promote-g07"));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json"
        ));
        assert!(rendered.contains(
            "cargo run -p signal-supervisor-tools -- --describe-g06-soak-lane --format=json"
        ));
        assert!(rendered.contains("next_generation_path: docs/roadmaps/g07/README.md"));
        assert!(rendered.contains("next_generation_status: active"));
        assert!(rendered.contains("next_queue_status: promoted-g07-active"));
        assert!(rendered.contains("id: runtime-hardening-and-recovery"));
        assert!(rendered.contains("status: sufficient-for-promotion"));
        assert!(rendered.contains("g06 now closes cleanly enough to promote g07"));
    }

    #[test]
    fn generation_closeout_json_reports_combined_boundary_and_next_queue() {
        let rendered = render_generation_closeout_json();
        assert!(rendered.contains("\"closeout\":\"signal.generation.closeout\""));
        assert!(rendered.contains("\"generation\":\"g06\""));
        assert!(rendered.contains(
            "\"contract_path\":\"docs/contracts/031-long-session-soak-promotion-gate-and-loophole-readiness-contract.md\""
        ));
        assert!(rendered.contains("\"closeout_task\":\"effigy acceptance:g06-closeout\""));
        assert!(rendered.contains("\"promotion_decision\":\"promote-g07\""));
        assert!(rendered.contains(
            "\"integrated_acceptance_command\":\"cargo run -p signal-supervisor-tools -- --describe-integrated-acceptance-lane --format=json\""
        ));
        assert!(rendered.contains(
            "\"g06_soak_lane_command\":\"cargo run -p signal-supervisor-tools -- --describe-g06-soak-lane --format=json\""
        ));
        assert!(rendered.contains("\"next_generation_path\":\"docs/roadmaps/g07/README.md\""));
        assert!(rendered.contains("\"next_generation_status\":\"active\""));
        assert!(rendered.contains("\"next_queue_status\":\"promoted-g07-active\""));
        assert!(rendered.contains("\"id\":\"integrated-acceptance-base\""));
        assert!(rendered.contains("\"id\":\"bounded-soak-lane\""));
        assert!(rendered.contains("\"id\":\"generation-closeout-description\""));
        assert!(rendered.contains("\"id\":\"runtime-hardening-and-recovery\""));
        assert!(rendered.contains("\"status\":\"sufficient-for-promotion\""));
        assert!(rendered.contains("\"the g06 closeout verdict is sufficient to promote g07"));
    }

    #[test]
    fn export_json_is_versioned() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"schema\":\"signal.supervisor.export\""));
        assert!(export.contains("\"schema_version\":1"));
        assert!(export.contains("\"profiling_receipt\":{"));
        assert!(export.contains("\"soak_receipt\":{"));
    }

    #[test]
    fn export_json_carries_last_deferred_service_receipt() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("enable safe mode");
        let purge_receipt = runtime
            .purge_offline_render_artifacts(RuntimeOfflineRenderPurgeRequest {
                request_id: "purge:export-proof".into(),
                artifact_root_path: Some("/tmp/nonexistent-artifacts".into()),
                report_path: Some("/tmp/nonexistent-report.json".into()),
            })
            .expect("safe mode should defer purge export proof");
        assert!(!purge_receipt.purged_report);
        assert!(!purge_receipt.purged_artifact_root);

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );

        assert!(export.contains("\"last_deferred_service\":{"));
        assert!(export.contains("\"work_class\":\"OfflineRenderPurge\""));
        assert!(export.contains("\"decision\":\"Defer\""));
        assert!(export.contains("\"reason\":\"SafeMode\""));
    }

    #[test]
    fn export_json_carries_runtime_owned_plugin_discovery_catalog() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins/CLAP".into()],
            formats: vec![PluginFormat::Clap],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_discovered_type_record(),
                sample_backend_breadth_record(),
            ],
        );
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "export-consumer-sandbox".into(),
            plugin_format: PluginFormat::Clap,
            plugin_type_id: None,
        });
        runtime.record_plugin_sandbox_lifecycle(
            "export-consumer-sandbox",
            PluginSandboxLifecycleStage::SandboxEnsured,
            None,
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );

        assert!(export.contains("\"host_summary\":{}"));
        assert!(export.contains("\"supervisor_report\":{"));
        assert!(export.contains("\"plugin_discovery_snapshot\":{"));
        assert!(export.contains("\"discovered_type_count\":2"));
        assert!(export.contains("\"discovered_format_count\":2"));
        assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
        assert!(export.contains("\"plugin_type_id\":\"plugin:vst3:export-instrument\""));
        assert!(export.contains("\"format\":\"Clap\""));
        assert!(export.contains("\"multi_format_catalog\":true"));
        assert!(export.contains("\"requires_main_thread_for_state_count\":1"));
        assert!(export.contains("\"format_coverage\":["));
        assert!(export.contains("\"supports_snapshot\":true"));
        assert!(export.contains("\"supports_activate\":true"));
    }

    #[test]
    fn export_json_carries_runtime_owned_plugin_discovery_capability_coverage() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec!["~/Library/Audio/Plug-Ins".into()],
            formats: vec![PluginFormat::Clap, PluginFormat::Vst3],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_discovered_type_record(),
                sample_backend_breadth_record(),
            ],
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &Default::default());
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Default,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"discovered_format_count\":2"));
        assert!(export.contains("\"multi_format_catalog\":true"));
        assert!(export.contains("\"requires_main_thread_for_state_count\":1"));
        assert!(export.contains("\"max_parameter_count\":24"));
        assert!(export.contains("\"format\":\"Vst3\""));
        assert!(export.contains("\"instrument_count\":1"));
    }

    #[test]
    fn export_json_carries_runtime_recovery_sequence() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::RecoveryCycle {
                sandbox_id: "sandbox-1".into(),
                intent: RecoveryRestartIntent::WatchdogRecovery,
                stop_reason: StopReason::DegradedModeRecovery,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxLifecycle {
                sandbox_id: "sandbox-1".into(),
                stage: PluginSandboxLifecycleStage::TransportAttached,
                processing_epoch: Some(4),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-1".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(4),
                block_sequence: Some(9),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::LeaseRollover {
                sandbox_id: "sandbox-1".into(),
                previous_lease_id: "lease-3".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                first_block_sequence: 9,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerInvalidation {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: Some(9),
                stage: BrokerInvalidationStage::CompletionRegionInvalidated,
                reason: "watchdog recovery teardown".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                processing_epoch: 4,
                block_sequence: 9,
                stage: CompletionSlotStage::FallbackApplied,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-1".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                block_sequence: Some(9),
                stage: BrokerFailureStage::PayloadRead,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::Detached,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-1".into(),
                lease_id: "lease-4".into(),
                region_id: "region-4".into(),
                stage: PluginSandboxTransportStage::DetachFault,
                processing_epoch: Some(4),
                detail: Some("broker detach fault: stale region mapping".into()),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::SandboxOperationFailure {
                sandbox_id: "sandbox-1".into(),
                lease_id: Some("lease-4".into()),
                processing_epoch: Some(4),
                operation: "processBlock".into(),
                error_kind: "resourceUnavailable".into(),
                stage: SandboxOperationFailureStage::ProcessAttach,
                detail: "failed to attach shared-memory region: stale mapping".into(),
            },
        );
        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Soak,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"recovery_events\":1"));
        assert!(export.contains("\"recovery_sequence\":[{"));
        assert!(export.contains("\"intent\":\"WatchdogRecovery\""));
        assert!(export.contains("\"last_recovery_intent\":\"WatchdogRecovery\""));
        assert!(export.contains("\"lifecycle_events\":1"));
        assert!(export.contains("\"lifecycle_sequence\":[{"));
        assert!(export.contains("\"stage\":\"TransportAttached\""));
        assert!(export.contains("\"transport_events\":4"));
        assert!(export.contains("\"transport_sequence\":[{"));
        assert!(export.contains("\"region_id\":\"region-4\""));
        assert!(export.contains("\"heartbeat_events\":1"));
        assert!(export.contains("\"heartbeat_sequence\":[{"));
        assert!(export.contains("\"block_sequence\":9"));
        assert!(export.contains("\"block_dispatch_events\":1"));
        assert!(export.contains("\"block_dispatch_sequence\":[{"));
        assert!(export.contains("\"completion_state\":\"Completed\""));
        assert!(export.contains("\"lease_rollover_events\":1"));
        assert!(export.contains("\"lease_rollover_sequence\":[{"));
        assert!(export.contains("\"previous_lease_id\":\"lease-3\""));
        assert!(export.contains("\"invalidation_events\":1"));
        assert!(export.contains("\"invalidation_sequence\":[{"));
        assert!(export.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(export.contains("\"completion_slot_events\":2"));
        assert!(export.contains("\"completion_slot_sequence\":[{"));
        assert!(export.contains("\"stage\":\"FallbackApplied\""));
        assert!(export.contains("\"transport_fault_events\":8"));
        assert!(export.contains("\"last_transport_fault\":{"));
        assert!(export.contains("\"transport_fault_sequence\":[{"));
        assert!(export.contains("\"source\":\"HostBroker\""));
        assert!(export.contains("\"source\":\"SandboxOperation\""));
        assert!(export.contains("\"source\":\"RuntimeDispatch\""));
        assert!(export.contains("\"phase\":\"Dispatch\""));
        assert!(export.contains("\"phase\":\"Teardown\""));
        assert!(export.contains("\"resource\":\"SharedMemoryPayload\""));
        assert!(export.contains("\"resource\":\"SharedMemoryLease\""));
        assert!(export.contains("\"resource\":\"CompletionSlot\""));
        assert!(export.contains("\"operation\":\"block_payload.read\""));
        assert!(export.contains("\"operation\":\"transport.detach_request\""));
        assert!(export.contains("\"operation\":\"transport.detached\""));
        assert!(export.contains("\"operation\":\"transport.detach_fault\""));
        assert!(export.contains("\"operation\":\"completion_region.invalidate\""));
        assert!(export.contains("\"operation\":\"completion_slot.timeout\""));
        assert!(export.contains("\"operation\":\"completion_slot.fallback_apply\""));
        assert!(export.contains("\"operation\":\"processBlock\""));
        assert!(export.contains("\"stage\":\"TransportDetachRequested\""));
        assert!(export.contains("\"stage\":\"TransportDetached\""));
        assert!(export.contains("\"stage\":\"DetachFault\""));
        assert!(export.contains("\"stage\":\"CompletionRegionInvalidated\""));
        assert!(export.contains("\"stage\":\"CompletionSlotTimedOut\""));
        assert!(export.contains("\"stage\":\"FallbackApplied\""));
        assert!(export.contains("\"transport_fault_summary\":{"));
        assert!(export.contains("\"boundary_mode\":\"FaultAdjacentOnly\""));
        assert!(export.contains("\"host_broker_events\":4"));
        assert!(export.contains("\"sandbox_operation_events\":1"));
        assert!(export.contains("\"runtime_dispatch_events\":3"));
        assert!(export.contains("\"dispatch_events\":"));
        assert!(export.contains("\"teardown_events\":"));
        assert!(export.contains("\"transport_concurrency_snapshot\":{"));
        assert!(export.contains("\"steady_session_limit\":1"));
        assert!(export.contains("\"recovery_session_limit\":2"));
        assert!(export.contains("\"current_attached_sessions\":0"));
        assert!(export.contains("\"current_lingering_sessions\":0"));
        assert!(export.contains("\"peak_lingering_sessions\":0"));
        assert!(export.contains("\"current_detach_requested_sessions\":0"));
        assert!(export.contains("\"current_detach_faulted_sessions\":0"));
        assert!(export.contains("\"transport_session_summary\":{"));
        assert!(export.contains("\"boundary_mode\":\"HealthyPathVisible\""));
        assert!(export.contains("\"current_state\":\"DetachFaulted\""));
        assert!(export.contains("\"currently_attached\":false"));
        assert!(export.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(export.contains("\"dispatch_state\":\"Completed\""));
        assert!(export.contains("\"current_attached_session_count\":0"));
        assert!(export.contains("\"max_concurrent_attached_sessions\":1"));
        assert!(export.contains("\"attach_events\":1"));
        assert!(export.contains("\"detach_requested_events\":1"));
        assert!(export.contains("\"detached_events\":1"));
        assert!(export.contains("\"detach_fault_events\":1"));
        assert!(export.contains("\"heartbeat_responded_events\":1"));
        assert!(export.contains("\"dispatch_completed_events\":1"));
        assert!(export.contains("\"active_sandbox_id\":null"));
        assert!(export.contains("\"active_lease_id\":null"));
        assert!(export.contains("\"active_region_id\":null"));
        assert!(export.contains("\"active_block_sequence\":"));
        assert!(export.contains("\"active_sessions\":[]"));
        assert!(export.contains("\"last_region_id\":\"region-4\""));
        assert!(export.contains("\"broker_failure_events\":1"));
        assert!(export.contains("\"broker_failure_sequence\":[{"));
        assert!(export.contains("\"stage\":\"PayloadRead\""));
        assert!(export.contains("\"sandbox_operation_failure_events\":1"));
        assert!(export.contains("\"sandbox_operation_failure_sequence\":[{"));
        assert!(export.contains("\"stage\":\"ProcessAttach\""));
    }

    #[test]
    fn export_json_carries_cross_family_integrated_acceptance_evidence() {
        let mut runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        runtime
            .handshake(HandshakeRequest {
                client_version: "integrated-acceptance-export".into(),
                anticipative_preferred: true,
                max_sample_rate_hint: Some(96_000),
            })
            .expect("integrated acceptance export handshake should succeed");
        runtime
            .configure(RuntimeConfigRequest::new(48_000, 512))
            .expect("integrated acceptance export configure should succeed");
        runtime
            .start()
            .expect("integrated acceptance export start should succeed");

        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "integrated-acceptance-sandbox".into(),
            trigger: RuntimeWatchdogTrigger::HeartbeatMisses,
            processing_epoch: 1,
        });
        runtime.record_watchdog_restart(WatchdogRestartRecord {
            sandbox_id: "integrated-acceptance-sandbox".into(),
            trigger: RuntimeWatchdogTrigger::DeadlineMisses,
            processing_epoch: 2,
        });

        runtime.record_plugin_format_platform_coverage(vec![
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Clap,
                supported_platforms: vec![
                    RuntimePluginHostPlatform::MacOs,
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                unsupported_platforms: Vec::new(),
                summary: "platforms=MacOs/Linux/Windows unsupported=none".into(),
            },
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Vst3,
                supported_platforms: vec![
                    RuntimePluginHostPlatform::MacOs,
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                unsupported_platforms: Vec::new(),
                summary: "platforms=MacOs/Linux/Windows unsupported=none".into(),
            },
            RuntimePluginFormatPlatformCoverageRecord {
                format: PluginFormat::Au,
                supported_platforms: vec![RuntimePluginHostPlatform::MacOs],
                unsupported_platforms: vec![
                    RuntimePluginHostPlatform::Linux,
                    RuntimePluginHostPlatform::Windows,
                ],
                summary: "platforms=MacOs unsupported=Linux/Windows".into(),
            },
        ]);
        let scan_handle = runtime.record_plugin_scan_request(&PluginScanRequest {
            roots: vec![
                "~/.clap".into(),
                "~/.vst3".into(),
                "~/Library/Audio/Plug-Ins/Components".into(),
            ],
            formats: vec![PluginFormat::Clap, PluginFormat::Vst3, PluginFormat::Au],
        });
        runtime.record_plugin_scan_results(
            scan_handle,
            vec![
                sample_discovered_type_record(),
                sample_backend_breadth_record(),
                sample_au_breadth_record(),
            ],
        );
        runtime.record_plugin_sandbox_spec(&PluginSandboxSpec {
            sandbox_id: "integrated-acceptance-vst3".into(),
            plugin_format: PluginFormat::Vst3,
            plugin_type_id: Some("plugin:vst3:export-instrument".into()),
        });
        runtime.record_plugin_sandbox_lifecycle(
            "integrated-acceptance-vst3",
            PluginSandboxLifecycleStage::InstancePrepared,
            Some(2),
        );

        runtime
            .set_safe_mode(SafeModeRequest { enabled: true })
            .expect("integrated acceptance export safe mode should enable");
        let deferred = runtime
            .render_offline_queue(vec![RuntimeOfflineRenderRequest {
                request_id: "render:integrated-acceptance".into(),
                timeline_start_samples: 0,
                duration_samples: 64,
                export_sample_rate_hz: 48_000,
                include_main_mix: true,
                artifact_root_path: None,
                stem_targets: Vec::new(),
                freeze_artifacts: Vec::new(),
            }])
            .expect("integrated acceptance export queue should defer in safe mode");
        assert_eq!(deferred.orchestration.deferred_work_item_count, 1);

        let ready_path = integrated_acceptance_media_fixture_path("ready");
        let missing_path = integrated_acceptance_media_fixture_path("missing");
        write_integrated_acceptance_test_wav(&ready_path);
        runtime
            .reconcile_media_assets(vec![
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:integrated-ready".into(),
                    content_hash: "integrated-ready".into(),
                    source_path: ready_path.display().to_string(),
                    file_name: "integrated-ready.wav".into(),
                    byte_size: fs::metadata(&ready_path)
                        .expect("integrated acceptance media fixture should exist")
                        .len(),
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 16,
                },
                RuntimeMediaAssetRegistration {
                    asset_id: "asset:sha256:integrated-missing".into(),
                    content_hash: "integrated-missing".into(),
                    source_path: missing_path.display().to_string(),
                    file_name: "integrated-missing.wav".into(),
                    byte_size: 0,
                    sample_rate_hz: 48_000,
                    channel_count: 1,
                    duration_samples: 128,
                    waveform_bin_count: 16,
                },
            ])
            .expect("integrated acceptance media assets should reconcile");
        runtime
            .start_media_preview("asset:sha256:integrated-ready")
            .expect("integrated acceptance media preview should start");

        let recorder = RuntimeEventRecorder::default();
        let mut report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        report.observation = report
            .observation
            .clone()
            .with_host_external_io(&sample_integrated_acceptance_host_io());
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &report.profiling_receipt(),
            &report.soak_receipt(),
            &report,
        );

        assert!(export.contains("\"fault_status\":{"));
        assert!(export.contains("\"primary_fault_cause\":\"WatchdogRestart\""));
        assert!(export.contains("\"interruption_summary\":{"));
        assert!(export.contains("\"watchdog_restart_count\":2"));
        assert!(export.contains("\"fault_diagnostic_receipt\":{"));
        assert!(export.contains("\"primary_family\":\"DeferredWorkPressure\""));
        assert!(export.contains("\"last_deferred_service\":{"));
        assert!(export.contains("\"decision\":\"Defer\""));
        assert!(export.contains("\"plugin_discovery_snapshot\":{"));
        assert!(export.contains("\"plugin_type_id\":\"plugin:clap:export-consumer\""));
        assert!(export.contains("\"plugin_type_id\":\"plugin:vst3:export-instrument\""));
        assert!(export.contains("\"plugin_type_id\":\"plugin:au:export-au\""));
        assert!(export.contains("\"parity_coverage\":[{"));
        assert!(export.contains("\"supported_platforms\":[\"MacOs\"]"));
        assert!(export.contains("\"unsupported_platforms\":[\"Linux\",\"Windows\"]"));
        assert!(export.contains("\"device_supervision_snapshot\":{"));
        assert!(export.contains("\"external_io_snapshot\":{"));
        assert!(export.contains("\"monitoring_state\":\"Guarded\""));
        assert!(export.contains("\"drift_state\":\"CrossClockManaged\""));
        assert!(export.contains("\"duplex_mismatch_state\":\"CrossClockDiverged\""));
        assert!(export.contains("\"endpoint_topology\":\"Duplex\""));
        assert!(export.contains("\"media_pipeline_snapshot\":{"));
        assert!(export.contains("\"media_service_snapshot\":{"));
        assert!(export.contains("\"preview_state\":\"Previewing\""));
        assert!(export.contains("\"invalidated_asset_count\":1"));
        assert!(export.contains("\"media_library_snapshot\":{"));
        assert!(export.contains("\"ready_descriptor_count\":1"));
        assert!(export.contains("\"loudness_ready_descriptor_count\":1"));
        assert!(export.contains("\"character_ready_descriptor_count\":1"));

        let _ = fs::remove_file(&ready_path);
        if let Some(path) = runtime
            .get_media_pipeline_snapshot()
            .assets
            .iter()
            .find(|asset| asset.asset_id == "asset:sha256:integrated-ready")
            .and_then(|asset| asset.cache_path.as_deref())
        {
            let _ = fs::remove_file(path);
        }
    }

    #[test]
    fn export_json_serializes_per_session_transport_liveness() {
        let runtime = SignalRuntime::new(RuntimeConfig::local(48_000, 512));
        let mut recorder = RuntimeEventRecorder::default();
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(2),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                region_id: "region-b".into(),
                stage: PluginSandboxTransportStage::Attached,
                processing_epoch: Some(3),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::PluginSandboxTransport {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                region_id: "region-a".into(),
                stage: PluginSandboxTransportStage::DetachRequested,
                processing_epoch: Some(4),
                detail: None,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-a".into(),
                stage: HeartbeatCycleStage::Missed,
                processing_epoch: Some(4),
                block_sequence: Some(11),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::HeartbeatCycle {
                sandbox_id: "sandbox-b".into(),
                stage: HeartbeatCycleStage::Responded,
                processing_epoch: Some(5),
                block_sequence: Some(12),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                frame_count: 512,
                stage: BlockDispatchStage::TimedOut,
                completion_state: Some(CompletionState::TimedOut),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BlockDispatch {
                sandbox_id: "sandbox-b".into(),
                lease_id: "lease-b".into(),
                processing_epoch: 5,
                block_sequence: 12,
                frame_count: 512,
                stage: BlockDispatchStage::Completed,
                completion_state: Some(CompletionState::Completed),
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::CompletionSlotTransition {
                sandbox_id: "sandbox-a".into(),
                lease_id: "lease-a".into(),
                processing_epoch: 4,
                block_sequence: 11,
                stage: CompletionSlotStage::TimedOut,
            },
        );
        RuntimeEventSink::push(
            &mut recorder,
            RuntimeEvent::BrokerFailure {
                sandbox_id: "sandbox-b".into(),
                lease_id: Some("lease-b".into()),
                processing_epoch: Some(5),
                block_sequence: Some(12),
                stage: BrokerFailureStage::PayloadRead,
                detail: "stale shared-memory mapping".into(),
            },
        );

        let report = RuntimeSupervisorReport::capture(&runtime, &recorder);
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].state,
            TransportSessionState::DetachRequested
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].heartbeat_freshness,
            TransportHeartbeatFreshness::Missed
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[0].dispatch_state,
            TransportDispatchState::TimedOut
        );
        assert_eq!(
            report.observation.transport_session_summary.active_sessions[1].heartbeat_freshness,
            TransportHeartbeatFreshness::Fresh
        );
        assert!(
            report.observation.transport_session_summary.active_sessions[0].transport_fault_count
                >= 1
        );
        assert!(
            report.observation.transport_session_summary.active_sessions[1].transport_fault_count
                >= 1
        );

        let profiling = report.profiling_receipt();
        let soak = report.soak_receipt();
        let export = render_supervisor_export_json(
            HostProfile::Local,
            Scenario::Mixed,
            "{}".into(),
            &profiling,
            &soak,
            &report,
        );
        assert!(export.contains("\"active_sessions\":[{"));
        assert!(export.contains("\"sandbox_id\":\"sandbox-a\""));
        assert!(export.contains("\"state\":\"DetachRequested\""));
        assert!(export.contains("\"currently_attached\":true"));
        assert!(export.contains("\"heartbeat_freshness\":\"Missed\""));
        assert!(export.contains("\"dispatch_state\":\"TimedOut\""));
        assert!(export.contains("\"peak_attached_sessions\":"));
        assert!(export.contains("\"active_block_sequence\":11"));
        assert!(export.contains("\"transport_fault_count\":1"));
        assert!(export.contains("\"last_transport_fault_source\":\"RuntimeDispatch\""));
        assert!(export.contains("\"last_transport_fault_stage\":\"CompletionSlotTimedOut\""));
        assert!(export.contains("\"last_transport_fault_phase\":\"Dispatch\""));
        assert!(export.contains("\"last_transport_fault_processing_epoch\":4"));
        assert!(export.contains("\"last_transport_fault_block_sequence\":11"));
        assert!(export.contains("\"sandbox_id\":\"sandbox-b\""));
        assert!(export.contains("\"heartbeat_freshness\":\"Fresh\""));
        assert!(export.contains("\"dispatch_state\":\"Completed\""));
        assert!(export.contains("\"active_block_sequence\":12"));
        assert!(export.contains("\"last_transport_fault_source\":\"HostBroker\""));
        assert!(export.contains("\"last_transport_fault_stage\":\"PayloadRead\""));
        assert!(export.contains("\"last_transport_fault_processing_epoch\":5"));
        assert!(export.contains("\"last_transport_fault_block_sequence\":12"));
    }

    #[test]
    fn local_summary_json_excludes_payload_by_default() {
        let summary = sample_local_summary();
        let rendered =
            super::render_local_summary_json(&summary, ExportDebugOptions { payload: false });
        assert!(!rendered.contains("\"payload\":{"));
        assert!(rendered.contains("\"sections\":[\"execution\",\"transport\",\"faults\"]"));
        assert!(rendered.contains("\"debug_sections_supported\":[\"payload\"]"));
        assert!(rendered.contains("\"debug_sections_enabled\":[]"));
        assert!(rendered.contains("\"last_recovery_intent\":\"WatchdogRecovery\""));
        assert!(rendered.contains("\"last_stop_reason\":\"DegradedModeRecovery\""));
    }

    #[test]
    fn local_summary_json_includes_payload_when_requested() {
        let summary = sample_local_summary();
        let rendered =
            super::render_local_summary_json(&summary, ExportDebugOptions { payload: true });
        assert!(rendered.contains("\"payload\""));
        assert!(rendered.contains("\"generated_event_bytes\""));
        assert!(
            rendered.contains("\"sections\":[\"execution\",\"transport\",\"faults\",\"payload\"]")
        );
        assert!(rendered.contains("\"debug_sections_supported\":[\"payload\"]"));
        assert!(rendered.contains("\"debug_sections_enabled\":[\"payload\"]"));
    }

    #[test]
    fn local_summary_text_reports_section_list() {
        let summary = sample_local_summary();
        let default_rendered =
            super::render_local_summary(&summary, ExportDebugOptions { payload: false });
        let payload_rendered =
            super::render_local_summary(&summary, ExportDebugOptions { payload: true });
        assert!(default_rendered.contains("sections: execution,transport,faults"));
        assert!(default_rendered.contains("debug_sections_supported: payload"));
        assert!(default_rendered.contains("debug_sections_enabled: none"));
        assert!(default_rendered.contains("last_recovery_intent=Some(WatchdogRecovery)"));
        assert!(default_rendered.contains("last_stop_reason=Some(DegradedModeRecovery)"));
        assert!(payload_rendered.contains("sections: execution,transport,faults,payload"));
        assert!(payload_rendered.contains("debug_sections_supported: payload"));
        assert!(payload_rendered.contains("debug_sections_enabled: payload"));
    }
}
