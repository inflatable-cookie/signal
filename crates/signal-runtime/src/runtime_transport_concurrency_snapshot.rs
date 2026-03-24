use super::*;

impl RuntimeTransportConcurrencyState {
    pub(super) fn active_session_view(
        session: &RuntimeTransportConcurrencySession,
    ) -> crate::interfaces::ActiveTransportConcurrencySession {
        crate::interfaces::ActiveTransportConcurrencySession {
            sandbox_id: session.sandbox_id.clone(),
            lease_id: session.lease_id.clone(),
            region_id: session.region_id.clone(),
            intent: session.intent,
            provenance: session.provenance,
            attach_sequence: session.attach_sequence,
            attach_processing_epoch: session.attach_processing_epoch,
            state: session.state,
            backing_path: session.backing_path.clone(),
            total_bytes: session.total_bytes,
            cleanup_attempt_count: session.cleanup_attempt_count,
            last_cleanup_mode: session.last_cleanup_mode,
            last_cleanup_wave: session.last_cleanup_wave,
            cleanup_in_progress: session.cleanup_in_progress,
            last_cleanup_epoch: session.last_cleanup_epoch,
            last_cleanup_error: session.last_cleanup_error.clone(),
        }
    }

    pub(crate) fn pending_deferred_retry_work_count(&self) -> usize {
        self.pending_cleanup_work
            .iter()
            .filter(|item| item.trigger == LingeringCleanupTrigger::DeferredRetry)
            .count()
    }

    fn oldest_pending_cleanup_ready_epoch(&self) -> Option<u64> {
        self.pending_cleanup_work
            .iter()
            .map(|item| item.ready_at_processing_epoch)
            .min()
    }

    fn pending_cleanup_waves(&self) -> Vec<crate::interfaces::PendingLingeringCleanupWaveSummary> {
        let mut by_wave: BTreeMap<
            (String, u64),
            crate::interfaces::PendingLingeringCleanupWaveSummary,
        > = BTreeMap::new();
        for item in &self.pending_cleanup_work {
            let key = (item.sandbox_id.clone(), item.cleanup_wave);
            let entry = by_wave.entry(key).or_insert_with(|| {
                crate::interfaces::PendingLingeringCleanupWaveSummary {
                    sandbox_id: item.sandbox_id.clone(),
                    cleanup_wave: item.cleanup_wave,
                    mode: item.mode,
                    first_trigger: item.trigger,
                    latest_trigger: item.trigger,
                    pending_work_items: 0,
                    deferred_retry_work_items: 0,
                    first_cleanup_epoch: item.cleanup_epoch,
                    latest_cleanup_epoch: item.cleanup_epoch,
                    first_processing_epoch: item.processing_epoch,
                    latest_processing_epoch: item.processing_epoch,
                    oldest_ready_at_processing_epoch: item.ready_at_processing_epoch,
                    newest_ready_at_processing_epoch: item.ready_at_processing_epoch,
                }
            });
            entry.latest_trigger = item.trigger;
            entry.pending_work_items = entry.pending_work_items.saturating_add(1);
            if item.trigger == LingeringCleanupTrigger::DeferredRetry {
                entry.deferred_retry_work_items = entry.deferred_retry_work_items.saturating_add(1);
            }
            entry.first_cleanup_epoch = entry.first_cleanup_epoch.min(item.cleanup_epoch);
            entry.latest_cleanup_epoch = entry.latest_cleanup_epoch.max(item.cleanup_epoch);
            entry.first_processing_epoch = entry.first_processing_epoch.min(item.processing_epoch);
            entry.latest_processing_epoch =
                entry.latest_processing_epoch.max(item.processing_epoch);
            entry.oldest_ready_at_processing_epoch = entry
                .oldest_ready_at_processing_epoch
                .min(item.ready_at_processing_epoch);
            entry.newest_ready_at_processing_epoch = entry
                .newest_ready_at_processing_epoch
                .max(item.ready_at_processing_epoch);
        }
        by_wave.into_values().collect()
    }

    pub(crate) fn lingering_reason_suffix(&self, intent: TransportAttachIntent) -> String {
        let lingering = self
            .active_sessions
            .values()
            .filter(|session| {
                session.intent == intent
                    && matches!(
                        session.state,
                        TransportSessionState::DetachRequested
                            | TransportSessionState::DetachFaulted
                    )
            })
            .count();
        if lingering == 0 {
            String::new()
        } else {
            format!(" ({lingering} lingering session(s) pending detach)")
        }
    }

    pub(crate) fn snapshot(&self) -> RuntimeTransportConcurrencySnapshot {
        RuntimeTransportConcurrencySnapshot {
            steady_session_limit: self.policy.steady_session_limit,
            recovery_session_limit: self.policy.recovery_session_limit,
            current_attached_sessions: self.active_sessions.len(),
            peak_attached_sessions: self.peak_attached_sessions,
            current_recovery_overlap_sessions: self.recovery_overlap_session_count(),
            peak_recovery_overlap_sessions: self.peak_recovery_overlap_sessions,
            current_lingering_sessions: self.lingering_session_count(),
            peak_lingering_sessions: self.peak_lingering_sessions,
            current_detach_requested_sessions: self.detach_requested_session_count(),
            current_detach_faulted_sessions: self.detach_faulted_session_count(),
            pending_cleanup_work_items: self.pending_cleanup_work.len(),
            pending_deferred_retry_work_items: self.pending_deferred_retry_work_count(),
            next_cleanup_epoch: self.next_cleanup_epoch,
            oldest_pending_cleanup_ready_epoch: self.oldest_pending_cleanup_ready_epoch(),
            pending_cleanup_waves: self.pending_cleanup_waves(),
            active_sessions: self
                .active_sessions
                .values()
                .map(Self::active_session_view)
                .collect(),
            last_admitted_sandbox_id: self.last_admitted_sandbox_id.clone(),
            last_rejected_sandbox_id: self.last_rejected_sandbox_id.clone(),
            last_rejection_reason: self.last_rejection_reason.clone(),
        }
    }
}
