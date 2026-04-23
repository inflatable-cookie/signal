/// Severity level attached to a hardware diagnostic event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareDiagnosticSeverity {
    /// Informational — no action required.
    Info,
    /// Something is degraded but the stream may still be running.
    Warning,
    /// A critical failure has occurred; the stream is likely unusable.
    Critical,
}

/// Category of a hardware diagnostic event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HardwareDiagnosticKind {
    /// Buffer underrun or overrun (xrun) occurred in the audio callback.
    Xrun,
    /// The audio callback ran longer than its allotted time window.
    CallbackOverrun,
    /// A device was disconnected or became unavailable.
    DeviceDisconnected,
    /// The backend attempted to restart the stream.
    RestartAttempted,
    /// A restart attempt failed.
    RestartFailed,
}

/// Coarse health state of a backend, derived from accumulated diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackendHealth {
    /// Backend is operating normally.
    Healthy,
    /// Backend has reported faults but is still partially operational.
    Degraded,
    /// Backend has detected a fault and is actively attempting recovery.
    Recovering,
}

/// A single recorded hardware diagnostic event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareDiagnosticEvent {
    /// Category of the event.
    pub kind: HardwareDiagnosticKind,
    /// Severity of the event.
    pub severity: HardwareDiagnosticSeverity,
    /// Device ID associated with the event, if applicable.
    pub device_id: Option<String>,
    /// Audio callback index at the time of the event, if known.
    pub callback_index: Option<u64>,
    /// Human-readable detail string.
    pub detail: String,
}

/// Point-in-time snapshot of accumulated hardware diagnostic counters.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HardwareDiagnosticsSnapshot {
    /// Overall health state at the time of the snapshot.
    pub health: BackendHealth,
    /// Total number of buffer xruns since the backend was initialized.
    pub xrun_count: u64,
    /// Total number of callback overruns since initialization.
    pub callback_overrun_count: u64,
    /// Total number of device-loss events since initialization.
    pub device_loss_count: u64,
    /// Total number of restart attempts since initialization.
    pub restart_attempt_count: u64,
    /// Total number of restart failures since initialization.
    pub restart_failure_count: u64,
    /// The most recent event, if any have been recorded.
    pub last_event: Option<HardwareDiagnosticEvent>,
}

impl HardwareDiagnosticsSnapshot {
    /// Construct a fully-zeroed snapshot with [`BackendHealth::Healthy`].
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
