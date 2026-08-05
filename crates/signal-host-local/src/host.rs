use std::{cell::RefCell, collections::HashMap};

use signal_hardware::HardwareStreamConfig;
use signal_plugin::PluginFormat;
use signal_plugin_au::AuHostAdapter;
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_vst3::Vst3HostAdapter;
use signal_render_plane::{
    OfflineStretchArtifactCacheDecision as RenderOfflineStretchArtifactCacheDecision,
    OfflineStretchArtifactCacheDecisionKind as RenderOfflineStretchArtifactCacheDecisionKind,
    OfflineStretchArtifactMaterializationReceipt as RenderOfflineStretchArtifactMaterializationReceipt,
    OfflineStretchArtifactScope as RenderOfflineStretchArtifactScope,
};
use signal_runtime::{
    BackendPolicyOverride, PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest,
    RuntimeClipProcessingRegistration, RuntimeError, RuntimeEventRecorder,
    RuntimeHostSupervisorReport, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeOfflineStretchArtifactCacheDecisionKind,
    RuntimeOfflineStretchArtifactCacheDecisionRegistration,
    RuntimeOfflineStretchArtifactMaterializationRegistration,
    RuntimeOfflineStretchArtifactPlanRegistration, RuntimeRecordingCaptureCommitReceipt,
    RuntimeRecordingCaptureStartRequest, RuntimeSupervisorApi, RuntimeWarpClipRegistration,
    SignalRuntime,
};

#[path = "host_api.rs"]
mod host_api;
#[path = "host_support.rs"]
mod host_support;
#[cfg(test)]
pub(crate) use host_support::demo_plugin_env_lock;
use host_support::{
    discovered_plugins_for_scan, ensure_discovered_sandbox_session,
    runtime_plugin_format_platform_coverage, teardown_broker_sandbox_session,
    LocalClockTransitionMemory, LocalHardwareBackend, LocalSupervisorState, SandboxBrokerSession,
};
pub use host_support::{
    ensure_default_demo_plugin_override, LocalAudioPumpSummary, LocalAudioStreamState,
    LocalHardwareSummary, LocalRuntimeHostSummary,
};
pub(crate) use host_support::{LOCAL_DEMO_GRAPH_ID, LOCAL_DEMO_PLUGIN_NODE_ID};

/// The local desktop runtime host.
///
/// Owns the [`SignalRuntime`], the local hardware backend, CLAP/AU/VST3 plugin
/// discovery adapters, and the audio pump. Construct with
/// [`LocalRuntimeHost::new`] and drive via the [`RuntimeSupervisorApi`] and
/// [`RuntimeObservationApi`] traits implemented in `host_api.rs`.
///
/// Plugin discovery only ever scans roots passed explicitly through
/// [`RuntimeSupervisorApi::start_plugin_scan`]; no system plugin directory is
/// touched by default and no plugin is instantiated in this process.
pub struct LocalRuntimeHost {
    runtime: SignalRuntime,
    hardware: LocalHardwareBackend,
    clap: ClapPluginHostAdapter,
    au: AuHostAdapter,
    vst3: Vst3HostAdapter,
    discovered_clap_types: HashMap<String, signal_plugin_clap::ClapDiscoveredPluginType>,
    discovered_au_types: HashMap<String, signal_plugin_au::AuDiscoveredPluginType>,
    discovered_vst3_types: HashMap<String, signal_plugin_vst3::Vst3DiscoveredPluginType>,
    active_sandbox_specs: HashMap<String, PluginSandboxSpec>,
    sandbox_broker_sessions: HashMap<String, SandboxBrokerSession>,
    active_output_stream: Option<HardwareStreamConfig>,
    clock_transition_memory: RefCell<LocalClockTransitionMemory>,
    stream_state: LocalAudioStreamState,
    supervisor: LocalSupervisorState,
    events: RuntimeEventRecorder,
}

impl LocalRuntimeHost {
    /// Construct a new local host wrapping the given runtime.
    ///
    /// Initialises the hardware backend and the plugin format discovery
    /// adapters. The runtime is subscribed to an internal event recorder
    /// immediately.
    pub fn new(runtime: SignalRuntime) -> Self {
        let events = RuntimeEventRecorder::default();
        let mut runtime = runtime;
        runtime.subscribe(Box::new(events.clone()));
        runtime.record_plugin_format_platform_coverage(runtime_plugin_format_platform_coverage());

        Self {
            runtime,
            hardware: LocalHardwareBackend::default(),
            clap: ClapPluginHostAdapter::default(),
            au: AuHostAdapter::default(),
            vst3: Vst3HostAdapter::default(),
            discovered_clap_types: HashMap::new(),
            discovered_au_types: HashMap::new(),
            discovered_vst3_types: HashMap::new(),
            active_sandbox_specs: HashMap::new(),
            sandbox_broker_sessions: HashMap::new(),
            active_output_stream: None,
            clock_transition_memory: RefCell::new(LocalClockTransitionMemory::default()),
            stream_state: LocalAudioStreamState::Stopped,
            supervisor: LocalSupervisorState::default(),
            events,
        }
    }

    /// Returns a reference to the underlying runtime.
    pub fn runtime(&self) -> &SignalRuntime {
        &self.runtime
    }

    /// Returns a supervisor report combining the current runtime state with
    /// host I/O metrics.
    pub fn host_supervisor_report(&self) -> RuntimeHostSupervisorReport {
        let (supervisor, host_io) = self.supervisor_with_host_io();
        RuntimeHostSupervisorReport::new(supervisor, host_io)
    }

