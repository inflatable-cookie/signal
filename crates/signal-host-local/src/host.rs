use std::{cell::RefCell, collections::HashMap};

use signal_hardware::HardwareStreamConfig;
use signal_plugin::PluginFormat;
use signal_plugin_au::AuHostAdapter;
use signal_plugin_clap::ClapPluginHostAdapter;
use signal_plugin_vst3::Vst3HostAdapter;
use signal_runtime::{
    BackendPolicyOverride, PluginSandboxLifecycleStage, PluginSandboxSpec, PluginScanRequest,
    RuntimeClipProcessingRegistration, RuntimeError, RuntimeEventRecorder,
    RuntimeHostSupervisorReport, RuntimeMediaAssetRegistration, RuntimeObservationApi,
    RuntimeRecordingCaptureCommitReceipt, RuntimeRecordingCaptureStartRequest,
    RuntimeSupervisorApi, RuntimeWarpClipRegistration, SignalRuntime,
};

#[path = "host_api.rs"]
mod host_api;
#[path = "host_support.rs"]
mod host_support;
#[cfg(test)]
pub(crate) use host_support::STEADY_STATE_BLOCKS;
use host_support::{
    discovered_plugins_for_scan, ensure_discovered_sandbox_session,
    runtime_plugin_format_platform_coverage, teardown_broker_sandbox_session, LocalAudioPumpState,
    LocalClockTransitionMemory, LocalHardwareBackend, LocalSupervisorState, SandboxBrokerSession,
};
pub use host_support::{
    ensure_default_demo_plugin_override, LocalAudioPumpSummary, LocalAudioStreamState,
    LocalAudioTransferPolicy, LocalEngineSummary, LocalHardwareSummary, LocalRuntimeHostSummary,
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
    audio_pump: LocalAudioPumpState,
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
            audio_pump: LocalAudioPumpState::default(),
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
}

#[cfg(test)]
mod tests {
    include!("host_tests.rs");
}
