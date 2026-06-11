//! Embeddable runtime that orchestrates Signal's executable graph processing,
//! plugin sandbox lifecycle, transport, hardware, and supervision.
//!
//! # Lifecycle
//!
//! A fresh `SignalRuntime` goes through four control-plane steps before it can
//! process audio:
//!
//! 1. **Handshake** — `handshake()` exchanges version and capability
//!    information.  Returns a `HandshakeResponse` advertising what the
//!    runtime supports.
//! 2. **Configure** — `configure()` accepts a [`RuntimeConfigRequest`]
//!    (sample rate, block size, prework policy).
//! 3. **Start** — `start()` transitions the runtime to the `Running` state and
//!    arms the anticipative prework scheduler.
//! 4. **Stop** — `stop()` with a [`StopReason`] tears down active transport
//!    sessions and halts block processing.
//!
//! Between start and stop the runtime processes audio blocks, manages plugin
//! sandbox transport sessions, and runs anticipative prework through
//! `service_prework_lane()`.
//!
//! # Control plane vs. observation plane
//!
//! **Control plane** — [`SignalRuntime`] implements [`RuntimeLifecycleApi`],
//! [`RuntimeProjectionApi`], and [`RuntimeSupervisorApi`].  These mutating
//! methods drive state forward.
//!
//! **Observation plane** — [`RuntimeObservationApi`] exposes read-only
//! snapshots of all runtime subsystems.  Subscribing a `RuntimeEventSink`
//! (via `subscribe()`) delivers push events; [`RuntimeEventRecorder`] is the
//! standard in-memory accumulator.  The two planes are intentionally separate:
//! host code that only observes never needs a mutable borrow.
//!
//! # Supervisor reports
//!
//! [`RuntimeSupervisorReport`] packages the full observation report together
//! with the pending event stream.  It is the primary artifact for diagnostics,
//! test assertions, and structured logging.  Use `capture()` to snapshot one
//! from any `RuntimeObservationApi` + `RuntimeEventRecorder` pair.
//!
//! # Graph execution
//!
//! [`GraphProjection`] carries the ordered list of [`GraphNodeProjection`]s
//! that the engine will process each block.  The matching
//! [`GraphContractProjection`] carries per-node buffer contracts (bus
//! endpoints, channel layout, silence policy).  After each block,
//! `RuntimeEngineBlockSnapshot` records the complete execution state:
//! prework cache metrics, dispatch counts, latency, peak levels, and transport
//! position.  [`RuntimeEngineBlockResult`] pairs that snapshot with the output
//! `AudioBuffer` and metering sources.
//!
//! # Example
//!
//! ```no_run
//! use signal_runtime::{
//!     SignalRuntime, RuntimeConfig, RuntimeLifecycleApi, HandshakeRequest,
//!     RuntimeConfigRequest, StopReason,
//! };
//!
//! let config = RuntimeConfig::local(48_000, 512);
//! let mut runtime = SignalRuntime::new(config);
//!
//! let _response = runtime.handshake(HandshakeRequest {
//!     client_version: "1.0.0".into(),
//!     anticipative_preferred: true,
//!     max_sample_rate_hint: None,
//! }).unwrap();
//!
//! runtime.configure(RuntimeConfigRequest::new(48_000, 512)).unwrap();
//! runtime.start().unwrap();
//! // … process blocks …
//! runtime.stop(StopReason::UserRequested).unwrap();
//! ```

#![warn(missing_docs)]

mod interfaces;
mod runtime;
mod sandbox_broker_support;