    /// Records a render-plane materialization receipt on the runtime observation surface.
    ///
    /// Render/export/freeze callers produce [`RenderOfflineStretchArtifactMaterializationReceipt`]
    /// when they materialize an OfflineHighQuality stretch artifact. The local
    /// host owns the crate boundary that can translate that render-plane receipt
    /// into runtime-owned observation without making `signal-runtime` depend on
    /// `signal-render-plane`.
    pub fn record_offline_stretch_artifact_materialization_receipt(
        &mut self,
        artifact_id: impl Into<String>,
        plan_id: impl Into<String>,
        clip_id: Option<String>,
        media_asset_id: Option<String>,
        receipt: RenderOfflineStretchArtifactMaterializationReceipt,
    ) -> Result<(), RuntimeError> {
        self.runtime
            .reconcile_offline_stretch_artifact_materializations(vec![
                RuntimeOfflineStretchArtifactMaterializationRegistration {
                    artifact_id: artifact_id.into(),
                    plan_id: plan_id.into(),
                    clip_id,
                    media_asset_id,
                    scope: runtime_offline_stretch_artifact_scope(receipt.scope),
                    tier: receipt.tier,
                    offline_path: receipt.offline_path,
                    cache_identity_hash: receipt.cache_identity_hash,
                    cache_identity_key: receipt.cache_identity_key,
                    promotion_evidence_id: receipt.promotion_evidence_id,
                    input_frame_count: receipt.input_frame_count,
                    output_frame_count: receipt.output_frame_count,
                    channels: receipt.channels,
                    sample_rate_hz: receipt.sample_rate_hz,
                    chunk_count: receipt.chunk_count,
                    max_chunk_source_frames: receipt.max_chunk_source_frames,
                    chunk_overlap_frames: receipt.chunk_overlap_frames,
                    max_chunk_render_source_frames: receipt.max_chunk_render_source_frames,
                    product_facing_allowed: receipt.product_facing_allowed,
                },
            ])
    }

    /// Records a render-cache decision on the runtime observation surface.
    ///
    /// Render-cache callers produce [`RenderOfflineStretchArtifactCacheDecision`]
    /// when they resolve an OfflineHighQuality artifact through Signal's
    /// render-cache bridge. The local host translates that render-plane receipt
    /// into runtime-owned observation without adding a runtime dependency on
    /// `signal-render-plane`.
    pub fn record_offline_stretch_artifact_cache_decision_receipt(
        &mut self,
        decision_id: impl Into<String>,
        plan_id: impl Into<String>,
        clip_id: Option<String>,
        media_asset_id: Option<String>,
        decision: RenderOfflineStretchArtifactCacheDecision,
    ) -> Result<(), RuntimeError> {
        let receipt = decision.handoff.receipt;
        self.runtime
            .reconcile_offline_stretch_artifact_cache_decisions(vec![
                RuntimeOfflineStretchArtifactCacheDecisionRegistration {
                    decision_id: decision_id.into(),
                    plan_id: plan_id.into(),
                    clip_id,
                    media_asset_id,
                    scope: runtime_offline_stretch_artifact_scope(receipt.scope),
                    kind: runtime_offline_stretch_artifact_cache_decision_kind(decision.kind),
                    tier: receipt.tier,
                    offline_path: receipt.offline_path,
                    cache_identity_hash: receipt.cache_identity_hash,
                    cache_identity_key: receipt.cache_identity_key,
                    promotion_evidence_id: receipt.promotion_evidence_id,
                    output_frame_count: receipt.output_frame_count,
                    chunk_count: receipt.chunk_count,
                    max_chunk_source_frames: receipt.max_chunk_source_frames,
                    chunk_overlap_frames: receipt.chunk_overlap_frames,
                    max_chunk_render_source_frames: receipt.max_chunk_render_source_frames,
                    product_facing_allowed: receipt.product_facing_allowed,
                },
            ])
    }
}

fn runtime_offline_stretch_artifact_scope(
    scope: RenderOfflineStretchArtifactScope,
) -> signal_runtime::RuntimeOfflineStretchArtifactScope {
    match scope {
        RenderOfflineStretchArtifactScope::Export => {
            signal_runtime::RuntimeOfflineStretchArtifactScope::Export
        }
        RenderOfflineStretchArtifactScope::Freeze => {
            signal_runtime::RuntimeOfflineStretchArtifactScope::Freeze
        }
        RenderOfflineStretchArtifactScope::RenderCache => {
            signal_runtime::RuntimeOfflineStretchArtifactScope::RenderCache
        }
    }
}

fn runtime_offline_stretch_artifact_cache_decision_kind(
    kind: RenderOfflineStretchArtifactCacheDecisionKind,
) -> RuntimeOfflineStretchArtifactCacheDecisionKind {
    match kind {
        RenderOfflineStretchArtifactCacheDecisionKind::Hit => {
            RuntimeOfflineStretchArtifactCacheDecisionKind::Hit
        }
        RenderOfflineStretchArtifactCacheDecisionKind::Written => {
            RuntimeOfflineStretchArtifactCacheDecisionKind::Written
        }
        RenderOfflineStretchArtifactCacheDecisionKind::Invalidated => {
            RuntimeOfflineStretchArtifactCacheDecisionKind::Invalidated
        }
    }
}

#[cfg(test)]
mod tests {
    include!("host_tests.rs");
}
