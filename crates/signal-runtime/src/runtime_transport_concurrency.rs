use super::*;
#[path = "runtime_transport_concurrency_cleanup.rs"]
mod runtime_transport_concurrency_cleanup;
#[path = "runtime_transport_concurrency_session_mutation.rs"]
mod runtime_transport_concurrency_session_mutation;
#[path = "runtime_transport_concurrency_snapshot.rs"]
mod runtime_transport_concurrency_snapshot;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencyPolicy {
    steady_session_limit: usize,
    recovery_session_limit: usize,
}

impl Default for RuntimeTransportConcurrencyPolicy {
    fn default() -> Self {
        Self {
            steady_session_limit: 1,
            recovery_session_limit: 2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeTransportConcurrencySession {
    sandbox_id: String,
    lease_id: String,
    region_id: String,
    intent: TransportAttachIntent,
    provenance: TransportSessionProvenance,
    attach_sequence: u64,
    attach_processing_epoch: Option<u64>,
    state: TransportSessionState,
    backing_path: Option<String>,
    total_bytes: Option<u32>,
    cleanup_attempt_count: u32,
    last_cleanup_mode: Option<LingeringCleanupMode>,
    last_cleanup_wave: Option<u64>,
    cleanup_in_progress: bool,
    last_cleanup_epoch: Option<u64>,
    last_cleanup_error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeTransportSessionKey {
    sandbox_id: String,
    lease_id: String,
    region_id: String,
}

impl RuntimeTransportSessionKey {
    pub(super) fn from_parts(sandbox_id: &str, lease_id: &str, region_id: &str) -> Self {
        Self {
            sandbox_id: sandbox_id.to_string(),
            lease_id: lease_id.to_string(),
            region_id: region_id.to_string(),
        }
    }

    pub(super) fn as_map_key(&self) -> (String, String, String) {
        (
            self.sandbox_id.clone(),
            self.lease_id.clone(),
            self.region_id.clone(),
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeLingeringCleanupEnqueueRequest {
    pub(super) sandbox_id: String,
    pub(super) mode: LingeringCleanupMode,
    pub(super) trigger: LingeringCleanupTrigger,
    pub(super) retry_count: u32,
    pub(super) processing_epoch: u64,
    pub(super) cleanup_wave: Option<u64>,
    pub(super) exclude_session: Option<RuntimeTransportSessionKey>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RuntimeLingeringCleanupWorkItem {
    work_id: u64,
    cleanup_epoch: u64,
    cleanup_wave: u64,
    sandbox_id: String,
    mode: LingeringCleanupMode,
    trigger: LingeringCleanupTrigger,
    retry_count: u32,
    processing_epoch: u64,
    ready_at_processing_epoch: u64,
    exclude_lease_id: Option<String>,
    exclude_region_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct RuntimeTransportConcurrencyState {
    policy: RuntimeTransportConcurrencyPolicy,
    active_sessions: BTreeMap<(String, String, String), RuntimeTransportConcurrencySession>,
    pending_cleanup_work: VecDeque<RuntimeLingeringCleanupWorkItem>,
    peak_attached_sessions: usize,
    peak_recovery_overlap_sessions: usize,
    peak_lingering_sessions: usize,
    next_attach_sequence: u64,
    next_cleanup_work_id: u64,
    next_cleanup_epoch: u64,
    next_cleanup_wave_by_sandbox: BTreeMap<String, u64>,
    last_admitted_sandbox_id: Option<String>,
    last_rejected_sandbox_id: Option<String>,
    last_rejection_reason: Option<String>,
}

impl RuntimeTransportConcurrencyState {
    pub(super) fn active_states_for_sandbox(&self, sandbox_id: &str) -> Vec<TransportSessionState> {
        self.active_sessions
            .values()
            .filter(|session| session.sandbox_id == sandbox_id)
            .map(|session| session.state)
            .collect()
    }

    fn steady_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.intent == TransportAttachIntent::SteadyState)
            .count()
    }

    pub(super) fn recovery_overlap_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.intent == TransportAttachIntent::RecoveryOverlap)
            .count()
    }

    pub(super) fn lingering_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| {
                matches!(
                    session.state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
            })
            .count()
    }

    fn detach_requested_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.state == TransportSessionState::DetachRequested)
            .count()
    }

    pub(super) fn detach_faulted_session_count(&self) -> usize {
        self.active_sessions
            .values()
            .filter(|session| session.state == TransportSessionState::DetachFaulted)
            .count()
    }

    fn recovery_overlap_limit(&self) -> usize {
        self.policy
            .recovery_session_limit
            .saturating_sub(self.policy.steady_session_limit)
            .max(1)
    }

    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }
}

impl Default for RuntimeTransportConcurrencyState {
    fn default() -> Self {
        Self {
            policy: RuntimeTransportConcurrencyPolicy::default(),
            active_sessions: BTreeMap::new(),
            pending_cleanup_work: VecDeque::new(),
            peak_attached_sessions: 0,
            peak_recovery_overlap_sessions: 0,
            peak_lingering_sessions: 0,
            next_attach_sequence: 1,
            next_cleanup_work_id: 1,
            next_cleanup_epoch: 1,
            next_cleanup_wave_by_sandbox: BTreeMap::new(),
            last_admitted_sandbox_id: None,
            last_rejected_sandbox_id: None,
            last_rejection_reason: None,
        }
    }
}
