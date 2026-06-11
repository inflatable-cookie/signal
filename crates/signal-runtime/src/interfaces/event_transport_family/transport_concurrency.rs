use super::*;

/// Whether a transport session is a normal steady-state attach or a recovery
/// overlap that runs alongside the previous session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportAttachIntent {
    /// Normal attach during steady-state operation.
    SteadyState,
    /// Attach that runs concurrently with the previous session during recovery.
    RecoveryOverlap,
}

/// Whether a transport session originated in steady state or replaced a faulted
/// session.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransportSessionProvenance {
    /// Session was created during normal steady-state operation.
    SteadyOrigin,
    /// Session was created to replace a faulted session.
    RecoveryReplacement,
}

/// Cleanup mode for lingering (unreleased) transport sessions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LingeringCleanupMode {
    /// Must complete before a new attach is allowed.
    StrictPreAttach,
    /// Best-effort cleanup that proceeds in the background after start.
    BestEffortPostStart,
}

/// What triggered a lingering cleanup wave.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LingeringCleanupTrigger {
    /// Cleanup was triggered by a recovery pre-attach sequence.
    RecoveryPreAttach,
    /// Cleanup was triggered by post-start reconciliation.
    PostStartReconciliation,
    /// Cleanup is a deferred retry of a previous attempt.
    DeferredRetry,
}

/// Parameters for a transport session attach request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeTransportSessionAttachRequest {
    /// Sandbox being attached.
    pub sandbox_id: String,
    /// Lease ID assigned to this session.
    pub lease_id: String,
    /// Region ID assigned to this session.
    pub region_id: String,
    /// Intent of the attach (steady-state or recovery overlap).
    pub intent: TransportAttachIntent,
    /// Provenance of the session (steady origin or recovery replacement).
    pub provenance: TransportSessionProvenance,
    /// Processing epoch at which the attach was initiated, if known.
    pub attach_processing_epoch: Option<u64>,
    /// Backing file path for audio transport, if applicable.
    pub backing_path: Option<String>,
    /// Total byte size of the backing file, if applicable.
    pub total_bytes: Option<u32>,
}

/// Live record of one active transport concurrency session.
///
/// Included in `RuntimeTransportConcurrencySnapshot::active_sessions` for
/// detailed per-session inspection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTransportConcurrencySession {
    /// Sandbox this session belongs to.
    pub sandbox_id: String,
    /// Lease ID of this session.
    pub lease_id: String,
    /// Region ID of this session.
    pub region_id: String,
    /// Attach intent (steady-state or recovery overlap).
    pub intent: TransportAttachIntent,
    /// Provenance of this session.
    pub provenance: TransportSessionProvenance,
    /// Monotonically increasing sequence number assigned at attach.
    pub attach_sequence: u64,
    /// Processing epoch at which this session was attached, if known.
    pub attach_processing_epoch: Option<u64>,
    /// Current state of the transport session.
    pub state: TransportSessionState,
    /// Backing file path, if applicable.
    pub backing_path: Option<String>,
    /// Backing file size in bytes, if applicable.
    pub total_bytes: Option<u32>,
    /// Number of cleanup attempts made for this session.
    pub cleanup_attempt_count: u32,
    /// Cleanup mode used in the last cleanup attempt.
    pub last_cleanup_mode: Option<LingeringCleanupMode>,
    /// Cleanup wave number of the last cleanup attempt.
    pub last_cleanup_wave: Option<u64>,
    /// Whether a cleanup is currently in progress for this session.
    pub cleanup_in_progress: bool,
    /// Processing epoch at which the last cleanup was attempted.
    pub last_cleanup_epoch: Option<u64>,
    /// Error string from the last failed cleanup attempt, if any.
    pub last_cleanup_error: Option<String>,
}

/// Receipt returned when a lingering cleanup work item is enqueued.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LingeringCleanupQueueReceipt {
    /// Unique identifier for this cleanup work item.
    pub work_id: u64,
    /// Processing epoch at which the cleanup was enqueued.
    pub cleanup_epoch: u64,
    /// Cleanup wave number this item belongs to.
    pub cleanup_wave: u64,
}

