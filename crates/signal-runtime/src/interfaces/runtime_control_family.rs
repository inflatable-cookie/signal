use super::*;

/// Runtime control-plane summary.
///
/// Callers typically pair this with `RuntimeReadiness` and
/// `EffectiveRuntimeConfig` to decide whether the runtime has been handshaken,
/// configured, started, or restarted and which control request most recently
/// changed that state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeControlSnapshot {
    pub handshaken: bool,
    pub configured: bool,
    pub running: bool,
    pub handshake_count: u64,
    pub configure_count: u64,
    pub start_count: u64,
    pub stop_count: u64,
    pub restart_count: u64,
    pub last_client_version: Option<String>,
    pub last_stop_reason: Option<StopReason>,
    pub last_reconfigure: Option<RuntimeConfigRequest>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSchedulerState {
    #[default]
    Stopped,
    Configured,
    ReadyIdle,
    RealtimeOnly,
    Anticipative,
    Degraded,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeExecutionPhase {
    #[default]
    Idle,
    Priming,
    Prework,
    Realtime,
    Degraded,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSchedulerSnapshot {
    pub state: RuntimeSchedulerState,
    pub phase: RuntimeExecutionPhase,
    pub graph_applied: bool,
    pub schedule_applied: bool,
    pub transport_projected: bool,
    pub anticipative_enabled: bool,
    pub active_graph_id: Option<String>,
    pub phase_count: usize,
    pub lane_count: usize,
    pub dispatch_count: usize,
    pub pending_prework_target_count: usize,
    pub processed_block_count: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeReadiness {
    Starting,
    Ready,
    Degraded { reasons: Vec<DegradedReason> },
    Stopped,
    Failed { fatal: RuntimeError },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveRuntimeConfig {
    pub sample_rate: SampleRate,
    pub block_size: usize,
    pub anticipative_enabled: bool,
    pub safe_mode_enabled: bool,
    pub active_output_device: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    pub cpu_load_percent: f32,
    pub xruns: u64,
    pub graph_latency_ms: f32,
    pub active_plugin_sandboxes: u32,
    pub backend_policy_tier: BackendPolicyTier,
    pub topology_compatible: bool,
    pub topology_issue_count: usize,
    pub degraded_bound_plugin_sandboxes: usize,
    pub missing_bound_plugin_sandboxes: usize,
    pub last_output_peak: Option<f32>,
    pub last_output_rms: Option<f32>,
    pub momentary_loudness_lufs: Option<f32>,
    pub short_term_loudness_lufs: Option<f32>,
    pub integrated_loudness_lufs: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultCause {
    XrunOverload,
    PluginFault,
    WatchdogRestart,
    DeviceLoss,
    TransportFault,
    MissingPluginBinding,
    RuntimeError,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticFamily {
    XrunPressure,
    CallbackPressure,
    PluginBoundaryFault,
    DevicePathFault,
    DeferredWorkPressure,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticAuthority {
    RuntimeCanonical,
    HostAdvisory,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultContributionReceipt {
    pub family: RuntimeFaultDiagnosticFamily,
    pub authority: RuntimeFaultDiagnosticAuthority,
    pub active: bool,
    pub event_count: u64,
    pub detail: Option<String>,
    pub summary: String,
}
