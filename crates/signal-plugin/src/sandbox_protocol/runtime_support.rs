use crate::{CompletionState, PluginFault, PluginFaultKind, PluginFaultSeverity};

/// Classification of an error returned by a sandbox operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxErrorKind {
    /// The request was malformed or contained invalid parameters.
    InvalidRequest,
    /// The operation is not valid in the sandbox's current state.
    InvalidState,
    /// The sandbox does not support the requested capability.
    Unsupported,
    /// The sandbox did not respond within the allowed time.
    Timeout,
    /// The sandbox violated the expected communication protocol.
    ProtocolViolation,
    /// The sandbox process crashed.
    Crashed,
}

/// An error returned from a sandbox operation, with a kind and human-readable message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxError {
    /// Classification of the error.
    pub kind: PluginSandboxErrorKind,
    /// Human-readable description of what went wrong.
    pub message: String,
}

impl PluginSandboxError {
    /// Creates a new `PluginSandboxError` with the given kind and message.
    pub fn new(kind: PluginSandboxErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    /// Converts this error into a [`PluginFault`] with appropriate kind and severity.
    pub fn as_fault(&self) -> PluginFault {
        PluginFault::new(
            PluginFaultKind::from(self.kind),
            PluginFaultSeverity::from(self.kind),
            self.message.clone(),
        )
    }
}

impl From<PluginSandboxErrorKind> for PluginFaultKind {
    fn from(value: PluginSandboxErrorKind) -> Self {
        match value {
            PluginSandboxErrorKind::InvalidRequest => Self::InvalidRequest,
            PluginSandboxErrorKind::InvalidState => Self::InvalidState,
            PluginSandboxErrorKind::Unsupported => Self::UnsupportedCapability,
            PluginSandboxErrorKind::Timeout => Self::Timeout,
            PluginSandboxErrorKind::ProtocolViolation => Self::ProtocolViolation,
            PluginSandboxErrorKind::Crashed => Self::Crash,
        }
    }
}

impl From<PluginSandboxErrorKind> for PluginFaultSeverity {
    fn from(value: PluginSandboxErrorKind) -> Self {
        match value {
            PluginSandboxErrorKind::InvalidRequest
            | PluginSandboxErrorKind::InvalidState
            | PluginSandboxErrorKind::Unsupported => Self::Warning,
            PluginSandboxErrorKind::Timeout => Self::Recoverable,
            PluginSandboxErrorKind::ProtocolViolation => Self::Critical,
            PluginSandboxErrorKind::Crashed => Self::Fatal,
        }
    }
}

/// Thresholds that control when the watchdog declares a sandbox unhealthy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxWatchdogPolicy {
    /// Number of consecutive block deadline misses before a restart is required.
    pub max_consecutive_deadline_misses: u32,
    /// Number of consecutive missed heartbeats before a restart is required.
    pub max_consecutive_heartbeat_misses: u32,
}

impl Default for SandboxWatchdogPolicy {
    fn default() -> Self {
        Self {
            max_consecutive_deadline_misses: 2,
            max_consecutive_heartbeat_misses: 2,
        }
    }
}

/// Why the watchdog decided to trigger a sandbox restart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogTriggerReason {
    /// Too many consecutive block deadlines were missed.
    DeadlineMisses,
    /// Too many consecutive heartbeat responses were missed.
    HeartbeatMisses,
}

/// Result of a watchdog health check after recording an event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogOutcome {
    /// The sandbox is within policy limits; no action required.
    Healthy,
    /// The sandbox has exceeded a policy limit and must be restarted.
    RestartRequired {
        /// What triggered the restart decision.
        reason: WatchdogTriggerReason,
        /// How many consecutive misses were observed.
        consecutive_misses: u32,
    },
}

/// Live counters used by the watchdog to track sandbox health.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxWatchdogState {
    policy: SandboxWatchdogPolicy,
    consecutive_deadline_misses: u32,
    consecutive_heartbeat_misses: u32,
}

impl SandboxWatchdogState {
    /// Creates a new watchdog state with the given policy and all counters at zero.
    pub fn new(policy: SandboxWatchdogPolicy) -> Self {
        Self {
            policy,
            consecutive_deadline_misses: 0,
            consecutive_heartbeat_misses: 0,
        }
    }

    /// Returns the active watchdog policy.
    pub fn policy(&self) -> SandboxWatchdogPolicy {
        self.policy
    }

    /// Returns the current consecutive deadline miss count.
    pub fn consecutive_deadline_misses(&self) -> u32 {
        self.consecutive_deadline_misses
    }

    /// Returns the current consecutive heartbeat miss count.
    pub fn consecutive_heartbeat_misses(&self) -> u32 {
        self.consecutive_heartbeat_misses
    }

