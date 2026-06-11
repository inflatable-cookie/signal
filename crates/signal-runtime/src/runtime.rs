//! Runtime configuration and shell implementation for Signal.
#[path = "runtime_api.rs"]
mod runtime_api;
#[path = "runtime_audio_file_io.rs"]
mod runtime_audio_file_io;
#[path = "runtime_contract.rs"]
mod runtime_contract;
#[path = "runtime_event_recording.rs"]
mod runtime_event_recording;
#[path = "runtime_event_surface.rs"]
mod runtime_event_surface;
#[path = "runtime_execution_plan.rs"]
mod runtime_execution_plan;
#[path = "runtime_graph_projection.rs"]
mod runtime_graph_projection;
#[path = "runtime_lifecycle_state.rs"]
mod runtime_lifecycle_state;
#[path = "runtime_media_processing.rs"]
mod runtime_media_processing;
#[path = "runtime_media_services.rs"]
mod runtime_media_services;
#[path = "runtime_media_state.rs"]
mod runtime_media_state;
#[path = "runtime_observation_surface.rs"]
mod runtime_observation_surface;
#[path = "runtime_plugin_lifecycle.rs"]
mod runtime_plugin_lifecycle;
#[path = "runtime_plugin_recording.rs"]
mod runtime_plugin_recording;
#[path = "runtime_projection_guards.rs"]
mod runtime_projection_guards;
#[path = "runtime_recording_capture.rs"]
mod runtime_recording_capture;
#[path = "runtime_shell.rs"]
mod runtime_shell;
#[path = "runtime_supervision_state.rs"]
mod runtime_supervision_state;
#[path = "runtime_support_models.rs"]
mod runtime_support_models;
#[path = "runtime_tempo_warp_state.rs"]
mod runtime_tempo_warp_state;
#[path = "runtime_utils.rs"]
pub(crate) mod runtime_utils;

use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fs,
    path::{Path, PathBuf},
};

#[allow(clippy::wildcard_imports)]
use crate::interfaces::*;
pub use runtime_contract::RuntimeConfig;
use runtime_execution_plan::RuntimeExecutionPlanState;
use runtime_media_processing::analyze_runtime_media_asset;
pub(crate) use runtime_media_state::{
    RuntimeClipProcessingPipelineStateModel, RuntimeMediaPipelineStateModel,
};
use runtime_plugin_lifecycle::{
    runtime_plugin_boundary_counts, runtime_plugin_stage_assignment,
    RuntimePluginLifecycleStateModel,
};
use runtime_plugin_recording::{
    runtime_plugin_capability_coverage, runtime_plugin_format_coverage,
};
use runtime_recording_capture::RuntimeRecordingCaptureStateModel;
pub(crate) use runtime_support_models::{
    runtime_plugin_chain_id, runtime_plugin_discovered_type_for_recall,
    RuntimeMediaAnalysisStateModel, RuntimeMediaPipelineAsset, RuntimeMediaPipelinePolicy,
    RuntimePluginCompensationObservation, RuntimeRecordingCapturePolicy,
};
use runtime_support_models::{RuntimePluginDiscoveryStateModel, RuntimeSupervisionState};
pub(crate) use runtime_tempo_warp_state::{
    RuntimeResolvedTempo, RuntimeTempoMapStateModel, RuntimeWarpPipelineStateModel,
};
use signal_graph::{
    ExecutableGraph, GraphConfig, GraphNodeBufferContract, GraphNodeExecutionClass, GraphNodeSpec,
    GraphNodeTopologyMetadata, GraphNodeTopologyRole,
};
use signal_hardware::{BackendPolicyTier, HardwareConfigRequest};
use signal_plugin::{PluginFeature, PluginFormat};
use signal_primitives::SampleRate;

