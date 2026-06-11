use signal_hardware::BackendPolicyTier;
use signal_runtime::{
    RecoveryRestartIntent, RuntimeHostClockDomain, RuntimeHostClockFallbackState, StopReason,
};

pub(crate) const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
pub(crate) const STEADY_STATE_BLOCKS: u64 = 8;
#[cfg(test)]
pub(crate) const SOAK_RESTART_EPISODES: u32 = 3;
pub(crate) const INTER_EPISODE_CONTINUITY_BLOCKS: u64 = 2;
pub(crate) const LOCAL_DEMO_GRAPH_ID: &str = "signal.host.local.demo";
pub(crate) const LOCAL_DEMO_PLUGIN_NODE_ID: &str = "plugin-insert";
pub(crate) const LOCAL_DEMO_PLUGIN_LATENCY_SAMPLES: u32 = 24;
pub(crate) const LOCAL_DEMO_PLUGIN_TAIL_SAMPLES: u32 = 48;

#[derive(Clone, Debug, Default)]
pub(crate) struct LocalSupervisorState {
    pub(crate) scans_started: u64,
    pub(crate) sandboxes: u64,
    pub(crate) restarts: u64,
    pub(crate) teardowns: u64,
    pub(crate) backend_policy: Option<BackendPolicyTier>,
    pub(crate) last_scan_roots: Vec<String>,
    pub(crate) last_sandbox_id: Option<String>,
    pub(crate) last_recovery_intent: Option<RecoveryRestartIntent>,
    pub(crate) last_stop_reason: Option<StopReason>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct LocalClockTransitionMemory {
    pub(crate) configured_stream: bool,
    pub(crate) domain: RuntimeHostClockDomain,
    pub(crate) fallback_state: RuntimeHostClockFallbackState,
    pub(crate) initialized: bool,
}

impl Default for LocalClockTransitionMemory {
    fn default() -> Self {
        Self {
            configured_stream: false,
            domain: RuntimeHostClockDomain::SameClock,
            fallback_state: RuntimeHostClockFallbackState::Unconfigured,
            initialized: false,
        }
    }
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaultInjection {
    Timeout,
    Crash,
    HeartbeatMiss,
    DeviceLoss,
    DeviceLossRestartFailure,
    RecoveryDeferredTeardownFailure,
    RecoveryDeferredTeardownThenCleanup,
    RecoveryDeferredTeardownCleanupRetry,
    RecoveryTeardownFailure,
    RecoveryRestartFailure,
    RecoveryOverlapContention,
    RecoveryInterleavedFailures,
    EscalatingHeartbeatMisses { restart_episodes: u32 },
    MixedWatchdogEpisodes { restart_episodes: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum RecoveryFailureInjection {
    OldTransportTeardown,
    DeferredOldTransportTeardown,
    LingeringCleanupTeardown,
    ReplacementStart,
    CompetingOverlapAttach,
}

/// Plan for exercising retrying timeout recovery against the local host.
///
/// Describes a sequence of injected failures and whether to attempt a clean
/// recovery after all failures have been exhausted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TimeoutRecoveryRetryPlan<'a> {
    /// Sequence of failure modes to inject, applied in order.
    pub(crate) failures: &'a [RecoveryFailureInjection],
    /// Detail message used when the terminal failure is reached.
    pub(crate) terminal_detail: &'a str,
    /// Whether to attempt a clean recovery after all failures have been exhausted.
    pub(crate) recover_after_failures: bool,
}

/// Plan for exercising repeated watchdog recovery cycles against the local host.
///
/// `restart_episodes` controls how many watchdog-trigger -> recover cycles to
/// run; `mixed_faults` alternates between heartbeat-miss and timeout faults.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RepeatedWatchdogRecoveryPlan {
    /// Number of watchdog-trigger -> recover cycles to run.
    pub(crate) restart_episodes: u32,
    /// Whether to alternate between heartbeat-miss and timeout faults across episodes.
    pub(crate) mixed_faults: bool,
}
