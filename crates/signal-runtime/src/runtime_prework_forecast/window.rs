use super::super::*;

impl SignalRuntime {
    pub(crate) fn reconcile_prework_window_with_forecast(
        &mut self,
        current_block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> usize {
        let desired_count = policy.target_window_blocks;
        let target_block_sequences =
            self.plan_prework_window_block_sequences(current_block_sequence, desired_count);
        if target_block_sequences.is_empty() {
            self.engine.update_prework_queue_snapshot(
                Some(current_block_sequence),
                self.engine.snapshot.prework_cache_state == RuntimePreworkCacheState::Invalidated,
            );
            return 0;
        }

        let targets = self.build_prework_window_targets(
            current_block_sequence,
            policy,
            target_block_sequences,
        );
        let graph_id = self
            .engine
            .graph
            .as_ref()
            .map(|graph| graph.graph_id().to_string());
        self.engine.reconcile_pending_prework_targets(
            &targets,
            graph_id.as_deref(),
            self.projection_epoch,
            self.latest_parameter_epoch,
            self.applied_transport,
            self.config.graph.block_size,
        );
        targets.len()
    }

    pub fn next_planned_prework_block_sequence(
        &self,
        after_block_sequence: Option<u64>,
    ) -> Option<u64> {
        self.engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let target = cache.source_block_sequence;
                match after_block_sequence {
                    Some(after) if target <= after => None,
                    _ => Some(target),
                }
            })
            .min()
    }

    pub fn plan_prework_window_block_sequences(
        &mut self,
        current_block_sequence: u64,
        desired_count: usize,
    ) -> Vec<u64> {
        if !self.engine.snapshot.prework_cache_enabled {
            return Vec::new();
        }

        let mut retained_sequences = self
            .engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let target = cache.source_block_sequence;
                (target > current_block_sequence).then_some(target)
            })
            .collect::<Vec<_>>();
        retained_sequences.sort_unstable();
        retained_sequences.dedup();

        if retained_sequences.len() > desired_count {
            retained_sequences.truncate(desired_count);
            self.engine.retire_prework_entries_matching(
                |cache| {
                    cache.source_block_sequence > current_block_sequence
                        && !retained_sequences.contains(&cache.source_block_sequence)
                },
                RuntimePreworkInvalidationReason::PlanningWindowRevised,
            );
        } else if desired_count == 0 {
            self.engine.retire_prework_entries_matching(
                |cache| cache.source_block_sequence > current_block_sequence,
                RuntimePreworkInvalidationReason::PlanningWindowRevised,
            );
            retained_sequences.clear();
        }

        let mut next_sequence = current_block_sequence.saturating_add(1);
        while retained_sequences.len() < desired_count {
            if !retained_sequences.contains(&next_sequence) {
                retained_sequences.push(next_sequence);
            }
            next_sequence = next_sequence.saturating_add(1);
        }
        retained_sequences
    }

    fn build_prework_window_targets(
        &self,
        current_block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
        target_block_sequences: Vec<u64>,
    ) -> Vec<RuntimePreworkWindowTarget> {
        target_block_sequences
            .into_iter()
            .map(|target_block_sequence| RuntimePreworkWindowTarget {
                target_block_sequence,
                admitted_from_block_sequence: current_block_sequence,
                buffer: synthetic_stereo_block(
                    self.config.sample_rate,
                    FrameCount(self.config.graph.block_size),
                    target_block_sequence.saturating_add(policy.buffer_seed_offset),
                ),
                parameter_epoch_override: Some(
                    self.forecast_parameter_batch_for_block(target_block_sequence, policy)
                        .epoch,
                ),
                transport_override: Some(
                    self.forecast_transport_projection_for_block(target_block_sequence, policy),
                ),
            })
            .collect()
    }
}
