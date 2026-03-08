//! Embeddable runtime orchestration and typed runtime-host interfaces for Signal.

mod interfaces;
mod runtime;

pub use interfaces::{
    BackendPolicyOverride, DegradedReason, EffectiveRuntimeConfig, GraphProjection,
    HandshakeRequest, HandshakeResponse, LoopRegion, ParameterBatch, ParameterEvent,
    PluginFaultKind, PluginFaultRecord, PluginSandboxSpec, PluginScanRequest, ProjectionReceipt,
    RestartRequest, RuntimeAutomationSnapshot, RuntimeConfigRequest, RuntimeDiagnosticsSnapshot,
    RuntimeError, RuntimeErrorKind, RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink,
    RuntimeLifecycleApi, RuntimeObservationApi, RuntimeObservationDiagnostics,
    RuntimeObservationReport, RuntimeProjectionApi, RuntimeReadiness, RuntimeSupervisionSnapshot,
    RuntimeSupervisorApi, RuntimeSupervisorReport, RuntimeTimelineSnapshot, RuntimeWatchdogTrigger,
    SafeModeRequest, SandboxHandle, ScanHandle, ScheduleProjection, StopReason, SubscriptionHandle,
    TransportProjection, WatchdogRestartRecord,
};
pub use runtime::{RuntimeConfig, RuntimeProfile, SignalRuntime};
