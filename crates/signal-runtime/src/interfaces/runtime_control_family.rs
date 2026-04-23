use super::*;

/// Runtime control-plane summary.
///
/// Callers typically pair this with `RuntimeReadiness` and
/// `EffectiveRuntimeConfig` to decide whether the runtime has been handshaken,
/// configured, started, or restarted and which control request most recently
/// changed that state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeControlSnapshot {
    /// Whether the runtime has completed the handshake sequence.
    pub handshaken: bool,
    /// Whether the runtime has been configured with a valid config.
    pub configured: bool,
    /// Whether the runtime is currently running (processing audio).
    pub running: bool,
    /// Total number of handshake operations completed.
    pub handshake_count: u64,
    /// Total number of configure operations completed.
    pub configure_count: u64,
    /// Total number of start operations completed.
    pub start_count: u64,
    /// Total number of stop operations completed.
    pub stop_count: u64,
    /// Total number of restart operations completed.
    pub restart_count: u64,
    /// Client version string from the most recent handshake, if any.
    pub last_client_version: Option<String>,
    /// Reason for the most recent stop, if any.
    pub last_stop_reason: Option<StopReason>,
    /// Most recent runtime reconfiguration request, if any.
    pub last_reconfigure: Option<RuntimeConfigRequest>,
}

/// High-level state of the runtime scheduler.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeSchedulerState {
    #[default]
    /// Scheduler is stopped; no audio processing is occurring.
    Stopped,
    /// Scheduler has been configured but is not yet running.
    Configured,
    /// Scheduler is running in idle mode (no active transport).
    ReadyIdle,
    /// Scheduler is running in realtime-only mode.
    RealtimeOnly,
    /// Scheduler is running in anticipative (prework) mode.
    Anticipative,
    /// Scheduler is running in a degraded state.
    Degraded,
}

/// Current execution phase within the scheduler.
///
/// Progresses `Idle → Priming → Prework → Realtime`; falls back to `Degraded`
/// when sandbox health is compromised.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RuntimeExecutionPhase {
    #[default]
    /// No execution is occurring.
    Idle,
    /// Execution is in the priming phase (initial plugin state setup).
    Priming,
    /// Execution is in the prework phase (anticipative pre-rendering).
    Prework,
    /// Execution is in the realtime phase (live audio processing).
    Realtime,
    /// Execution is degraded due to sandbox health issues.
    Degraded,
}

/// Point-in-time snapshot of the graph scheduler.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSchedulerSnapshot {
    /// High-level state of the scheduler.
    pub state: RuntimeSchedulerState,
    /// Current execution phase.
    pub phase: RuntimeExecutionPhase,
    /// Whether a graph has been applied to the scheduler.
    pub graph_applied: bool,
    /// Whether a schedule has been applied to the scheduler.
    pub schedule_applied: bool,
    /// Whether a transport projection has been applied.
    pub transport_projected: bool,
    /// Whether anticipative (prework) scheduling is enabled.
    pub anticipative_enabled: bool,
    /// Identifier of the currently active graph, if any.
    pub active_graph_id: Option<String>,
    /// Number of scheduling phases in the current plan.
    pub phase_count: usize,
    /// Number of dispatch lanes in the current plan.
    pub lane_count: usize,
    /// Total number of dispatches completed.
    pub dispatch_count: usize,
    /// Number of pending prework targets.
    pub pending_prework_target_count: usize,
    /// Total number of realtime blocks processed.
    pub processed_block_count: u64,
}

/// Overall readiness state reported by the runtime.
///
/// Callers should gate audio processing on `Ready`; anything else signals
/// that the runtime is transitioning, degraded, or terminated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuntimeReadiness {
    /// Runtime is in the process of starting.
    Starting,
    /// Runtime is healthy and processing audio.
    Ready,
    /// Runtime is running but in a degraded state (sandbox faults, xruns,
    /// etc.).
    Degraded {
        /// List of reasons the runtime is currently degraded.
        reasons: Vec<DegradedReason>,
    },
    /// Runtime is stopped (not processing audio).
    Stopped,
    /// Runtime encountered a fatal error and cannot continue.
    Failed {
        /// The fatal error that caused the runtime to fail.
        fatal: RuntimeError,
    },
}

