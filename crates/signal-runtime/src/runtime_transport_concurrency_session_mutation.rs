use super::*;

impl RuntimeTransportConcurrencyState {
    pub(crate) fn set_policy(
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

    pub(crate) fn begin_session(
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

    pub(crate) fn mark_session_state(
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

    pub(crate) fn promote_session_to_steady_state(
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

    pub(crate) fn record_cleanup_failure(
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

    pub(crate) fn clear_cleanup_in_progress(
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

    pub(crate) fn end_session(
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
}