/// Control-plane and observation-plane for Signal runtime supervision.
///
/// `SignalRuntime` owns the graph plan vocabulary, plugin sandbox lifecycle
/// records, media pipeline, and recording capture. Production audio execution
/// lives in `signal-render-plane`; this type is the thin control library the
/// local host and pulse observe through.
///
/// Construct with [`SignalRuntime::new`], then drive through the lifecycle
/// with the [`RuntimeLifecycleApi`] trait methods (`handshake → configure →
/// start → … → stop`). Use [`RuntimeProjectionApi`] to apply the graph plan,
/// and [`RuntimeSupervisorApi`](crate::RuntimeSupervisorApi) to manage plugin
/// sandboxes. Read state without mutating via [`RuntimeObservationApi`].
pub struct SignalRuntime {
    config: RuntimeConfig,
    readiness: RuntimeReadiness,
    safe_mode_enabled: bool,
    anticipative_enabled: bool,
    active_output_device: Option<String>,
    projection_epoch: u64,
    control: RuntimeControlSnapshot,
    plan: RuntimeExecutionPlanState,
    plugin_discovery: RuntimePluginDiscoveryStateModel,
    plugin_lifecycle: RuntimePluginLifecycleStateModel,
    plugin_placement_policy: RuntimePluginPlacementPolicy,
    recording_capture: RuntimeRecordingCaptureStateModel,
    media_pipeline: RuntimeMediaPipelineStateModel,
    tempo_map: RuntimeTempoMapStateModel,
    warp_pipeline: RuntimeWarpPipelineStateModel,
    clip_processing_pipeline: RuntimeClipProcessingPipelineStateModel,
    diagnostics: RuntimeDiagnosticsSnapshot,
    supervision: RuntimeSupervisionState,
    next_subscription: u64,
    sinks: Vec<Box<dyn RuntimeEventSink>>,
}

impl SignalRuntime {
    /// Creates a new `SignalRuntime` instance with the given boot-time configuration.
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            readiness: RuntimeReadiness::Stopped,
            safe_mode_enabled: false,
            anticipative_enabled: true,
            active_output_device: None,
            projection_epoch: 0,
            control: RuntimeControlSnapshot::default(),
            plan: RuntimeExecutionPlanState::default(),
            plugin_discovery: RuntimePluginDiscoveryStateModel::default(),
            plugin_lifecycle: RuntimePluginLifecycleStateModel::default(),
            plugin_placement_policy: RuntimePluginPlacementPolicy::default(),
            recording_capture: RuntimeRecordingCaptureStateModel::default(),
            media_pipeline: RuntimeMediaPipelineStateModel::default(),
            tempo_map: RuntimeTempoMapStateModel::default(),
            warp_pipeline: RuntimeWarpPipelineStateModel::default(),
            clip_processing_pipeline: RuntimeClipProcessingPipelineStateModel::default(),
            diagnostics: RuntimeDiagnosticsSnapshot {
                cpu_load_percent: 0.0,
                xruns: 0,
                graph_latency_ms: 0.0,
                active_plugin_sandboxes: 0,
                backend_policy_tier: BackendPolicyTier::Tier0InHost,
            },
            supervision: RuntimeSupervisionState::default(),
            next_subscription: 1,
            sinks: Vec::new(),
        }
    }

    /// Returns the boot-time configuration for this runtime instance.
    pub fn config(&self) -> RuntimeConfig {
        self.config
    }

    /// Sets the active output device ID and emits a hardware device changed event.
    pub fn set_active_output_device(&mut self, device_id: impl Into<String>) {
        self.active_output_device = Some(device_id.into());
        self.emit(RuntimeEvent::HardwareDeviceChanged {
            device_id: self.active_output_device.clone(),
        });
    }

    /// Updates the active plugin sandbox count.
    pub fn set_active_plugin_sandboxes(&mut self, count: u32) {
        self.diagnostics.active_plugin_sandboxes = count;
        self.plugin_lifecycle.set_active_sandbox_count(count);
        self.emit(RuntimeEvent::PluginSandboxChanged {
            active_sandboxes: self.diagnostics.active_plugin_sandboxes,
        });
    }

    /// Sets the backend policy tier reported in diagnostics.
    pub fn set_backend_policy_tier(&mut self, tier: BackendPolicyTier) {
        self.diagnostics.backend_policy_tier = tier;
    }
}
