use super::*;

impl SignalRuntime {
    pub fn prepare_engine_prework_for_block(
        &mut self,
        processing_epoch: u64,
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
    ) -> Result<bool, RuntimeError> {
        self.prepare_engine_prework_for_block_with_future_state(
            processing_epoch,
            target_block_sequence,
            admitted_from_block_sequence,
            buffer,
            None,
            None,
        )
    }

    pub fn prepare_engine_prework_for_block_with_future_state(
        &mut self,
        processing_epoch: u64,
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
        buffer: AudioBuffer,
        parameter_epoch_override: Option<u64>,
        transport_override: Option<TransportProjection>,
    ) -> Result<bool, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before preparing engine prework",
            ));
        }
        let context = self.build_engine_execution_context_with_overrides(
            processing_epoch,
            target_block_sequence,
            parameter_epoch_override,
            transport_override,
        );
        let transport = self.resolve_transport(transport_override);
        self.engine.admit_prework_for_block(
            context,
            transport,
            admitted_from_block_sequence,
            buffer,
        )
    }

    pub fn prepare_engine_prework_window(
        &mut self,
        processing_epoch: u64,
        targets: Vec<RuntimePreworkWindowTarget>,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before preparing engine prework",
            ));
        }
        if targets.is_empty() {
            return Ok(0);
        }

        let mut sorted_targets = targets;
        sorted_targets.sort_by_key(|target| target.target_block_sequence);
        let current_block_sequence = sorted_targets
            .iter()
            .map(|target| target.admitted_from_block_sequence)
            .min()
            .unwrap_or(0);

        let planned_sequences: Vec<u64> = sorted_targets
            .iter()
            .map(|target| target.target_block_sequence)
            .collect();
        self.engine.retire_prework_entries_matching(
            |cache| {
                cache.source_block_sequence >= current_block_sequence
                    && !planned_sequences.contains(&cache.source_block_sequence)
            },
            RuntimePreworkInvalidationReason::PlanningWindowRevised,
        );
        self.engine.pending_prework_targets.clear();

        let mut admitted = 0usize;
        for target in sorted_targets {
            let context = self.build_engine_execution_context_with_overrides(
                processing_epoch,
                target.target_block_sequence,
                target.parameter_epoch_override,
                target.transport_override,
            );
            let transport = self.resolve_transport(target.transport_override);
            if self.engine.admit_prework_for_block(
                context,
                transport,
                target.admitted_from_block_sequence,
                target.buffer,
            )? {
                admitted = admitted.saturating_add(1);
            }
        }
        self.engine.update_prework_queue_snapshot(
            Some(current_block_sequence),
            self.engine.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        Ok(admitted)
    }

    pub(crate) fn service_pending_prework_cycle(
        &mut self,
        processing_epoch: u64,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Result<usize, RuntimeError> {
        if budget == 0 || !self.control.configured {
            return Ok(0);
        }
        let targets = self
            .engine
            .take_pending_prework_targets(budget, max_backlog_class);
        if targets.is_empty() {
            return Ok(0);
        }

        let mut admitted = 0usize;
        for target in targets {
            self.engine.record_last_serviced_pending_target(&target);
            if self.prepare_engine_prework_for_block_with_future_state(
                processing_epoch,
                target.target_block_sequence,
                target.admitted_from_block_sequence,
                target.buffer,
                target.parameter_epoch_override,
                target.transport_override,
            )? {
                admitted = admitted.saturating_add(1);
            }
        }
        Ok(admitted)
    }
}
