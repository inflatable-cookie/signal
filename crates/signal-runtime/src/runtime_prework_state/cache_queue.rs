use super::super::super::*;

impl RuntimeEngineState {
    fn classify_prework_backlog_class(
        target_block_sequence: u64,
        admitted_from_block_sequence: u64,
    ) -> RuntimePreworkBacklogClass {
        match target_block_sequence.saturating_sub(admitted_from_block_sequence) {
            0 | 1 => RuntimePreworkBacklogClass::Immediate,
            2 => RuntimePreworkBacklogClass::NearTerm,
            _ => RuntimePreworkBacklogClass::Deferred,
        }
    }

    pub(crate) fn update_prework_queue_snapshot(
        &mut self,
        current_block_sequence: Option<u64>,
        preserve_invalidated: bool,
    ) {
        self.snapshot.prework_cache_queue_capacity = PREWORK_QUEUE_CAPACITY;
        self.snapshot.prework_cache_queue_depth = self.prework_queue.len();
        self.snapshot.prework_cache_peak_queue_depth = self
            .snapshot
            .prework_cache_peak_queue_depth
            .max(self.prework_queue.len());
        self.snapshot.prework_pending_target_count = self.pending_prework_targets.len();
        self.snapshot.prework_pending_immediate_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::Immediate)
            .count();
        self.snapshot.prework_pending_near_term_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::NearTerm)
            .count();
        self.snapshot.prework_pending_deferred_target_count = self
            .pending_prework_targets
            .iter()
            .filter(|target| target.backlog_class == RuntimePreworkBacklogClass::Deferred)
            .count();
        self.snapshot.prework_next_pending_target_block_sequence = self
            .pending_prework_targets
            .iter()
            .map(|target| target.target_block_sequence)
            .min();
        let mut target_block_sequences = self
            .prework_queue
            .iter()
            .map(|cache| cache.source_block_sequence)
            .chain(
                self.pending_prework_targets
                    .iter()
                    .map(|target| target.target_block_sequence),
            )
            .collect::<Vec<_>>();
        target_block_sequences.sort_unstable();
        target_block_sequences.dedup();
        self.snapshot.prework_cache_window_target_count = target_block_sequences.len();
        self.snapshot.prework_cache_window_target_block_sequences = target_block_sequences;

        let latest = self.prework_queue.back();
        self.snapshot.prework_cache_freshness_state =
            self.prework_freshness_state(latest, current_block_sequence);
        self.snapshot.prework_cache_remaining_valid_blocks = latest.map(|cache| {
            cache
                .valid_until_block_sequence
                .saturating_sub(current_block_sequence.unwrap_or(cache.source_block_sequence))
        });
        self.snapshot.prework_cache_valid_until_processing_epoch =
            latest.map(|cache| cache.valid_until_processing_epoch);
        self.snapshot.prework_cache_valid_until_block_sequence =
            latest.map(|cache| cache.valid_until_block_sequence);

        if latest.is_none() && !preserve_invalidated {
            self.snapshot.prework_cache_state = if self.snapshot.prework_cache_enabled {
                RuntimePreworkCacheState::Empty
            } else {
                RuntimePreworkCacheState::Disabled
            };
        }
    }

    fn pending_target_matches(
        pending: &RuntimePendingPreworkTarget,
        target: &RuntimePreworkWindowTarget,
    ) -> bool {
        pending.target_block_sequence == target.target_block_sequence
            && pending.admitted_from_block_sequence == target.admitted_from_block_sequence
            && pending.parameter_epoch_override == target.parameter_epoch_override
            && pending.transport_override == target.transport_override
            && pending.input_signature == hash_audio_buffer(&target.buffer)
            && pending.buffer.frames() == target.buffer.frames()
            && pending.buffer.channel_count() == target.buffer.channel_count()
    }

    fn prepared_target_matches(
        cache: &RuntimeEnginePreworkCache,
        graph_id: &str,
        target: &RuntimePreworkWindowTarget,
        projection_epoch: u64,
        latest_parameter_epoch: u64,
        applied_transport: Option<TransportProjection>,
        block_size: usize,
    ) -> bool {
        let transport = target.transport_override.or(applied_transport);
        cache.graph_id == graph_id
            && cache.source_block_sequence == target.target_block_sequence
            && cache.admitted_from_block_sequence == target.admitted_from_block_sequence
            && cache.projection_epoch == projection_epoch
            && cache.parameter_epoch
                == target
                    .parameter_epoch_override
                    .unwrap_or(latest_parameter_epoch)
            && cache.transport.playing == transport.map(|t| t.playing).unwrap_or(false)
            && cache.transport.tempo_bpm == transport.map(|t| t.tempo_bpm).unwrap_or(0.0)
            && cache.transport.timeline_position_samples
                == transport.map(|t| t.timeline_position_samples).unwrap_or(0)
            && cache.block_size == block_size
            && cache.frame_count == target.buffer.frames().0
            && cache.channel_count == target.buffer.channel_count().0
            && cache.input_signature == hash_audio_buffer(&target.buffer)
    }

    pub(crate) fn reconcile_pending_prework_targets(
        &mut self,
        targets: &[RuntimePreworkWindowTarget],
        graph_id: Option<&str>,
        projection_epoch: u64,
        latest_parameter_epoch: u64,
        applied_transport: Option<TransportProjection>,
        block_size: usize,
    ) {
        self.pending_prework_targets.retain(|pending| {
            targets
                .iter()
                .any(|target| Self::pending_target_matches(pending, target))
        });

        for target in targets {
            let already_prepared = graph_id.is_some_and(|graph_id| {
                self.prework_queue.iter().any(|cache| {
                    Self::prepared_target_matches(
                        cache,
                        graph_id,
                        target,
                        projection_epoch,
                        latest_parameter_epoch,
                        applied_transport,
                        block_size,
                    )
                })
            });
            let already_pending = self
                .pending_prework_targets
                .iter()
                .any(|pending| Self::pending_target_matches(pending, target));
            if !already_prepared && !already_pending {
                self.pending_prework_targets
                    .push_back(RuntimePendingPreworkTarget {
                        target_block_sequence: target.target_block_sequence,
                        admitted_from_block_sequence: target.admitted_from_block_sequence,
                        input_signature: hash_audio_buffer(&target.buffer),
                        buffer: target.buffer.clone(),
                        backlog_class: Self::classify_prework_backlog_class(
                            target.target_block_sequence,
                            target.admitted_from_block_sequence,
                        ),
                        parameter_epoch_override: target.parameter_epoch_override,
                        transport_override: target.transport_override,
                    });
            }
        }

        self.pending_prework_targets
            .make_contiguous()
            .sort_by_key(|target| (target.backlog_class, target.target_block_sequence));
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
    }

    pub(crate) fn take_pending_prework_targets(
        &mut self,
        budget: usize,
        max_backlog_class: RuntimePreworkBacklogClass,
    ) -> Vec<RuntimePendingPreworkTarget> {
        let mut drained = Vec::with_capacity(budget.min(self.pending_prework_targets.len()));
        let mut retained = VecDeque::with_capacity(self.pending_prework_targets.len());
        while let Some(target) = self.pending_prework_targets.pop_front() {
            if drained.len() < budget && target.backlog_class <= max_backlog_class {
                drained.push(target);
            } else {
                retained.push_back(target);
            }
        }
        self.pending_prework_targets = retained;
        self.update_prework_queue_snapshot(
            None,
            self.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
        );
        drained
    }

    pub(crate) fn matching_prework_index(
        &self,
        graph_id: &str,
        context: &GraphExecutionContext,
        buffer: &AudioBuffer,
        input_signature: u64,
    ) -> Option<usize> {
        self.prework_queue
            .iter()
            .enumerate()
            .rev()
            .find(|(_, cache)| {
                self.prework_cache_matches(cache, graph_id, context, buffer, input_signature)
            })
            .map(|(index, _)| index)
    }
}