/// Resolved configuration values currently active in the runtime.
///
/// Derived from the last successful `configure()` call and updated on
/// reconfiguration.  Use this (not `RuntimeConfig`) to inspect the live
/// operating parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EffectiveRuntimeConfig {
    /// Sample rate at which audio is being processed.
    pub sample_rate: SampleRate,
    /// Block size in frames used for audio processing.
    pub block_size: usize,
    /// Whether anticipative (prework) scheduling is enabled.
    pub anticipative_enabled: bool,
    /// Whether safe mode is enabled.
    pub safe_mode_enabled: bool,
    /// Name of the active output device, if one is configured.
    pub active_output_device: Option<String>,
}

/// Quick-access scalar diagnostics: CPU load, xruns, plugin sandbox counts,
/// and output levels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RuntimeDiagnosticsSnapshot {
    /// Current CPU load as a percentage.
    pub cpu_load_percent: f32,
    /// Cumulative xrun count.
    pub xruns: u64,
    /// Current graph latency in milliseconds.
    pub graph_latency_ms: f32,
    /// Number of currently active plugin sandboxes.
    pub active_plugin_sandboxes: u32,
    /// Active backend policy tier.
    pub backend_policy_tier: BackendPolicyTier,
    /// Whether the current scheduler topology is compatible.
    pub topology_compatible: bool,
    /// Number of topology compatibility issues.
    pub topology_issue_count: usize,
    /// Number of bound plugin sandboxes in a degraded state.
    pub degraded_bound_plugin_sandboxes: usize,
    /// Number of plugin chain stages with a missing sandbox binding.
    pub missing_bound_plugin_sandboxes: usize,
    /// Peak output level from the most recent block.
    pub last_output_peak: Option<f32>,
    /// RMS output level from the most recent block.
    pub last_output_rms: Option<f32>,
    /// Momentary loudness in LUFS (400 ms window).
    pub momentary_loudness_lufs: Option<f32>,
    /// Short-term loudness in LUFS (3 s window).
    pub short_term_loudness_lufs: Option<f32>,
    /// Integrated loudness in LUFS since last reset.
    pub integrated_loudness_lufs: Option<f32>,
}

/// Primary cause of the current fault condition.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultCause {
    /// Fault caused by an excessive xrun rate.
    XrunOverload,
    /// Fault caused by a plugin sandbox failure.
    PluginFault,
    /// Fault triggered by the watchdog restart mechanism.
    WatchdogRestart,
    /// Fault caused by an audio device loss.
    DeviceLoss,
    /// Fault caused by a transport session failure.
    TransportFault,
    /// Fault caused by a missing plugin sandbox binding.
    MissingPluginBinding,
    /// Fault caused by an internal runtime error.
    RuntimeError,
}

/// Category of a fault diagnostic contribution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticFamily {
    /// Fault contributions from xrun pressure.
    XrunPressure,
    /// Fault contributions from audio callback pressure.
    CallbackPressure,
    /// Fault contributions from plugin boundary failures.
    PluginBoundaryFault,
    /// Fault contributions from device path failures.
    DevicePathFault,
    /// Fault contributions from deferred-work pressure.
    DeferredWorkPressure,
}

/// Authority reporting a fault diagnostic (runtime canonical vs. host
/// advisory).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuntimeFaultDiagnosticAuthority {
    /// Fault diagnostic is the canonical runtime assessment.
    RuntimeCanonical,
    /// Fault diagnostic is an advisory from the host.
    HostAdvisory,
}

/// A single categorised fault contribution entry in a
/// [`RuntimeFaultDiagnosticReceipt`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeFaultContributionReceipt {
    /// Diagnostic family category.
    pub family: RuntimeFaultDiagnosticFamily,
    /// Authority that reported this contribution.
    pub authority: RuntimeFaultDiagnosticAuthority,
    /// Whether this fault contribution is currently active.
    pub active: bool,
    /// Number of events counted for this contribution.
    pub event_count: u64,
    /// Optional detail message for this contribution.
    pub detail: Option<String>,
    /// Human-readable one-line summary.
    pub summary: String,
}
