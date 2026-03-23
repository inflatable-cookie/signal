use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportAttachIntent {
    SteadyState,
    RecoveryOverlap,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionProvenance {
    SteadyOrigin,
    RecoveryReplacement,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LingeringCleanupMode {
    StrictPreAttach,
    BestEffortPostStart,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LingeringCleanupTrigger {
    RecoveryPreAttach,
    PostStartReconciliation,
    DeferredRetry,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTransportConcurrencySession {
    pub sandbox_id: String,
    pub lease_id: String,
    pub region_id: String,
    pub intent: TransportAttachIntent,
    pub provenance: TransportSessionProvenance,
    pub attach_sequence: u64,
    pub attach_processing_epoch: Option<u64>,
    pub state: TransportSessionState,
    pub backing_path: Option<String>,
    pub total_bytes: Option<u32>,
    pub cleanup_attempt_count: u32,
    pub last_cleanup_mode: Option<LingeringCleanupMode>,
    pub last_cleanup_wave: Option<u64>,
    pub cleanup_in_progress: bool,
    pub last_cleanup_epoch: Option<u64>,
    pub last_cleanup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LingeringCleanupQueueReceipt {
    pub work_id: u64,
    pub cleanup_epoch: u64,
    pub cleanup_wave: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LingeringCleanupPlan {
    pub work_id: u64,
    pub cleanup_epoch: u64,
    pub cleanup_wave: u64,
    pub sandbox_id: String,
    pub mode: LingeringCleanupMode,
    pub trigger: LingeringCleanupTrigger,
    pub retry_count: u32,
    pub processing_epoch: u64,
    pub ready_at_processing_epoch: u64,
    pub exclude_lease_id: Option<String>,
    pub exclude_region_id: Option<String>,
    pub candidates: Vec<ActiveTransportConcurrencySession>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingLingeringCleanupWaveSummary {
    pub sandbox_id: String,
    pub cleanup_wave: u64,
    pub mode: LingeringCleanupMode,
    pub first_trigger: LingeringCleanupTrigger,
    pub latest_trigger: LingeringCleanupTrigger,
    pub pending_work_items: usize,
    pub deferred_retry_work_items: usize,
    pub first_cleanup_epoch: u64,
    pub latest_cleanup_epoch: u64,
    pub first_processing_epoch: u64,
    pub latest_processing_epoch: u64,
    pub oldest_ready_at_processing_epoch: u64,
    pub newest_ready_at_processing_epoch: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTransportConcurrencySnapshot {
    pub steady_session_limit: usize,
    pub recovery_session_limit: usize,
    pub current_attached_sessions: usize,
    pub peak_attached_sessions: usize,
    pub current_recovery_overlap_sessions: usize,
    pub peak_recovery_overlap_sessions: usize,
    pub current_lingering_sessions: usize,
    pub peak_lingering_sessions: usize,
    pub current_detach_requested_sessions: usize,
    pub current_detach_faulted_sessions: usize,
    pub pending_cleanup_work_items: usize,
    pub pending_deferred_retry_work_items: usize,
    pub next_cleanup_epoch: u64,
    pub oldest_pending_cleanup_ready_epoch: Option<u64>,
    pub pending_cleanup_waves: Vec<PendingLingeringCleanupWaveSummary>,
    pub active_sessions: Vec<ActiveTransportConcurrencySession>,
    pub last_admitted_sandbox_id: Option<String>,
    pub last_rejected_sandbox_id: Option<String>,
    pub last_rejection_reason: Option<String>,
}
