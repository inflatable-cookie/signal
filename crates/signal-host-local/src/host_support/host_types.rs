use signal_hardware::BackendPolicyTier;
use signal_runtime::{
    RecoveryRestartIntent, RuntimeHostClockDomain, RuntimeHostClockFallbackState, StopReason,
};

pub(crate) const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
pub(crate) const STEADY_STATE_BLOCKS: u64 = 8;
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
