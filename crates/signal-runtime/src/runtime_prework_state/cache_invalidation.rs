use super::super::super::*;

impl RuntimeEngineState {
    pub(crate) fn retire_prework_entry(
        &mut self,
        cache: RuntimeEnginePreworkCache,
        reason: RuntimePreworkInvalidationReason,
    ) {
        self.snapshot.prework_cache_invalidation_count = self
            .snapshot
            .prework_cache_invalidation_count
            .saturating_add(1);
        self.snapshot.last_prework_invalidation_reason = Some(reason);
        let retirement_reason = self.retirement_reason_from_invalidation(reason);
        let retired_unconsumed = cache.consumption_count == 0;
        self.snapshot.prework_cache_retirement_count = self
            .snapshot
            .prework_cache_retirement_count
            .saturating_add(1);
        if retired_unconsumed {
            self.snapshot.prework_cache_unconsumed_retirement_count = self
                .snapshot
                .prework_cache_unconsumed_retirement_count
                .saturating_add(1);
        } else {
            self.snapshot.prework_cache_consumed_retirement_count = self
                .snapshot
                .prework_cache_consumed_retirement_count
                .saturating_add(1);
        }
        self.snapshot.last_prework_retirement_reason = Some(retirement_reason);
        self.snapshot.last_prework_retired_unconsumed = Some(retired_unconsumed);
        self.snapshot.last_prework_retirement_processing_epoch =
            Some(cache.source_processing_epoch);
        self.snapshot.last_prework_retirement_block_sequence = Some(cache.source_block_sequence);
        self.snapshot.prework_cache_state = RuntimePreworkCacheState::Invalidated;
    }

    pub(crate) fn retire_prework_entries_matching(
        &mut self,
        mut should_retire: impl FnMut(&RuntimeEnginePreworkCache) -> bool,
        reason: RuntimePreworkInvalidationReason,
    ) {
        let mut index = 0;
        while index < self.prework_queue.len() {
            if should_retire(&self.prework_queue[index]) {
                let cache = self.prework_queue.remove(index).expect("queue index valid");
                self.retire_prework_entry(cache, reason);
            } else {
                index += 1;
            }
        }
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    pub(crate) fn retire_unready_or_mismatched_prework_for_current_block(
        &mut self,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) {
        let mut index = 0;
        while index < self.prework_queue.len() {
            let maybe_reason = {
                let cache = &self.prework_queue[index];
                if context.processing_epoch > cache.valid_until_processing_epoch {
                    Some(RuntimePreworkInvalidationReason::ProcessingEpochExpired)
                } else if cache.source_block_sequence > context.block_sequence {
                    None
                } else {
                    self.prework_cache_mismatch_reason(
                        cache,
                        graph_id,
                        context,
                        buffer,
                        input_signature,
                    )
                }
            };

            if let Some(reason) = maybe_reason {
                let cache = self.prework_queue.remove(index).expect("queue index valid");
                self.retire_prework_entry(cache, reason);
            } else {
                index += 1;
            }
        }
        self.update_prework_queue_snapshot(
            Some(context.block_sequence),
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    pub(crate) fn invalidate_prework_cache(&mut self, reason: RuntimePreworkInvalidationReason) {
        if !self.prework_queue.is_empty() {
            let drained = self.prework_queue.drain(..).collect::<Vec<_>>();
            for cache in drained {
                self.retire_prework_entry(cache, reason);
            }
            self.pending_prework_targets.clear();
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Invalidated;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.prework_cache_valid_until_processing_epoch = None;
            self.snapshot.prework_cache_valid_until_block_sequence = None;
            self.snapshot.last_prework_source_processing_epoch = None;
            self.snapshot.last_prework_source_block_sequence = None;
            self.snapshot.prework_cache_queue_depth = 0;
        } else if self.snapshot.prework_cache_enabled {
            self.pending_prework_targets.clear();
            self.snapshot.prework_cache_state = RuntimePreworkCacheState::Invalidated;
            self.snapshot.prework_cache_freshness_state = RuntimePreworkFreshnessState::Invalidated;
            self.snapshot.prework_cache_remaining_valid_blocks = None;
            self.snapshot.last_prework_invalidation_reason = Some(reason);
        }
    }

    fn retirement_reason_from_invalidation(
        &self,
        reason: RuntimePreworkInvalidationReason,
    ) -> RuntimePreworkRetirementReason {
        match reason {
            RuntimePreworkInvalidationReason::RuntimeReconfigured => {
                RuntimePreworkRetirementReason::RuntimeReconfigured
            }
            RuntimePreworkInvalidationReason::RuntimeStopped => {
                RuntimePreworkRetirementReason::RuntimeStopped
            }
            RuntimePreworkInvalidationReason::ForecastPlanChanged => {
                RuntimePreworkRetirementReason::ForecastPlanChanged
            }
            RuntimePreworkInvalidationReason::PlanningDisabled => {
                RuntimePreworkRetirementReason::PlanningDisabled
            }
            RuntimePreworkInvalidationReason::GraphProjectionChanged => {
                RuntimePreworkRetirementReason::GraphProjectionChanged
            }
            RuntimePreworkInvalidationReason::TransportStarted => {
                RuntimePreworkRetirementReason::TransportStarted
            }
            RuntimePreworkInvalidationReason::TransportStopped => {
                RuntimePreworkRetirementReason::TransportStopped
            }
            RuntimePreworkInvalidationReason::TransportSeeked => {
                RuntimePreworkRetirementReason::TransportSeeked
            }
            RuntimePreworkInvalidationReason::TransportTempoChanged => {
                RuntimePreworkRetirementReason::TransportTempoChanged
            }
            RuntimePreworkInvalidationReason::TransportLoopStateChanged => {
                RuntimePreworkRetirementReason::TransportLoopStateChanged
            }
            RuntimePreworkInvalidationReason::TransportLoopWrapped => {
                RuntimePreworkRetirementReason::TransportLoopWrapped
            }
            RuntimePreworkInvalidationReason::ParameterBatchApplied => {
                RuntimePreworkRetirementReason::ParameterBatchApplied
            }
            RuntimePreworkInvalidationReason::InputSignatureChanged => {
                RuntimePreworkRetirementReason::InputSignatureChanged
            }
            RuntimePreworkInvalidationReason::ProcessingEpochExpired => {
                RuntimePreworkRetirementReason::ProcessingEpochExpired
            }
            RuntimePreworkInvalidationReason::BlockSequenceExpired => {
                RuntimePreworkRetirementReason::BlockSequenceExpired
            }
            RuntimePreworkInvalidationReason::SupersededByAdmission => {
                RuntimePreworkRetirementReason::SupersededByAdmission
            }
            RuntimePreworkInvalidationReason::PlanningWindowRevised => {
                RuntimePreworkRetirementReason::PlanningWindowRevised
            }
            RuntimePreworkInvalidationReason::QueueCapacityExceeded => {
                RuntimePreworkRetirementReason::QueueCapacityExceeded
            }
        }
    }
}
