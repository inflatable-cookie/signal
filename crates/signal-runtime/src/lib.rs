//! Embeddable runtime orchestration and typed runtime-host interfaces for Signal.

mod interfaces;
mod runtime;

pub use interfaces::{
    ActiveTransportConcurrencySession, BackendPolicyOverride, BlockDispatchRecord,
    BlockDispatchStage, BrokerFailureRecord, BrokerFailureStage, BrokerInvalidationRecord,
    BrokerInvalidationStage, CompletionSlotRecord, CompletionSlotStage, DegradedReason,
    EffectiveRuntimeConfig, GraphNodeProjection, GraphProjection, HandshakeRequest,
    HandshakeResponse, HeartbeatCycleRecord, HeartbeatCycleStage, LeaseRolloverRecord,
    LingeringCleanupMode, LingeringCleanupPlan, LingeringCleanupQueueReceipt,
    LingeringCleanupTrigger, LoopRegion, ParameterBatch, ParameterEvent, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginFaultRecord,
    PluginSandboxLifecycleRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportRecord, PluginSandboxTransportStage, PluginScanRequest,
    ProjectionReceipt, RecoveryRecord, RecoveryRestartIntent, RestartRequest,
    RuntimeAutomationSnapshot, RuntimeConfigRequest, RuntimeDiagnosticsSnapshot,
    RuntimeEngineBlockResult, RuntimeEngineBlockSnapshot, RuntimeError, RuntimeErrorKind,
    RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink, RuntimeLifecycleApi,
    RuntimeObservationApi, RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimePreworkBacklogClass, RuntimePreworkCacheState, RuntimePreworkForecastMode,
    RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
    RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
    RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason, RuntimePreworkRetirementReason,
    RuntimePreworkServicePressure, RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
    RuntimePreworkWindowTarget, RuntimeProjectionApi, RuntimeReadiness, RuntimeSupervisionSnapshot,
    RuntimeSupervisorApi, RuntimeSupervisorReport, RuntimeTimelineSnapshot,
    RuntimeTransportConcurrencySnapshot, RuntimeWatchdogTrigger, SafeModeRequest, SandboxHandle,
    SandboxOperationFailureRecord, SandboxOperationFailureStage, ScanHandle, ScheduleProjection,
    StopReason, SubscriptionHandle, TransportAttachIntent, TransportDispatchState,
    TransportFaultBoundaryMode, TransportFaultPhase, TransportFaultRecord, TransportFaultResource,
    TransportFaultSource, TransportFaultStage, TransportFaultSummary, TransportHeartbeatFreshness,
    TransportProjection, TransportSessionBoundaryMode, TransportSessionProvenance,
    TransportSessionState, TransportSessionSummary, WatchdogRestartRecord,
};
pub use runtime::{RuntimeConfig, RuntimeProfile, SignalRuntime};
