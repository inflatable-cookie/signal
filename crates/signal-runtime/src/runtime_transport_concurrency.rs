use super::*;
#[path = "runtime_transport_concurrency_cleanup.rs"]
mod runtime_transport_concurrency_cleanup;
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
    pub(super) fn set_policy(
        &mut self,
        steady_session_limit: usize,
        recovery_session_limit: usize,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        if steady_session_limit == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "steady-state transport session limit must be greater than zero",
            ));
        }
        if recovery_session_limit <= steady_session_limit {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "recovery transport session limit must exceed steady-state limit",
            ));
        }
        if self.steady_session_count() > steady_session_limit {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "active steady-state transport sessions exceed the requested limit",
            ));
        }
        if self.active_sessions.len() > recovery_session_limit {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "active transport sessions exceed the requested recovery limit",
            ));
        }
        self.policy = RuntimeTransportConcurrencyPolicy {
            steady_session_limit,
            recovery_session_limit,
        };
        Ok(self.snapshot())
    }

    pub(super) fn pending_work_item_count(&self) -> usize {
        self.pending_cleanup_work.len()
    }

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

    pub(super) fn begin_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        intent: TransportAttachIntent,
        provenance: TransportSessionProvenance,
        attach_processing_epoch: Option<u64>,
        backing_path: Option<String>,
        total_bytes: Option<u32>,
    ) -> Result<RuntimeTransportConcurrencySnapshot, RuntimeError> {
        let key = (
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        );
        if self.active_sessions.contains_key(&key) {
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some("transport session is already attached".to_string());
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "transport session is already attached",
            ));
        }

        let steady_sessions = self.steady_session_count();
        let recovery_sessions = self.recovery_overlap_session_count();

        if matches!(intent, TransportAttachIntent::SteadyState)
            && steady_sessions >= self.policy.steady_session_limit
        {
            let reason = format!(
                "steady-state transport session limit {} is already attached{}",
                self.policy.steady_session_limit,
                self.lingering_reason_suffix(TransportAttachIntent::SteadyState)
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        if matches!(intent, TransportAttachIntent::RecoveryOverlap)
            && recovery_sessions >= self.recovery_overlap_limit()
        {
            let reason = format!(
                "recovery overlap session limit {} is already attached{}",
                self.recovery_overlap_limit(),
                self.lingering_reason_suffix(TransportAttachIntent::RecoveryOverlap)
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        let limit = match intent {
            TransportAttachIntent::SteadyState => self.policy.steady_session_limit,
            TransportAttachIntent::RecoveryOverlap => self.policy.recovery_session_limit,
        };
        if self.active_sessions.len() >= limit {
            let reason = format!(
                "transport session admission exceeds {:?} limit {}",
                intent, limit
            );
            self.last_rejected_sandbox_id = Some(sandbox_id.to_string());
            self.last_rejection_reason = Some(reason.clone());
            return Err(RuntimeError::new(
                RuntimeErrorKind::ResourceUnavailable,
                reason,
            ));
        }

        self.active_sessions.insert(
            key,
            RuntimeTransportConcurrencySession {
                sandbox_id: sandbox_id.to_string(),
                lease_id: lease_id.to_string(),
                region_id: region_id.to_string(),
                intent,
                provenance,
                attach_sequence: self.next_attach_sequence,
                attach_processing_epoch,
                state: TransportSessionState::AttachActive,
                backing_path,
                total_bytes,
                cleanup_attempt_count: 0,
                last_cleanup_mode: None,
                last_cleanup_wave: None,
                cleanup_in_progress: false,
                last_cleanup_epoch: None,
                last_cleanup_error: None,
            },
        );
        self.next_attach_sequence = self.next_attach_sequence.saturating_add(1);
        self.peak_attached_sessions = self.peak_attached_sessions.max(self.active_sessions.len());
        let recovery_sessions = self.recovery_overlap_session_count();
        self.peak_recovery_overlap_sessions =
            self.peak_recovery_overlap_sessions.max(recovery_sessions);
        let lingering_sessions = self.lingering_session_count();
        self.peak_lingering_sessions = self.peak_lingering_sessions.max(lingering_sessions);
        self.last_admitted_sandbox_id = Some(sandbox_id.to_string());
        self.last_rejected_sandbox_id = None;
        self.last_rejection_reason = None;
        Ok(self.snapshot())
    }

    pub(super) fn mark_session_state(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        state: TransportSessionState,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.state = state;
        }
        let lingering_sessions = self.lingering_session_count();
        self.peak_lingering_sessions = self.peak_lingering_sessions.max(lingering_sessions);
        self.snapshot()
    }

    pub(super) fn promote_session_to_steady_state(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.intent = TransportAttachIntent::SteadyState;
        }
        self.snapshot()
    }

    pub(super) fn record_cleanup_failure(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        error: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.last_cleanup_mode = Some(mode);
            session.cleanup_in_progress = false;
            session.last_cleanup_epoch = Some(processing_epoch);
            session.last_cleanup_error = Some(error.to_string());
        }
        self.snapshot()
    }

    pub(super) fn clear_cleanup_in_progress(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        if let Some(session) = self.active_sessions.get_mut(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        )) {
            session.cleanup_in_progress = false;
            session.last_cleanup_error = None;
        }
        self.snapshot()
    }

    pub(super) fn end_session(
        &mut self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> RuntimeTransportConcurrencySnapshot {
        self.active_sessions.remove(&(
            sandbox_id.to_string(),
            lease_id.to_string(),
            region_id.to_string(),
        ));
        self.snapshot()
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
