//! Embeddable runtime orchestration and typed runtime-host interfaces for
//! Signal.
//!
//! The crate owns the control-plane and observation-plane around executable
//! graph processing: graph projection, transport and parameter application,
//! anticipative prework policy, and supervisor-facing runtime snapshots.

mod interfaces;
mod runtime;

pub use interfaces::{
    ActiveTransportConcurrencySession, BackendPolicyOverride, BlockDispatchRecord,
    BlockDispatchStage, BrokerFailureRecord, BrokerFailureStage, BrokerInvalidationRecord,
    BrokerInvalidationStage, CompletionSlotRecord, CompletionSlotStage, DegradedReason,
    EffectiveRuntimeConfig, GraphContractProjection, GraphNodeBufferContractProjection,
    GraphNodeBusEndpointProjection, GraphNodeContractProjection, GraphNodeProjection,
    GraphNodeTopologyProjection, GraphProjection, HandshakeRequest, HandshakeResponse,
    HeartbeatCycleRecord, HeartbeatCycleStage, LeaseRolloverRecord, LingeringCleanupMode,
    LingeringCleanupPlan, LingeringCleanupQueueReceipt, LingeringCleanupTrigger, LoopRegion,
    ParameterBatch, ParameterEvent, PluginBackedNodeBinding, PluginBackedNodeBindingProjection,
    PluginFaultKind, PluginFaultRecord, PluginNodeRender, PluginNodeRenderBatch,
    PluginSandboxInstanceFaultRecord, PluginSandboxInstanceStateRecord,
    PluginSandboxLifecycleRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportRecord, PluginSandboxTransportStage, PluginScanRequest,
    ProjectionReceipt, RecoveryRecord, RecoveryRestartIntent, RestartRequest,
    RuntimeAutomationSnapshot, RuntimeConfigRequest, RuntimeDiagnosticsSnapshot,
    RuntimeEngineBlockResult, RuntimeEngineBlockSnapshot, RuntimeError, RuntimeErrorKind,
    RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink, RuntimeExecutionPhase,
    RuntimeExecutionTopologySummary, RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState,
    RuntimeHostAudioTransferPolicy, RuntimeHostClockSource, RuntimeHostClockingSummary,
    RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
    RuntimeHostLifecycleOwnership, RuntimeHostObservationReport, RuntimeHostRestartPolicy,
    RuntimeHostSupervisorReport, RuntimeLifecycleApi, RuntimeMediaAssetRegistration,
    RuntimeMediaAssetSnapshot, RuntimeMediaAssetState, RuntimeMediaPipelineSnapshot,
    RuntimeObservationApi, RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimePluginDispatchState, RuntimePreworkBacklogClass, RuntimePreworkCacheState,
    RuntimePreworkForecastMode, RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
    RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
    RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason, RuntimePreworkRetirementReason,
    RuntimePreworkServicePressure, RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
    RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness,
    RuntimeRecordingCaptureCommitReceipt, RuntimeRecordingCaptureSnapshot,
    RuntimeRecordingCaptureStartRequest, RuntimeRecordingCaptureState, RuntimeSchedulerSnapshot,
    RuntimeSchedulerState, RuntimeSupervisionSnapshot, RuntimeSupervisorApi,
    RuntimeSupervisorReport, RuntimeTimelineSnapshot, RuntimeTransportConcurrencySnapshot,
    RuntimeTransportObservationSnapshot, RuntimeTransportTransitionKind, RuntimeWarpClipRegistration,
    RuntimeWarpClipSnapshot, RuntimeWarpMode, RuntimeWarpPipelineSnapshot,
    RuntimeWarpReadiness, RuntimeWatchdogTrigger, SafeModeRequest, SandboxHandle,
    SandboxOperationFailureRecord, SandboxOperationFailureStage, ScanHandle,
    ScheduleProjection, StopReason, SubscriptionHandle, TransportAttachIntent,
    TransportDispatchState, TransportFaultBoundaryMode, TransportFaultPhase, TransportFaultRecord,
    TransportFaultResource, TransportFaultSource, TransportFaultStage, TransportFaultSummary,
    TransportHeartbeatFreshness, TransportProjection, TransportSessionBoundaryMode,
    TransportSessionProvenance, TransportSessionState, TransportSessionSummary,
    WatchdogRestartRecord,
};
pub use runtime::{RuntimeConfig, RuntimeProfile, SignalRuntime};
