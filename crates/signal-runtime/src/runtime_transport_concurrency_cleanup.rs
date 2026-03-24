use super::*;

impl RuntimeTransportConcurrencyState {
    fn next_cleanup_wave_for_sandbox(&mut self, sandbox_id: &str) -> u64 {
        let next = self
            .next_cleanup_wave_by_sandbox
            .entry(sandbox_id.to_string())
            .or_insert(1);
        let cleanup_wave = *next;
        *next = next.saturating_add(1);
        cleanup_wave
    }

    fn has_lingering_candidates(
        &self,
        sandbox_id: &str,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> bool {
        self.active_sessions.values().any(|session| {
            session.sandbox_id == sandbox_id
                && matches!(
                    session.state,
                    TransportSessionState::DetachRequested | TransportSessionState::DetachFaulted
                )
                && !matches!(
                    (exclude_lease_id, exclude_region_id),
                    (Some(exclude_lease_id), Some(exclude_region_id))
                        if session.lease_id == exclude_lease_id
                            && session.region_id == exclude_region_id
                )
        })
    }

    pub(crate) fn enqueue_cleanup_work(
        &mut self,
        sandbox_id: &str,
        mode: LingeringCleanupMode,
        trigger: LingeringCleanupTrigger,
        retry_count: u32,
        processing_epoch: u64,
        cleanup_wave: Option<u64>,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
    ) -> Option<LingeringCleanupQueueReceipt> {
        if !self.has_lingering_candidates(sandbox_id, exclude_lease_id, exclude_region_id) {
            return None;
        }

        let work_id = self.next_cleanup_work_id;
        self.next_cleanup_work_id = self.next_cleanup_work_id.saturating_add(1);
        let cleanup_epoch = self.next_cleanup_epoch;
        self.next_cleanup_epoch = self.next_cleanup_epoch.saturating_add(1);
        let cleanup_wave =
            cleanup_wave.unwrap_or_else(|| self.next_cleanup_wave_for_sandbox(sandbox_id));
        let backoff = match trigger {
            LingeringCleanupTrigger::DeferredRetry => retry_count.max(1) as u64,
            LingeringCleanupTrigger::RecoveryPreAttach
            | LingeringCleanupTrigger::PostStartReconciliation => 0,
        };
        self.pending_cleanup_work
            .push_back(RuntimeLingeringCleanupWorkItem {
                work_id,
                cleanup_epoch,
                cleanup_wave,
                sandbox_id: sandbox_id.to_string(),
                mode,
                trigger,
                retry_count,
                processing_epoch,
                ready_at_processing_epoch: processing_epoch.saturating_add(backoff),
                exclude_lease_id: exclude_lease_id.map(ToOwned::to_owned),
                exclude_region_id: exclude_region_id.map(ToOwned::to_owned),
            });
        Some(LingeringCleanupQueueReceipt {
            work_id,
            cleanup_epoch,
            cleanup_wave,
        })
    }

    pub(crate) fn cleanup_attempt_count(
        &self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> u32 {
        self.active_sessions
            .get(&(
                sandbox_id.to_string(),
                lease_id.to_string(),
                region_id.to_string(),
            ))
            .map(|session| session.cleanup_attempt_count)
            .unwrap_or(0)
    }

    pub(crate) fn cleanup_wave_for_session(
        &self,
        sandbox_id: &str,
        lease_id: &str,
        region_id: &str,
    ) -> Option<u64> {
        self.active_sessions
            .get(&(
                sandbox_id.to_string(),
                lease_id.to_string(),
                region_id.to_string(),
            ))
            .and_then(|session| session.last_cleanup_wave)
    }

    pub(crate) fn dequeue_cleanup_work_for_sandbox(
        &mut self,
        sandbox_id: &str,
        current_processing_epoch: u64,
    ) -> Option<crate::interfaces::LingeringCleanupPlan> {
        let position = self.pending_cleanup_work.iter().position(|item| {
            item.sandbox_id == sandbox_id
                && item.ready_at_processing_epoch <= current_processing_epoch
        })?;
        let work = self.pending_cleanup_work.remove(position)?;
        let candidates = self.lingering_cleanup_candidates(
            work.sandbox_id.as_str(),
            work.exclude_lease_id.as_deref(),
            work.exclude_region_id.as_deref(),
            work.mode,
            work.processing_epoch,
            work.cleanup_wave,
        );
        if candidates.is_empty() {
            return None;
        }
        Some(crate::interfaces::LingeringCleanupPlan {
            work_id: work.work_id,
            cleanup_epoch: work.cleanup_epoch,
            cleanup_wave: work.cleanup_wave,
            sandbox_id: work.sandbox_id,
            mode: work.mode,
            trigger: work.trigger,
            retry_count: work.retry_count,
            processing_epoch: work.processing_epoch,
            ready_at_processing_epoch: work.ready_at_processing_epoch,
            exclude_lease_id: work.exclude_lease_id,
            exclude_region_id: work.exclude_region_id,
            candidates,
        })
    }

    fn lingering_cleanup_candidates(
        &mut self,
        sandbox_id: &str,
        exclude_lease_id: Option<&str>,
        exclude_region_id: Option<&str>,
        mode: LingeringCleanupMode,
        processing_epoch: u64,
        cleanup_wave: u64,
    ) -> Vec<crate::interfaces::ActiveTransportConcurrencySession> {
        let mut session_keys: Vec<_> = self
            .active_sessions
            .iter()
            .filter(|(_, session)| {
                session.sandbox_id == sandbox_id
                    && matches!(
                        session.state,
                        TransportSessionState::DetachRequested
                            | TransportSessionState::DetachFaulted
                    )
                    && !matches!(
                        (exclude_lease_id, exclude_region_id),
                        (Some(exclude_lease_id), Some(exclude_region_id))
                            if session.lease_id == exclude_lease_id
                                && session.region_id == exclude_region_id
                    )
            })
            .map(|(key, _)| key.clone())
            .collect();

        session_keys.sort_by(|left, right| {
            let left = self
                .active_sessions
                .get(left)
                .expect("missing left session");
            let right = self
                .active_sessions
                .get(right)
                .expect("missing right session");
            let left_key = (
                match left.provenance {
                    TransportSessionProvenance::SteadyOrigin => 0_u8,
                    TransportSessionProvenance::RecoveryReplacement => 1_u8,
                },
                left.attach_sequence,
                match left.state {
                    TransportSessionState::DetachRequested => 0_u8,
                    TransportSessionState::DetachFaulted => 1_u8,
                    _ => 2_u8,
                },
                left.lease_id.as_str(),
                left.region_id.as_str(),
            );
            let right_key = (
                match right.provenance {
                    TransportSessionProvenance::SteadyOrigin => 0_u8,
                    TransportSessionProvenance::RecoveryReplacement => 1_u8,
                },
                right.attach_sequence,
                match right.state {
                    TransportSessionState::DetachRequested => 0_u8,
                    TransportSessionState::DetachFaulted => 1_u8,
                    _ => 2_u8,
                },
                right.lease_id.as_str(),
                right.region_id.as_str(),
            );
            left_key.cmp(&right_key)
        });

        let mut sessions = Vec::with_capacity(session_keys.len());
        for key in session_keys {
            if let Some(session) = self.active_sessions.get_mut(&key) {
                session.cleanup_attempt_count = session.cleanup_attempt_count.saturating_add(1);
                session.last_cleanup_mode = Some(mode);
                session.last_cleanup_wave = Some(cleanup_wave);
                session.cleanup_in_progress = true;
                session.last_cleanup_epoch = Some(processing_epoch);
                session.last_cleanup_error = None;
                sessions.push(Self::active_session_view(session));
            }
        }

        sessions
    }
}
