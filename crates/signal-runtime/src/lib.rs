//! Embeddable runtime that orchestrates Signal's executable graph processing,
//! plugin sandbox lifecycle, transport, hardware, and supervision.
//!
//! # Lifecycle
//!
//! A fresh `SignalRuntime` goes through four control-plane steps before it can
//! process audio:
//!
//! 1. **Handshake** — `handshake()` exchanges version and capability
//!    information.  Returns a [`HandshakeResponse`] advertising what the
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
//! snapshots of all runtime subsystems.  Subscribing a [`RuntimeEventSink`]
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
//! [`RuntimeEngineBlockSnapshot`] records the complete execution state:
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


mod host_unification_support;
mod interfaces;
mod runtime;
mod sandbox_broker_support;

pub use host_unification_support::{RepeatedWatchdogRecoveryPlan, TimeoutRecoveryRetryPlan};
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
    RuntimeAcceptanceReceipt, RuntimeAdvancedControlFeedbackAuthority,
    RuntimeAdvancedControlFeedbackOutcome, RuntimeAdvancedHardwareActionClass,
    RuntimeAdvancedHardwareCapabilitySummary, RuntimeAdvancedHardwareDeviceDescriptor,
    RuntimeAdvancedHardwareGraphState, RuntimeAdvancedHardwareSnapshot,
    RuntimeAuditionSinkAuthority, RuntimeAuditionSinkClass, RuntimeAutomationInterpolation,
    RuntimeAutomationLaneProjection, RuntimeAutomationPointProjection, RuntimeAutomationProjection,
    RuntimeAutomationResolution, RuntimeAutomationSnapshot, RuntimeAutomationTargetProjection,
    RuntimeAuxiliaryPathKind, RuntimeAuxiliaryPathSummary, RuntimeBlockDeadlinePressure,
    RuntimeBusConnectionAttachmentClass, RuntimeBusConnectionFallbackOutcome,
    RuntimeBusConnectionSummary, RuntimeBusIntent, RuntimeBusRole, RuntimeCanonicalChannelLayout,
    RuntimeChannelRole, RuntimeClipFadeEnvelope, RuntimeClipFadeShape, RuntimeClipGainEnvelope,
    RuntimeClipGainShape, RuntimeClipProcessingPipelineSnapshot, RuntimeClipProcessingReadiness,
    RuntimeClipProcessingRegistration, RuntimeClipProcessingSnapshot, RuntimeClipProcessingStage,
    RuntimeClipRenderInputStage, RuntimeClipRenderRequest, RuntimeClipRenderResult,
    RuntimeConfigRequest, RuntimeControlSurfaceCapabilitySummary,
    RuntimeControlSurfaceDeviceDescriptor, RuntimeControlSurfaceFeedbackReadiness,
    RuntimeControlSurfaceGraphState, RuntimeControlSurfaceMappingPosture,
    RuntimeControlSurfaceSnapshot, RuntimeControlSurfaceTransportPosture,
    RuntimeControlSurfaceWorkflowAuthority, RuntimeControllerExpressionMidi2Posture,
    RuntimeControllerExpressionMpePosture, RuntimeDeferredServiceBackpressureSource,
    RuntimeDeferredServiceCancellationCause, RuntimeDeferredServiceClass,
    RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
    RuntimeDeferredServiceReason, RuntimeDeferredServiceReceipt, RuntimeDeploymentClass,
    RuntimeDeploymentMonitoringSummary, RuntimeDeviceFaultBoundaryState, RuntimeDeviceRestartState,
    RuntimeDeviceSupervisionSnapshot, RuntimeDeviceSupervisionState, RuntimeDiagnosticsSnapshot,
    RuntimeDisplayContentClass, RuntimeDisplayTransportPosture,
    RuntimeDynamicBusNegotiationPosture, RuntimeEngineBlockResult, RuntimeEngineBlockSnapshot,
    RuntimeError, RuntimeErrorKind, RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink,
    RuntimeExecutionPhase, RuntimeExecutionTopologySummary, RuntimeExternalIoDeviceChangeState,
    RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState, RuntimeExternalIoMonitoringState,
    RuntimeExternalIoMonitoringTapPoint, RuntimeExternalIoPrimaryRole, RuntimeExternalIoSnapshot,
    RuntimeExternalMidiAttachContinuity, RuntimeExternalMidiBackendParity,
    RuntimeExternalMidiDeviceDescriptor, RuntimeExternalMidiDeviceLifecycleState,
    RuntimeExternalMidiDiscoveryState, RuntimeExternalMidiEndpointCapabilitySummary,
    RuntimeExternalMidiEndpointDescriptor, RuntimeExternalMidiEndpointDirection,
    RuntimeExternalMidiEndpointGraphSnapshot, RuntimeExternalMidiEndpointLifecycleState,
    RuntimeExternalMidiGraphState, RuntimeExternalMidiGuardedParityOutcome,
    RuntimeExternalMidiLiveOwnershipPosture, RuntimeExternalMidiLiveOwnershipSummary,
    RuntimeExternalMidiRouteState, RuntimeFaultCause, RuntimeFaultContributionReceipt,
    RuntimeFaultDiagnosticAuthority, RuntimeFaultDiagnosticFamily, RuntimeFaultDiagnosticReceipt,
    RuntimeFaultStatusSnapshot, RuntimeFeedbackPageClass, RuntimeFeedbackPagePosture,
    RuntimeFoldDownPolicy, RuntimeGuardedFeedbackChannelPosture, RuntimeHapticTransportPosture,
    RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostAudioTransferPolicy,
    RuntimeHostClockDiscontinuityState, RuntimeHostClockDomain, RuntimeHostClockDriftState,
    RuntimeHostClockFallbackState, RuntimeHostClockSource, RuntimeHostClockTransitionState,
    RuntimeHostClockingSummary, RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology,
    RuntimeHostHardwareSummary, RuntimeHostIoSummary, RuntimeHostLatencySummary,
    RuntimeHostLifecycleOwnership, RuntimeHostObservationReport, RuntimeHostRestartPolicy,
    RuntimeHostSupervisorReport, RuntimeImmersiveExportAuthority, RuntimeImmersiveExportClass,
    RuntimeImmersiveExportOutcome, RuntimeImmersiveObjectRenderingPosture,
    RuntimeImmersiveRoomOutcome, RuntimeImmersiveRoomPolicySummary, RuntimeInterruptionClass,
    RuntimeInterruptionSummary, RuntimeJackClientRole, RuntimeJackCoordinationSnapshot,
    RuntimeJackGraphCoordinationState, RuntimeJackGuardedCoordinationState,
    RuntimeJackTransportPosture, RuntimeLifecycleApi, RuntimeLinuxAudioBackendClockingParityBand,
    RuntimeLinuxAudioBackendDuplexParityState, RuntimeLinuxAudioBackendEndpointTopologyParityState,
    RuntimeLinuxAudioBackendIdentity, RuntimeLinuxAudioBackendPortabilityBand,
    RuntimeLinuxBackendDeviceClaimPosture, RuntimeLinuxBackendOwnershipFallbackState,
    RuntimeLinuxBackendSessionLifecycleState, RuntimeLinuxBackendSessionOwnership,
    RuntimeLinuxBackendSessionRole, RuntimeLinuxBackendSessionSnapshot,
    RuntimeLinuxHostIoParityInput, RuntimeLowLatencyDevicePolicyClass,
    RuntimeLowLatencyDevicePolicyOutcome, RuntimeLv2ExtensionCapabilitySummary,
    RuntimeLv2ExtensionNegotiationState, RuntimeLv2ExtensionRecord, RuntimeLv2ExtensionSnapshot,
    RuntimeLv2PatchCapability, RuntimeLv2PatchExchangePosture, RuntimeLv2PreparedNegotiationRecord,
    RuntimeLv2UridCapability, RuntimeLv2UridNegotiationPosture, RuntimeLv2WorkerCapability,
    RuntimeLv2WorkerPosture, RuntimeMarkerAnalysisClipSnapshot,
    RuntimeMarkerAnalysisInvalidationState, RuntimeMarkerAnalysisReadiness,
    RuntimeMarkerAnalysisSnapshot, RuntimeMediaAnalysisDescriptorState,
    RuntimeMediaAnalysisFamilyState, RuntimeMediaAssetRegistration, RuntimeMediaAssetSnapshot,
    RuntimeMediaAssetState, RuntimeMediaAuditionContinuityOutcome,
    RuntimeMediaAuditionOrchestrationAuthority, RuntimeMediaAuditionOrchestrationPosture,
    RuntimeMediaCharacterDescriptor, RuntimeMediaIndexingState, RuntimeMediaLibraryAssetDescriptor,
    RuntimeMediaLibraryServiceSnapshot, RuntimeMediaLoudnessDescriptor,
    RuntimeMediaPipelineSnapshot, RuntimeMediaPreviewState, RuntimeMediaServiceSnapshot,
    RuntimeMeterSourceRole, RuntimeMeterSourceSnapshot, RuntimeMeteringSnapshot,
    RuntimeMonitoringOutcome, RuntimeMonitoringSceneAuthority, RuntimeMonitoringSceneClass,
    RuntimeMotorTransportPosture, RuntimeMultichannelIoSummary, RuntimeMultichannelLayoutSummary,
    RuntimeObservationApi, RuntimeObservationDiagnostics, RuntimeObservationReport,
    RuntimeOfflineFreezeArtifactPreview, RuntimeOfflineFreezeArtifactRequest,
    RuntimeOfflineFreezeArtifactResult, RuntimeOfflinePluginDelegatedExecutionMerge,
    RuntimeOfflinePluginDelegatedExecutionOutcome, RuntimeOfflinePluginDelegatedExecutionReceipt,
    RuntimeOfflinePluginDelegatedExecutionRequest,
    RuntimeOfflinePluginDelegatedExecutionStageReceipt,
    RuntimeOfflinePluginDelegatedExecutionStageRequest,
    RuntimeOfflinePluginDelegatedExecutionStatus,
    RuntimeOfflinePluginDelegatedFreezeArtifactOutput, RuntimeOfflinePluginDelegatedStemOutput,
    RuntimeOfflinePluginExecutionBoundary, RuntimeOfflinePluginExecutionOwner,
    RuntimeOfflinePluginExecutionStageBoundary, RuntimeOfflinePluginOverrideState,
    RuntimeOfflineRenderArtifactKind, RuntimeOfflineRenderArtifactReceipt,
    RuntimeOfflineRenderCheckpointReceipt, RuntimeOfflineRenderCheckpointStage,
    RuntimeOfflineRenderComplexIoStageSummary, RuntimeOfflineRenderContractPreview,
    RuntimeOfflineRenderExecutionCancellationReceipt, RuntimeOfflineRenderExecutionProgressReceipt,
    RuntimeOfflineRenderExecutionReceipt, RuntimeOfflineRenderExecutionState,
    RuntimeOfflineRenderManifest, RuntimeOfflineRenderProfilingReceipt,
    RuntimeOfflineRenderPurgeReceipt, RuntimeOfflineRenderPurgeRequest,
    RuntimeOfflineRenderQueueProgressReceipt, RuntimeOfflineRenderQueueResult,
    RuntimeOfflineRenderReportReceipt, RuntimeOfflineRenderRequest, RuntimeOfflineRenderResult,
    RuntimeOfflineRenderSoakReceipt, RuntimeOfflineRenderSpatialStageSummary,
    RuntimeOfflineRenderStemPreview, RuntimeOfflineRenderStemResult,
    RuntimeOfflineRenderStemTarget, RuntimeOfflineRenderTargetKind, RuntimePerformanceSnapshot,
    RuntimePerformanceTraceReceipt, RuntimePipeWireAlsaDeviceClaimParity,
    RuntimePipeWireAlsaGuardedParityState, RuntimePipeWireAlsaParitySnapshot,
    RuntimePipeWireAlsaSessionRoleParity, RuntimePipeWireAlsaStreamPolicyParity,
    RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginBusCapableFxClass,
    RuntimePluginCapabilityCoverageSummary, RuntimePluginChainSnapshot,
    RuntimePluginChainStageSnapshot, RuntimePluginCompensationState, RuntimePluginComplexIoSummary,
    RuntimePluginDiscoveredTypeRecord, RuntimePluginDiscoverySnapshot, RuntimePluginDispatchState,
    RuntimePluginEventSnapshot, RuntimePluginExecutionChainSummary,
    RuntimePluginFormatCoverageRecord, RuntimePluginFormatParityRecord,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginInterchangeSnapshot, RuntimePluginIsolationOutcome,
    RuntimePluginLifecycleSnapshot, RuntimePluginLifecycleState,
    RuntimePluginNegotiationFallbackOutcome, RuntimePluginParityBand,
    RuntimePluginPinGroupIdentity, RuntimePluginPinMatrixPosture, RuntimePluginPinMatrixRecord,
    RuntimePluginPinMatrixSnapshot, RuntimePluginPlacementPolicy, RuntimePluginPlacementRule,
    RuntimePluginPlacementRuleMatcher, RuntimePluginPortClass, RuntimePluginPresetDescriptor,
    RuntimePluginPresetOrigin, RuntimePluginRecallHandoffSelection,
    RuntimePluginRecallHandoffSnapshot, RuntimePluginRecallHandoffStage,
    RuntimePluginRecallHandoffStageId, RuntimePluginRecallPayload,
    RuntimePluginRecallPortabilityClass, RuntimePluginRecallSnapshot, RuntimePluginRecallState,
    RuntimePluginSandboxSnapshot, RuntimePluginScanDiagnosticKind,
    RuntimePluginScanDiagnosticRecord, RuntimePluginScanReceipt,
    RuntimePluginTopologyAttachmentPolicy, RuntimePluginTopologyFallbackOutcome,
    RuntimePreviewBrowserQueueClass, RuntimePreviewBrowserQueueOutcome,
    RuntimePreviewBrowserQueuePosture, RuntimePreviewDevicePolicySummary,
    RuntimePreviewOutputRoutingPosture, RuntimePreviewTransformClipSnapshot,
    RuntimePreviewTransformDegradedState, RuntimePreviewTransformFallbackKind,
    RuntimePreviewTransformReadiness, RuntimePreviewTransformSchedulingAuthority,
    RuntimePreviewTransformSchedulingOutcome, RuntimePreviewTransformSchedulingPosture,
    RuntimePreviewTransformServiceClass, RuntimePreviewTransformServiceSnapshot,
    RuntimePreviewWorkflowSummary, RuntimePreworkBacklogClass, RuntimePreworkCacheState,
    RuntimePreworkForecastMode, RuntimePreworkForecastPolicy, RuntimePreworkForecastProfile,
    RuntimePreworkForecastProfileSelection, RuntimePreworkForecastProfileSource,
    RuntimePreworkFreshnessState, RuntimePreworkInvalidationReason, RuntimePreworkRetirementReason,
    RuntimePreworkServicePressure, RuntimePreworkServiceSemanticPolicy, RuntimePreworkServiceState,
    RuntimePreworkWindowTarget, RuntimeProfilingReceipt, RuntimeProjectionApi, RuntimeReadiness,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureCheckpointSnapshot,
    RuntimeRecordingCaptureCommitReceipt, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureSnapshot, RuntimeRecordingCaptureStartRequest,
    RuntimeRecordingCaptureState, RuntimeRecoveryState, RuntimeRendererCapabilityAuthority,
    RuntimeRendererCapabilityNegotiationPosture, RuntimeRendererImmersiveExportSummary,
    RuntimeRoomPolicyAuthority, RuntimeRoomPolicyClass, RuntimeRoutedPluginChainSummary,
    RuntimeSafeActionGraphPosture, RuntimeSafeActionOutcome, RuntimeSceneMappingPosture,
    RuntimeSchedulerSnapshot, RuntimeSchedulerState, RuntimeSchedulerTopologyIssue,
    RuntimeSchedulerTopologySummary, RuntimeScriptingSafeDevicePolicyPosture,
    RuntimeSecondaryInputAttachmentPolicy, RuntimeSecondaryInputContractProjection,
    RuntimeSecondaryInputFallbackOutcome, RuntimeSecondaryInputRouteSummary,
    RuntimeSecondaryInputSourceKind, RuntimeSecondaryInputTargetKind, RuntimeSoakReceipt,
    RuntimeSpatialActivationPolicy, RuntimeSpatialAdapterClass, RuntimeSpatialBedClass,
    RuntimeSpatialControlFamily, RuntimeSpatialExecutionMode, RuntimeSpatialExecutionSummary,
    RuntimeSpatialExpandedFallbackOutcome, RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy,
    RuntimeSpatialObjectRole, RuntimeSpatialRenderScope, RuntimeSpatialTargetEnvironment,
    RuntimeStretchClipSnapshot, RuntimeStretchEngineClass, RuntimeStretchEngineSnapshot,
    RuntimeStretchFallbackKind, RuntimeStretchReadiness, RuntimeSupervisionSnapshot,
    RuntimeSupervisorApi, RuntimeSupervisorReport, RuntimeTempoAssistHintSource,
    RuntimeTempoAssistPosture, RuntimeTempoMapInterpolation, RuntimeTempoMapProjection,
    RuntimeTempoMapSegmentProjection, RuntimeTempoMapSegmentSnapshot, RuntimeTempoMapSnapshot,
    RuntimeTempoSource, RuntimeTimelineSnapshot, RuntimeTransformArtifactClipSnapshot,
    RuntimeTransformArtifactInvalidationState, RuntimeTransformArtifactReadiness,
    RuntimeTransformArtifactReuseState, RuntimeTransformArtifactSnapshot,
    RuntimeTransformCachePlacementAuthority, RuntimeTransformCachePlacementOutcome,
    RuntimeTransformCachePlacementPosture, RuntimeTransformPersistencePosture,
    RuntimeTransformPersistenceSummary, RuntimeTransformRetentionAuthority,
    RuntimeTransformRetentionOutcome, RuntimeTransformRetentionPolicyClass,
    RuntimeTransportConcurrencySnapshot, RuntimeTransportObservationSnapshot,
    RuntimeTransportSessionAttachRequest, RuntimeTransportTransitionKind,
    RuntimeWarpClipRegistration, RuntimeWarpClipSnapshot, RuntimeWarpMode,
    RuntimeWarpPipelineSnapshot, RuntimeWarpReadiness, RuntimeWatchdogTrigger, SafeModeRequest,
    SandboxHandle, SandboxOperationFailureRecord, SandboxOperationFailureStage, ScanHandle,
    ScheduleProjection, StopReason, SubscriptionHandle, TransportAttachIntent,
    TransportDispatchState, TransportFaultBoundaryMode, TransportFaultPhase, TransportFaultRecord,
    TransportFaultResource, TransportFaultSource, TransportFaultStage, TransportFaultSummary,
    TransportHeartbeatFreshness, TransportProjection, TransportSessionBoundaryMode,
    TransportSessionProvenance, TransportSessionState, TransportSessionSummary,
    WatchdogRestartRecord,
};
pub use runtime::{RuntimeConfig, RuntimeProfile, SignalRuntime};
pub use sandbox_broker_support::{
    begin_brokered_recovery_cycle, begin_recovery_overlap, complete_broker_transport_detach,
    complete_lingering_recovery_restart_or_rollback, complete_recovery_overlap_restart,
    complete_recovery_overlap_restart_or_rollback, ensure_broker_sandbox_session,
    ensure_prepared_sandbox_session, finalize_brokered_recovery_transport_detach,
    handle_overlap_prepare_contention, handle_recovery_overlap_old_transport_teardown,
    record_broker_attached_execution_summary, record_broker_sandbox_detached,
    record_broker_sandbox_prepared, record_broker_transport_detach_failure,
    record_broker_transport_detach_fault, record_broker_transport_detach_requested,
    record_protocol_violation_prepare_failure, rollback_recovery_overlap,
    run_vst3_broker_execution_sequence, teardown_broker_sandbox_session, PreparedBrokerSandboxSpec,
    PreparedSandboxSessionRecord, RecoveryOverlapOldTransportTeardownOutcome,
    SandboxBrokerAttachedSession, SandboxBrokerClientSession, SandboxBrokerFlavor,
    SandboxBrokerSession, SandboxBrokerSpawnConfig,
};
