//! Thin control library for Signal runtime supervision.
//!
//! Production audio execution lives in `signal-render-plane`. This crate is
//! the control/observation surface the local host assembly
//! (`signal-host-local`) and pulse consume: lifecycle handshake, graph plan
//! vocabulary, plugin discovery and sandbox lifecycle records, the media
//! decode/analysis pipeline, recording capture, and the supervisor/observation
//! reports.
//!
//! # Lifecycle
//!
//! A fresh `SignalRuntime` goes through the control-plane steps
//! `handshake() → configure() → start() → stop()`. Between configure and stop
//! the host applies graph projections and manages plugin sandboxes; the
//! observation plane ([`RuntimeObservationApi`]) exposes read-only snapshots
//! at any point.
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
//! runtime.stop(StopReason::UserRequested).unwrap();
//! ```

#![warn(missing_docs)]

mod interfaces;
mod runtime;
mod sandbox_broker_support;

pub use interfaces::{
    BackendPolicyOverride, BrokerFailureStage, BrokerInvalidationStage, CompletionSlotStage,
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeBusEndpointProjection,
    GraphNodeContractProjection, GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection,
    HandshakeRequest, HandshakeResponse, HeartbeatCycleStage, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginSandboxInstanceStateRecord,
    PluginSandboxLifecycleStage, PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest,
    RuntimeClipProcessingRegistration, RuntimeConfigRequest, RuntimeError, RuntimeErrorKind,
    RuntimeEvent, RuntimeEventRecorder, RuntimeEventSink, RuntimeExecutionTopologySummary,
    RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState, RuntimeExternalIoMonitoringState,
    RuntimeExternalIoMonitoringTapPoint, RuntimeExternalIoPrimaryRole, RuntimeExternalIoSnapshot,
    RuntimeHostAudioPumpSummary, RuntimeHostAudioStreamState, RuntimeHostClockDiscontinuityState,
    RuntimeHostClockDomain, RuntimeHostClockDriftState, RuntimeHostClockFallbackState,
    RuntimeHostClockSource, RuntimeHostClockTransitionState, RuntimeHostClockingSummary,
    RuntimeHostDuplexMismatchState, RuntimeHostEndpointTopology, RuntimeHostHardwareSummary,
    RuntimeHostIoSummary, RuntimeHostLatencySummary, RuntimeHostLifecycleOwnership,
    RuntimeHostObservationReport, RuntimeHostRestartPolicy, RuntimeHostSupervisorReport,
    RuntimeLifecycleApi, RuntimeLv2ExtensionCapabilitySummary, RuntimeLv2PreparedNegotiationRecord,
    RuntimeMediaAnalysisDescriptorState, RuntimeMediaAnalysisFamilyState,
    RuntimeMediaAssetRegistration, RuntimeMediaPreviewState, RuntimeMultichannelIoSummary,
    RuntimeObservationApi, RuntimeObservationReport, RuntimeOfflineStretchArtifactPlanRegistration,
    RuntimeOfflineStretchArtifactPlanSnapshot, RuntimeOfflineStretchArtifactPlanSnapshotSet,
    RuntimeOfflineStretchArtifactReadiness, RuntimeOfflineStretchArtifactScope,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord,
    RuntimePluginFormatPlatformCoverageRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginLifecycleState, RuntimePluginParityBand,
    RuntimeProjectionApi, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureStartRequest, RuntimeSupervisorApi, RuntimeSupervisorReport,
    RuntimeWarpClipRegistration, SandboxHandle, ScanHandle, StopReason,
};
pub use runtime::{RuntimeConfig, SignalRuntime};
pub use sandbox_broker_support::{
    ensure_prepared_sandbox_session, record_broker_attached_execution_summary,
    record_broker_sandbox_prepared, record_protocol_violation_prepare_failure,
    teardown_broker_sandbox_session, PreparedBrokerSandboxSpec, PreparedSandboxSessionRecord,
    SandboxBrokerAttachedSession, SandboxBrokerClientSession, SandboxBrokerReceiptState,
    SandboxBrokerSession, SandboxBrokerSpawnConfig, SandboxPluginActivateOutcome,
    SandboxPluginAudioLease, SandboxPluginInventory, SandboxPluginParameter,
};
pub use signal_dsp_stretch::{
    StretchBackendTier, StretchCacheIdentityInput, StretchChannelLayout, StretchPitchPoint,
    StretchRatioPoint, StretchWarpMarker,
};
