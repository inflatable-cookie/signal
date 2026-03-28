use crate::{CompletionState, PluginFault, PluginFaultKind, PluginFaultSeverity};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PluginSandboxErrorKind {
    InvalidRequest,
    InvalidState,
    Unsupported,
    Timeout,
    ProtocolViolation,
    Crashed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PluginSandboxError {
    pub kind: PluginSandboxErrorKind,
    pub message: String,
}

impl PluginSandboxError {
    pub fn new(kind: PluginSandboxErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxWatchdogPolicy {
    pub max_consecutive_deadline_misses: u32,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogTriggerReason {
    DeadlineMisses,
    HeartbeatMisses,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchdogOutcome {
    Healthy,
    RestartRequired {
        reason: WatchdogTriggerReason,
        consecutive_misses: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SandboxWatchdogState {
    policy: SandboxWatchdogPolicy,
    consecutive_deadline_misses: u32,
    consecutive_heartbeat_misses: u32,
}

impl SandboxWatchdogState {
    pub fn new(policy: SandboxWatchdogPolicy) -> Self {
        Self {
            policy,
            consecutive_deadline_misses: 0,
            consecutive_heartbeat_misses: 0,
        }
    }

    pub fn policy(&self) -> SandboxWatchdogPolicy {
        self.policy
    }

    pub fn consecutive_deadline_misses(&self) -> u32 {
        self.consecutive_deadline_misses
    }

    pub fn consecutive_heartbeat_misses(&self) -> u32 {
        self.consecutive_heartbeat_misses
    }

    pub fn record_heartbeat_response(&mut self) {
        self.consecutive_heartbeat_misses = 0;
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartEscalationPolicy {
    pub safe_mode_restart_threshold: u32,
}

impl Default for RestartEscalationPolicy {
    fn default() -> Self {
        Self {
            safe_mode_restart_threshold: 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RestartEscalationState {
    policy: RestartEscalationPolicy,
    watchdog_restart_count: u32,
    safe_mode_requested: bool,
}

impl RestartEscalationState {
    pub fn new(policy: RestartEscalationPolicy) -> Self {
        Self {
            policy,
            watchdog_restart_count: 0,
            safe_mode_requested: false,
        }
    }

    pub fn watchdog_restart_count(&self) -> u32 {
        self.watchdog_restart_count
    }

    pub fn safe_mode_requested(&self) -> bool {
        self.safe_mode_requested
    }

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LoopRange {
    pub start_samples: i64,
    pub end_samples: i64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PluginRenderContext {
    pub sample_rate_hz: u32,
    pub tempo_bpm: f64,
    pub timeline_position_samples: i64,
    pub playing: bool,
    pub bypassed: bool,
    pub loop_range: Option<LoopRange>,
    pub deadline_frames: u32,
}

impl PluginRenderContext {
    pub(crate) const ENCODED_BYTES: usize = 48;
}