/// Execution plan for cleaning up one lingering transport session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LingeringCleanupPlan {
    /// Unique identifier for this cleanup work item.
    pub work_id: u64,
    /// Processing epoch for this cleanup.
    pub cleanup_epoch: u64,
    /// Cleanup wave number.
    pub cleanup_wave: u64,
    /// Sandbox being cleaned up.
    pub sandbox_id: String,
    /// Cleanup mode to apply.
    pub mode: LingeringCleanupMode,
    /// Trigger that initiated this cleanup wave.
    pub trigger: LingeringCleanupTrigger,
    /// Number of times this plan has been retried.
    pub retry_count: u32,
    /// Processing epoch at which this plan was created.
    pub processing_epoch: u64,
    /// Processing epoch at which this plan becomes eligible to execute.
    pub ready_at_processing_epoch: u64,
    /// Lease ID to exclude from cleanup (the new active session).
    pub exclude_lease_id: Option<String>,
    /// Region ID to exclude from cleanup (the new active session).
    pub exclude_region_id: Option<String>,
    /// Candidate sessions eligible for cleanup.
    pub candidates: Vec<ActiveTransportConcurrencySession>,
}

/// Aggregated summary of all pending work items in one lingering cleanup wave.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingLingeringCleanupWaveSummary {
    /// Sandbox this cleanup wave targets.
    pub sandbox_id: String,
    /// Cleanup wave number.
    pub cleanup_wave: u64,
    /// Cleanup mode applied by this wave.
    pub mode: LingeringCleanupMode,
    /// Trigger that first started this wave.
    pub first_trigger: LingeringCleanupTrigger,
    /// Most recent trigger that added work to this wave.
    pub latest_trigger: LingeringCleanupTrigger,
    /// Number of pending work items remaining in this wave.
    pub pending_work_items: usize,
    /// Number of pending deferred-retry work items in this wave.
    pub deferred_retry_work_items: usize,
    /// Processing epoch when the first item in this wave was enqueued.
    pub first_cleanup_epoch: u64,
    /// Processing epoch when the most recent item in this wave was enqueued.
    pub latest_cleanup_epoch: u64,
    /// Processing epoch of the earliest item's creation.
    pub first_processing_epoch: u64,
    /// Processing epoch of the most recently created item.
    pub latest_processing_epoch: u64,
    /// Earliest ready-at epoch across all items in this wave.
    pub oldest_ready_at_processing_epoch: u64,
    /// Latest ready-at epoch across all items in this wave.
    pub newest_ready_at_processing_epoch: u64,
}

/// Live snapshot of the transport concurrency governor.
///
/// Tracks active, lingering, recovery-overlap, and detach-faulted sessions;
/// exposes the admission and rejection history for debugging transport health.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuntimeTransportConcurrencySnapshot {
    /// Maximum number of steady-state sessions allowed concurrently.
    pub steady_session_limit: usize,
    /// Maximum number of recovery-overlap sessions allowed concurrently.
    pub recovery_session_limit: usize,
    /// Number of sessions currently attached.
    pub current_attached_sessions: usize,
    /// Peak number of simultaneously attached sessions since startup.
    pub peak_attached_sessions: usize,
    /// Number of recovery-overlap sessions currently running.
    pub current_recovery_overlap_sessions: usize,
    /// Peak number of simultaneous recovery-overlap sessions since startup.
    pub peak_recovery_overlap_sessions: usize,
    /// Number of sessions that are lingering (not yet released).
    pub current_lingering_sessions: usize,
    /// Peak number of simultaneous lingering sessions since startup.
    pub peak_lingering_sessions: usize,
    /// Number of sessions currently awaiting detach.
    pub current_detach_requested_sessions: usize,
    /// Number of sessions that faulted during detach.
    pub current_detach_faulted_sessions: usize,
    /// Number of pending cleanup work items.
    pub pending_cleanup_work_items: usize,
    /// Number of pending deferred-retry cleanup work items.
    pub pending_deferred_retry_work_items: usize,
    /// Next epoch assigned to a cleanup work item.
    pub next_cleanup_epoch: u64,
    /// Earliest ready-at epoch across all pending cleanup items.
    pub oldest_pending_cleanup_ready_epoch: Option<u64>,
    /// Summaries of all active cleanup waves, keyed by sandbox and wave number.
    pub pending_cleanup_waves: Vec<PendingLingeringCleanupWaveSummary>,
    /// Detailed records of all currently active sessions.
    pub active_sessions: Vec<ActiveTransportConcurrencySession>,
    /// Sandbox ID of the last admitted session.
    pub last_admitted_sandbox_id: Option<String>,
    /// Sandbox ID of the last rejected session.
    pub last_rejected_sandbox_id: Option<String>,
    /// Reason the last session was rejected.
    pub last_rejection_reason: Option<String>,
}
