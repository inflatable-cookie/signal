use std::{cell::RefCell, collections::HashMap};

use signal_hardware::HardwareStreamConfig;
use signal_hardware_coreaudio::CoreAudioBackend;
use signal_ipc::SharedMemoryBroker;
use signal_plugin::PluginFormat;
use signal_plugin_au::AuHostAdapter;
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_vst3::Vst3HostAdapter;
use signal_runtime::{
    BackendPolicyOverride, PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest,
    RuntimeClipProcessingRegistration, RuntimeError, RuntimeEventRecorder,
    RuntimeHostObservationReport, RuntimeHostSupervisorReport, RuntimeMediaAssetRegistration,
    RuntimeObservationApi, RuntimeOfflineRenderExecutionCancellationReceipt,
    RuntimeOfflineRenderExecutionProgressReceipt, RuntimeOfflineRenderExecutionReceipt,
    RuntimeOfflineRenderPurgeReceipt, RuntimeOfflineRenderPurgeRequest,
    RuntimeOfflineRenderQueueResult, RuntimeOfflineRenderRequest, RuntimeOfflineRenderResult,
    RuntimeRecordingCaptureCommitReceipt, RuntimeRecordingCaptureStartRequest,
    RuntimeSupervisorApi, RuntimeWarpClipRegistration, SignalRuntime,
};

#[path = "host_api.rs"]
mod host_api;
#[path = "host_support.rs"]
mod host_support;
use host_support::{
    discovered_plugins_for_scan, ensure_au_sandbox_session, ensure_clap_sandbox_session,
    ensure_vst3_sandbox_session, runtime_plugin_format_platform_coverage,
    teardown_broker_sandbox_session, LifecycleRunSummary, LocalAudioPumpState,
    LocalClockTransitionMemory, LocalSupervisorState, SandboxBrokerSession,
};
pub use host_support::{
    ensure_default_demo_plugin_override, LocalAudioPumpSummary, LocalAudioStreamState,
    LocalAudioTransferPolicy, LocalExecutionSummary, LocalFaultSummary, LocalHardwareSummary,
    LocalPayloadSummary, LocalPluginDispatchSummary, LocalRuntimeHostSummary,
    LocalTransportSummary,
};
pub(crate) use host_support::{
    FaultInjection, RecoveryFailureInjection, INTER_EPISODE_CONTINUITY_BLOCKS, LOCAL_DEMO_GRAPH_ID,
    LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES, LOCAL_DEMO_PLUGIN_NODE_ID, LOCAL_DEMO_PLUGIN_TAIL_SAMPLES,
    SOAK_RESTART_EPISODES, STEADY_STATE_BLOCKS, WATCHDOG_TRIGGER_WINDOW_BLOCKS,
};

/// The local desktop runtime host.
///
/// Owns the [`SignalRuntime`], the [`CoreAudioBackend`], CLAP/AU/VST3 plugin
/// adapters, the shared-memory broker, and the audio pump. Construct with
/// [`LocalRuntimeHost::new`] and drive via the [`RuntimeSupervisorApi`] and
/// [`RuntimeObservationApi`] traits implemented in `host_api.rs`.
pub struct LocalRuntimeHost {
    runtime: SignalRuntime,
    coreaudio: CoreAudioBackend,
    clap: ClapPluginHostAdapter,
    au: AuHostAdapter,
    vst3: Vst3HostAdapter,
    discovered_clap_types: HashMap<String, signal_plugin_clap::ClapDiscoveredPluginType>,
    discovered_au_types: HashMap<String, signal_plugin_au::AuDiscoveredPluginType>,
    discovered_vst3_types: HashMap<String, signal_plugin_vst3::Vst3DiscoveredPluginType>,
    active_sandbox_specs: HashMap<String, PluginSandboxSpec>,
    sandbox_broker_sessions: HashMap<String, SandboxBrokerSession>,
    broker: SharedMemoryBroker,
    active_output_stream: Option<HardwareStreamConfig>,
    clock_transition_memory: RefCell<LocalClockTransitionMemory>,
    audio_pump: LocalAudioPumpState,
    supervisor: LocalSupervisorState,
    events: RuntimeEventRecorder,
}

impl LocalRuntimeHost {
    /// Construct a new local host wrapping the given runtime.
    ///
    /// Initialises the CoreAudio backend, all plugin format adapters, and the
    /// shared-memory broker. The runtime is subscribed to an internal event
    /// recorder immediately.
    pub fn new(runtime: SignalRuntime) -> Self {
        let events = RuntimeEventRecorder::default();
        let mut runtime = runtime;
        runtime.subscribe(Box::new(events.clone()));
        runtime.record_plugin_format_platform_coverage(runtime_plugin_format_platform_coverage());

        Self {
            runtime,
            coreaudio: CoreAudioBackend::default(),
            clap: ClapPluginHostAdapter::default(),
            au: AuHostAdapter::default(),
            vst3: Vst3HostAdapter::default(),
            discovered_clap_types: HashMap::new(),
            discovered_au_types: HashMap::new(),
            discovered_vst3_types: HashMap::new(),
            active_sandbox_specs: HashMap::new(),
            sandbox_broker_sessions: HashMap::new(),
            broker: SharedMemoryBroker::default(),
            active_output_stream: None,
            clock_transition_memory: RefCell::new(LocalClockTransitionMemory::default()),
            audio_pump: LocalAudioPumpState::default(),
            supervisor: LocalSupervisorState::default(),
            events,
        }
    }

    /// Returns a reference to the underlying runtime.
    pub fn runtime(&self) -> &SignalRuntime {
        &self.runtime
    }

    /// Returns `true` if the CLAP plugin format is supported on this platform.
    pub fn clap_supported(&self) -> bool {
        self.clap.supports_format(PluginFormat::Clap)
    }

    /// Returns an observation report combining the current runtime state with
    /// host I/O metrics (hardware, pump, transport).
    pub fn host_observation_report(&self) -> RuntimeHostObservationReport {
        let (observation, host_io) = self.observation_with_host_io();
        RuntimeHostObservationReport::new(observation, host_io)
    }

    /// Returns a supervisor report combining the current runtime state with
    /// host I/O metrics for use by recovery and watchdog logic.
    pub fn host_supervisor_report(&self) -> RuntimeHostSupervisorReport {
        let (supervisor, host_io) = self.supervisor_with_host_io();
        RuntimeHostSupervisorReport::new(supervisor, host_io)
    }
}

#[cfg(test)]
#[path = "host_test_support.rs"]
mod host_test_support;

#[cfg(test)]
mod tests {
    include!("host_tests.rs");
}