    /// Resets the consecutive heartbeat miss counter after a successful response.
    pub fn record_heartbeat_response(&mut self) {
        self.consecutive_heartbeat_misses = 0;
    }

    /// Increments the heartbeat miss counter and returns the watchdog outcome.
    pub fn record_heartbeat_miss(&mut self) -> WatchdogOutcome {
        self.consecutive_heartbeat_misses = self.consecutive_heartbeat_misses.saturating_add(1);
        if self.consecutive_heartbeat_misses >= self.policy.max_consecutive_heartbeat_misses {
            return WatchdogOutcome::RestartRequired {
                reason: WatchdogTriggerReason::HeartbeatMisses,
                consecutive_misses: self.consecutive_heartbeat_misses,
            };
        }
        WatchdogOutcome::Healthy
    }

    /// Updates deadline miss counters based on the given block completion state and returns the outcome.
    pub fn record_block_completion(&mut self, state: CompletionState) -> WatchdogOutcome {
        match state {
            CompletionState::TimedOut => {
                self.consecutive_deadline_misses =
                    self.consecutive_deadline_misses.saturating_add(1);
                if self.consecutive_deadline_misses >= self.policy.max_consecutive_deadline_misses {
                    return WatchdogOutcome::RestartRequired {
                        reason: WatchdogTriggerReason::DeadlineMisses,
                        consecutive_misses: self.consecutive_deadline_misses,
                    };
                }
            }
            CompletionState::Completed => {
                self.consecutive_deadline_misses = 0;
            }
            _ => {}
        }
        WatchdogOutcome::Healthy
    }

    /// Resets all miss counters to zero.
    pub fn reset(&mut self) {
        self.consecutive_deadline_misses = 0;
        self.consecutive_heartbeat_misses = 0;
    }
}

impl Default for SandboxWatchdogState {
    fn default() -> Self {
        Self::new(SandboxWatchdogPolicy::default())
    }
}

/// Policy that controls when repeated watchdog restarts escalate to safe-mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartEscalationPolicy {
    /// Number of watchdog-triggered restarts before safe-mode is requested.
    pub safe_mode_restart_threshold: u32,
}

impl Default for RestartEscalationPolicy {
    fn default() -> Self {
        Self {
            safe_mode_restart_threshold: 2,
        }
    }
}

/// Tracks watchdog restart history and determines when to escalate to safe-mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartEscalationState {
    policy: RestartEscalationPolicy,
    watchdog_restart_count: u32,
    safe_mode_requested: bool,
}

impl RestartEscalationState {
    /// Creates a new escalation state with the given policy and zero restarts.
    pub fn new(policy: RestartEscalationPolicy) -> Self {
        Self {
            policy,
            watchdog_restart_count: 0,
            safe_mode_requested: false,
        }
    }

    /// Returns the total number of watchdog-triggered restarts recorded so far.
    pub fn watchdog_restart_count(&self) -> u32 {
        self.watchdog_restart_count
    }

    /// Returns `true` if the restart threshold has been reached and safe-mode has been requested.
    pub fn safe_mode_requested(&self) -> bool {
        self.safe_mode_requested
    }

    /// Records a watchdog-triggered restart and returns `true` if safe-mode should now be entered.
    pub fn record_watchdog_restart(&mut self) -> bool {
        self.watchdog_restart_count = self.watchdog_restart_count.saturating_add(1);
        if self.watchdog_restart_count >= self.policy.safe_mode_restart_threshold {
            self.safe_mode_requested = true;
        }
        self.safe_mode_requested
    }
}

impl Default for RestartEscalationState {
    fn default() -> Self {
        Self::new(RestartEscalationPolicy::default())
    }
}

/// A closed loop region on the timeline, expressed in sample offsets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopRange {
    /// Start of the loop in samples (relative to timeline origin).
    pub start_samples: i64,
    /// End of the loop in samples (exclusive).
    pub end_samples: i64,
}

/// Transport and timing context delivered to a plugin with each processing block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PluginRenderContext {
    /// Sample rate for this processing session in Hz.
    pub sample_rate_hz: u32,
    /// Current tempo in beats per minute.
    pub tempo_bpm: f64,
    /// Timeline position at the start of this block, in samples from the session origin.
    pub timeline_position_samples: i64,
    /// `true` if the transport is currently playing.
    pub playing: bool,
    /// `true` if the plugin should pass audio through without processing.
    pub bypassed: bool,
    /// Active loop region, or `None` if looping is disabled.
    pub loop_range: Option<LoopRange>,
    /// Number of frames the plugin has to complete processing before the deadline expires.
    pub deadline_frames: u32,
}

impl PluginRenderContext {
    pub(crate) const ENCODED_BYTES: usize = 48;
}
