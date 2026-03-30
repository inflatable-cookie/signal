use signal_hardware::BackendPolicyTier;
use signal_runtime::{RecoveryRestartIntent, StopReason};

pub(crate) const WATCHDOG_TRIGGER_WINDOW_BLOCKS: u64 = 3;
pub(crate) const STEADY_STATE_BLOCKS: u64 = 8;
pub(crate) const SOAK_RESTART_EPISODES: u32 = 3;
pub(crate) const INTER_EPISODE_CONTINUITY_BLOCKS: u64 = 2;

pub(crate) fn samples_to_ms(samples: u32, sample_rate_hz: u32) -> f32 {
    if sample_rate_hz == 0 {
        0.0
    } else {
        samples as f32 * 1_000.0 / sample_rate_hz as f32
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ServerSupervisorState {
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

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FaultInjection {
    Timeout,
    Crash,
    HeartbeatMiss,
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
