#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphNodeTopologyRole,
    GraphStageSpec,
};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::{PluginFeature, PluginFormat, PluginIoLayout};
use signal_primitives::{ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, PluginSandboxLifecycleStage,
    PluginSandboxSpec, PluginSandboxTransportStage, PluginScanRequest, RuntimeBlockDeadlinePressure,
    RuntimeConfig, RuntimeConfigRequest, RuntimeDeferredServiceBackpressureSource,
    RuntimeDeferredServiceDecision, RuntimeDeferredServicePriorityBand,
    RuntimeDeferredServiceReason, RuntimeInterruptionClass, RuntimeLifecycleApi,
    RuntimeOfflineRenderRequest, RuntimePluginAraContextSnapshot, RuntimePluginAraDocumentContext,
    RuntimePluginAraRegionContext, RuntimePluginAraSourceContext, RuntimePluginComplexIoSummary,
    RuntimePluginDiscoveredTypeRecord, RuntimePluginPresetDescriptor,
    RuntimePluginPresetOrigin, RuntimePluginRecallPortabilityClass, RuntimeProjectionApi,
    RuntimeRecoveryState, RuntimeSecondaryInputAttachmentPolicy,
    RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputFallbackOutcome,
    RuntimeSecondaryInputTargetKind, RuntimeSupervisorApi, SafeModeRequest, SignalRuntime,
};

#[path = "public_host_edge_boundary/fixtures.rs"]
mod fixtures;
#[path = "public_host_edge_boundary/graphs.rs"]
mod graphs;

pub(crate) use fixtures::{
    record_public_plugin_sandbox_ready,
};
pub(crate) use graphs::{
    apply_public_capture_graph, apply_public_render_graph, apply_public_sidechain_graph,
};

#[path = "public_host_edge_boundary/consumable_surface.rs"]
mod consumable_surface;
#[path = "public_host_edge_boundary/deferred_work.rs"]
mod deferred_work;
#[path = "public_host_edge_boundary/performance_truth.rs"]
mod performance_truth;
#[path = "public_host_edge_boundary/sidechain_truth.rs"]
mod sidechain_truth;
