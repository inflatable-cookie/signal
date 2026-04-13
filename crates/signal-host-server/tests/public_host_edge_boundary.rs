#![allow(dead_code, unused_imports)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphNodeTopologyRole,
    GraphStageSpec,
};
use signal_host_server::ServerRuntimeHost;
use signal_plugin::{EventPacketSummary, PluginFeature, PluginFormat, PluginIoLayout};
use signal_primitives::{AudioBuffer, ChannelCount, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginFaultKind, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RestartRequest,
    RuntimeAuxiliaryPathKind, RuntimeBlockDeadlinePressure, RuntimeBusIntent, RuntimeBusRole,
    RuntimeCanonicalChannelLayout, RuntimeConfig, RuntimeConfigRequest,
    RuntimeDeferredServiceCancellationCause, RuntimeDeferredServiceDecision,
    RuntimeDeferredServicePriorityBand, RuntimeDeferredServiceReason, RuntimeDeploymentClass,
    RuntimeDeviceFaultBoundaryState, RuntimeDeviceRestartState, RuntimeDeviceSupervisionState,
    RuntimeError, RuntimeErrorKind, RuntimeExternalIoHealthState, RuntimeExternalIoLoopbackState,
    RuntimeExternalIoMonitoringState, RuntimeExternalIoMonitoringTapPoint,
    RuntimeExternalIoPrimaryRole, RuntimeFoldDownPolicy, RuntimeImmersiveExportAuthority,
    RuntimeImmersiveExportClass, RuntimeImmersiveExportOutcome,
    RuntimeImmersiveObjectRenderingPosture, RuntimeImmersiveRoomOutcome, RuntimeInterruptionClass,
    RuntimeJackClientRole, RuntimeJackGraphCoordinationState, RuntimeJackGuardedCoordinationState,
    RuntimeJackTransportPosture, RuntimeLifecycleApi, RuntimeMonitoringOutcome,
    RuntimeMonitoringSceneAuthority, RuntimeMonitoringSceneClass, RuntimeObservationApi,
    RuntimeOfflineRenderExecutionState, RuntimeOfflineRenderPurgeRequest,
    RuntimeOfflineRenderRequest, RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginBusCapableFxClass,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord, RuntimePluginHostPlatform,
    RuntimePluginIsolationOutcome, RuntimePluginParityBand, RuntimePluginPlacementPolicy,
    RuntimePluginPlacementRule, RuntimePluginPlacementRuleMatcher,
    RuntimePluginRecallPortabilityClass, RuntimeProjectionApi,
    RuntimeRecordingCaptureCheckpointClass, RuntimeRecordingCaptureKind,
    RuntimeRecordingCaptureStartRequest, RuntimeRecoveryState, RuntimeRendererCapabilityAuthority,
    RuntimeRendererCapabilityNegotiationPosture, RuntimeRoomPolicyAuthority,
    RuntimeRoomPolicyClass, RuntimeSecondaryInputAttachmentPolicy,
    RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputFallbackOutcome,
    RuntimeSecondaryInputTargetKind, RuntimeSpatialBedClass, RuntimeSpatialExecutionMode,
    RuntimeSpatialExpandedFallbackOutcome, RuntimeSpatialFallbackOutcome, RuntimeSpatialMixPolicy,
    RuntimeSupervisorApi, RuntimeWatchdogTrigger, SignalRuntime, StopReason, WatchdogRestartRecord,
};

#[path = "public_host_edge_boundary/fixtures.rs"]
mod fixtures;
#[path = "public_host_edge_boundary/graphs.rs"]
mod graphs;

pub(crate) use fixtures::record_public_plugin_sandbox_ready;
pub(crate) use graphs::{
    apply_public_capture_graph, apply_public_render_graph, apply_public_sidechain_graph,
};

#[path = "public_host_edge_boundary/consumable_surface.rs"]
mod consumable_surface;
#[path = "public_host_edge_boundary/deferred_work.rs"]
mod deferred_work;
#[path = "public_host_edge_boundary/linux_parity_truth.rs"]
mod linux_parity_truth;
#[path = "public_host_edge_boundary/performance_truth.rs"]
mod performance_truth;
#[path = "public_host_edge_boundary/sidechain_truth.rs"]
mod sidechain_truth;
