#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareDiagnosticSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareDiagnosticKind {
    Xrun,
    CallbackOverrun,
    DeviceDisconnected,
    RestartAttempted,
    RestartFailed,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendHealth {
    Healthy,
    Degraded,
    Recovering,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareDiagnosticEvent {
    pub kind: HardwareDiagnosticKind,
    pub severity: HardwareDiagnosticSeverity,
    pub device_id: Option<String>,
    pub callback_index: Option<u64>,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareDiagnosticsSnapshot {
    pub health: BackendHealth,
    pub xrun_count: u64,
    pub callback_overrun_count: u64,
    pub device_loss_count: u64,
    pub restart_attempt_count: u64,
    pub restart_failure_count: u64,
    pub last_event: Option<HardwareDiagnosticEvent>,
}

impl HardwareDiagnosticsSnapshot {
    pub fn healthy() -> Self {
        Self {
            health: BackendHealth::Healthy,
            xrun_count: 0,
            callback_overrun_count: 0,
            device_loss_count: 0,
            restart_attempt_count: 0,
            restart_failure_count: 0,
            last_event: None,
        }
    }
}
