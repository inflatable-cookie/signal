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
    /// Reported CPU load percentage.
    pub cpu_load_percent: f32,
    /// Total xrun count since startup.
    pub xruns: u64,
    /// Reported graph latency in milliseconds.
    pub graph_latency_ms: f32,
    /// Number of active plugin sandboxes.
    pub active_plugin_sandboxes: u32,
    /// Backend policy tier in effect.
    pub backend_policy_tier: BackendPolicyTier,
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
