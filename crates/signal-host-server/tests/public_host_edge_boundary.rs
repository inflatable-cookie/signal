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

mod fixtures;
mod graphs;

pub(crate) use fixtures::{
    public_server_media_fixture_path, record_public_plugin_sandbox_ready,
    sample_complex_bus_fx_record, sample_complex_multi_output_record, sample_server_ara_context,
    write_public_test_wav, write_public_transient_test_wav,
};
pub(crate) use graphs::{
    apply_public_capture_graph, apply_public_complex_io_graph, apply_public_multi_bus_graph,
    apply_public_multichannel_graph, apply_public_plugin_continuity_graph,
    apply_public_render_graph, apply_public_sidechain_graph, apply_public_spatial_graph,
};

mod consumable_surface;
mod deferred_work;
mod linux_parity_truth;
mod performance_truth;
mod sidechain_truth;