#[cfg(test)]
pub(crate) use interfaces::RuntimeMediaIndexingState;
pub use interfaces::{
    ActiveTransportConcurrencySession, BackendPolicyOverride, BlockDispatchRecord,
    BlockDispatchStage, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection,
    HandshakeRequest, HeartbeatCycleStage, LingeringCleanupMode, LingeringCleanupTrigger,
    ParameterBatch, PluginBackedNodeBinding, PluginBackedNodeBindingProjection, PluginFaultKind,
    PluginNodeRender, PluginNodeRenderBatch, PluginSandboxInstanceFaultRecord,
    PluginSandboxInstanceStateRecord, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RecoveryRestartIntent, RestartRequest,
    RuntimeAuditionSinkAuthority, RuntimeAuditionSinkClass, RuntimeAuxiliaryPathKind,
    RuntimeAuxiliaryPathSummary, RuntimeBlockDeadlinePressure, RuntimeBusIntent, RuntimeBusRole,
    RuntimeCanonicalChannelLayout, RuntimeClipFadeEnvelope, RuntimeClipGainEnvelope,
    RuntimeClipProcessingRegistration, RuntimeClipRenderInputStage, RuntimeClipRenderRequest,
    RuntimeConfigRequest, RuntimeDeferredServiceBackpressureSource,
    RuntimeDeferredServiceCancellationCause, RuntimeDeferredServiceDecision,
    RuntimeDeferredServicePriorityBand, RuntimeDeferredServiceReason, RuntimeDeploymentClass,
    RuntimeDeviceFaultBoundaryState, RuntimeDeviceRestartState, RuntimeDeviceSupervisionState,
    RuntimeDynamicBusNegotiationPosture, RuntimeEngineBlockResult, RuntimeError, RuntimeErrorKind,
    RuntimeEvent, RuntimeEventRecorder, RuntimeExecutionTopologySummary,
    RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState, RuntimeExternalIoMonitoringState,
    RuntimeExternalIoMonitoringTapPoint, RuntimeExternalIoPrimaryRole, RuntimeFaultCause,
    RuntimeFaultDiagnosticAuthority, RuntimeFaultDiagnosticFamily, RuntimeFoldDownPolicy,
    RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
    RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
    RuntimeHostClockFallbackState, RuntimeHostClockSource, RuntimeHostClockTransitionState,
    RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
    RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
    RuntimeHostLifecycleOwnership, RuntimeHostObservationReport, RuntimeHostRestartPolicy,
    RuntimeHostSupervisorReport, RuntimeImmersiveExportAuthority, RuntimeImmersiveExportClass,
    RuntimeImmersiveExportOutcome, RuntimeImmersiveObjectRenderingPosture,
    RuntimeImmersiveRoomOutcome, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeLowLatencyDevicePolicyClass, RuntimeLowLatencyDevicePolicyOutcome,
    RuntimeLv2ExtensionCapabilitySummary, RuntimeLv2ExtensionNegotiationState,
    RuntimeLv2PatchExchangePosture, RuntimeLv2PreparedNegotiationRecord,
    RuntimeLv2UridNegotiationPosture, RuntimeLv2WorkerPosture, RuntimeMediaAnalysisDescriptorState,
    RuntimeMediaAnalysisFamilyState, RuntimeMediaAssetRegistration,
    RuntimeMediaAuditionContinuityOutcome, RuntimeMediaAuditionOrchestrationAuthority,
    RuntimeMediaAuditionOrchestrationPosture, RuntimeMediaPreviewState, RuntimeMonitoringOutcome,
    RuntimeMonitoringSceneAuthority, RuntimeMonitoringSceneClass, RuntimeMultichannelIoSummary,
    RuntimeObservationApi, RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginBusCapableFxClass,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord, RuntimePluginDispatchState,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginLifecycleState,
    RuntimePluginNegotiationFallbackOutcome, RuntimePluginParityBand,
    RuntimePluginPinGroupIdentity, RuntimePluginPinMatrixPosture, RuntimePluginPlacementPolicy,
    RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher, RuntimePluginPresetDescriptor,
    RuntimePluginPresetOrigin, RuntimePluginRecallHandoffSelection,
    RuntimePluginRecallHandoffStageId, RuntimePluginRecallPortabilityClass,
    RuntimePluginScanDiagnosticKind, RuntimePluginScanDiagnosticRecord,
    RuntimePreviewBrowserQueueClass, RuntimePreviewBrowserQueueOutcome,
    RuntimePreviewBrowserQueuePosture, RuntimePreviewOutputRoutingPosture,
    RuntimePreviewTransformReadiness, RuntimePreviewTransformSchedulingAuthority,
    RuntimePreviewTransformSchedulingOutcome, RuntimePreviewTransformSchedulingPosture,
    RuntimePreviewTransformServiceClass, RuntimePreworkCacheState, RuntimePreworkForecastMode,
    RuntimePreworkForecastProfile, RuntimePreworkForecastProfileSource,
    RuntimePreworkFreshnessState, RuntimePreworkRetirementReason, RuntimePreworkServicePressure,
    RuntimePreworkServiceSemanticPolicy, RuntimeProjectionApi, RuntimeReadiness,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureKind, RuntimeRecordingCaptureStartRequest, RuntimeRecordingCaptureState,
    RuntimeRecoveryState, RuntimeRendererCapabilityAuthority,
    RuntimeRendererCapabilityNegotiationPosture, RuntimeRoomPolicyAuthority,
    RuntimeRoomPolicyClass, RuntimeSecondaryInputAttachmentPolicy,
    RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputFallbackOutcome,
    RuntimeSecondaryInputSourceKind, RuntimeSecondaryInputTargetKind, RuntimeSpatialAdapterClass,
    RuntimeSpatialBedClass, RuntimeSpatialExecutionMode, RuntimeSpatialExpandedFallbackOutcome,
    RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy, RuntimeSpatialRenderScope,
    RuntimeSpatialTargetEnvironment, RuntimeStretchEngineClass, RuntimeStretchFallbackKind,
    RuntimeStretchReadiness, RuntimeSupervisorApi, RuntimeSupervisorReport,
    RuntimeTempoAssistHintSource, RuntimeTempoAssistPosture, RuntimeTransformArtifactReadiness,
    RuntimeTransformArtifactReuseState, RuntimeTransformCachePlacementOutcome,
    RuntimeTransformPersistencePosture, RuntimeTransformRetentionOutcome,
    RuntimeTransportSessionAttachRequest, RuntimeWarpClipRegistration, RuntimeWarpMode,
    RuntimeWatchdogTrigger, SafeModeRequest, SandboxHandle, SandboxOperationFailureRecord,
    SandboxOperationFailureStage, ScanHandle, StopReason, TransportAttachIntent,
    TransportDispatchState, TransportHeartbeatFreshness, TransportProjection,
    TransportSessionBoundaryMode, TransportSessionProvenance, TransportSessionState,
    TransportSessionSummary, WatchdogRestartRecord,
};
pub(crate) use interfaces::{
    RuntimeMultichannelLayoutSummary, RuntimePreworkBacklogClass, RuntimePreworkForecastPolicy,
};
pub use runtime::{RuntimeConfig, SignalRuntime};
pub use sandbox_broker_support::{
    ensure_prepared_sandbox_session, record_broker_attached_execution_summary,
    record_broker_sandbox_prepared, record_protocol_violation_prepare_failure,
    teardown_broker_sandbox_session, PreparedBrokerSandboxSpec, PreparedSandboxSessionRecord,
    SandboxBrokerSession, SandboxBrokerSpawnConfig,
};
