#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use signal_graph::{
    synthetic_stereo_block, GraphExecutionLane, GraphNodeExecutionClass, GraphNodeTopologyRole,
    GraphStageSpec,
};
use signal_host_local::LocalRuntimeHost;
use signal_plugin::{PluginFeature, PluginFormat, PluginIoLayout};
use signal_primitives::{ChannelCount, ChannelLayout, FrameCount, SampleRate};
use signal_runtime::{
    GraphContractProjection, GraphNodeBufferContractProjection, GraphNodeContractProjection,
    GraphNodeProjection, GraphNodeTopologyProjection, GraphProjection, PluginBackedNodeBinding,
    PluginBackedNodeBindingProjection, PluginSandboxLifecycleStage, PluginSandboxSpec,
    PluginSandboxTransportStage, PluginScanRequest, RuntimeBlockDeadlinePressure, RuntimeConfig,
    RuntimeConfigRequest, RuntimeDeferredServiceBackpressureSource, RuntimeDeferredServiceDecision,
    RuntimeDeferredServicePriorityBand, RuntimeDeferredServiceReason, RuntimeInterruptionClass,
    RuntimeLifecycleApi, RuntimeOfflineRenderRequest, RuntimePluginAraContextSnapshot,
    RuntimePluginAraDocumentContext, RuntimePluginAraRegionContext, RuntimePluginAraSourceContext,
    RuntimePluginComplexIoSummary, RuntimePluginDiscoveredTypeRecord,
    RuntimePluginPresetDescriptor, RuntimePluginPresetOrigin, RuntimePluginRecallPortabilityClass,
    RuntimeProjectionApi, RuntimeRecoveryState, RuntimeSecondaryInputAttachmentPolicy,
    RuntimeSecondaryInputContractProjection, RuntimeSecondaryInputFallbackOutcome,
    RuntimeSecondaryInputTargetKind, RuntimeSupervisorApi, SafeModeRequest, SignalRuntime,
};

mod fixtures;
mod graphs;

pub(crate) use fixtures::{
    public_local_media_fixture_path, record_public_plugin_sandbox_ready,
    sample_complex_bus_fx_record, sample_complex_multi_output_record, sample_host_ara_context,
    sample_host_preset_descriptor, write_public_test_wav, write_public_transient_test_wav,
};
pub(crate) use graphs::{
    apply_public_capture_graph, apply_public_complex_io_graph, apply_public_multi_bus_graph,
    apply_public_multichannel_graph, apply_public_plugin_continuity_graph,
    apply_public_render_graph, apply_public_sidechain_graph, apply_public_spatial_graph,
};

mod consumable_surface;
mod deferred_work;
mod performance_truth;
mod sidechain_truth;
