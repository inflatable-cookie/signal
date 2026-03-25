use super::super::*;

impl SignalRuntime {
    pub(crate) fn invalidate_prework_for_forecast_plan_change_if_needed(
        &mut self,
        previous_requested_mode: RuntimePreworkForecastMode,
        previous_effective_mode: RuntimePreworkForecastMode,
        previous_profile: Option<RuntimePreworkForecastProfileSelection>,
        previous_profile_source: Option<RuntimePreworkForecastProfileSource>,
        previous_policy: Option<RuntimePreworkForecastPolicy>,
    ) -> Result<(), RuntimeError> {
        let changed = previous_requested_mode != self.prework_forecast_requested_mode
            || previous_effective_mode != self.prework_forecast_mode
            || previous_profile != self.prework_forecast_profile
            || previous_profile_source != self.prework_forecast_profile_source
            || previous_policy != self.prework_forecast_policy;
        if !changed {
            return Ok(());
        }

        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            if !matches!(
                self.engine.snapshot.last_prework_invalidation_reason,
                Some(
                    RuntimePreworkInvalidationReason::PlanningDisabled
                        | RuntimePreworkInvalidationReason::RuntimeReconfigured
                )
            ) {
                self.engine.invalidate_prework_cache(
                    RuntimePreworkInvalidationReason::ForecastPlanChanged,
                );
            }
            return Ok(());
        }

        let Some(policy) = self.prework_forecast_policy.clone() else {
            self.engine
                .invalidate_prework_cache(RuntimePreworkInvalidationReason::ForecastPlanChanged);
            return Ok(());
        };
        if self.engine.prework_queue.is_empty() {
            let _ = self.maybe_rebuild_prework_window_from_current_forecast_plan()?;
        } else {
            self.reconcile_prework_queue_with_current_forecast_plan(&policy);
        }
        self.reconcile_prework_service_state(None);
        Ok(())
    }

    pub(crate) fn reconcile_prework_queue_with_current_forecast_plan(
        &mut self,
        policy: &RuntimePreworkForecastPolicy,
    ) {
        let current_block_sequence = self
            .engine
            .prework_queue
            .iter()
            .map(|cache| cache.admitted_from_block_sequence)
            .max()
            .unwrap_or(0);
        let processing_epoch = self
            .engine
            .prework_queue
            .iter()
            .map(|cache| cache.source_processing_epoch)
            .max()
            .unwrap_or_else(|| {
                self.engine
                    .snapshot
                    .last_processing_epoch
                    .unwrap_or(self.projection_epoch)
            });
        let desired_sequences = (1..=policy.target_window_blocks)
            .map(|offset| current_block_sequence.saturating_add(offset as u64))
            .collect::<Vec<_>>();
        let projection_epoch = self.projection_epoch;
        let sample_rate = self.config.sample_rate;
        let block_size = self.config.graph.block_size;
        let retire_sequences = self
            .engine
            .prework_queue
            .iter()
            .filter_map(|cache| {
                let expected_loop_length_blocks = policy.transport_loop_length_blocks.max(1);
                let loop_end_samples =
                    (block_size.saturating_mul(expected_loop_length_blocks)) as i64;
                let expected_timeline_position_samples = ((cache.source_block_sequence as i64)
                    .saturating_mul(block_size as i64))
                .rem_euclid(loop_end_samples);
                let expected_parameter_epoch = projection_epoch
                    .saturating_add(cache.source_block_sequence)
                    .saturating_add(1);
                let expected_input_signature = hash_audio_buffer(&synthetic_stereo_block(
                    sample_rate,
                    FrameCount(block_size),
                    cache
                        .source_block_sequence
                        .saturating_add(policy.buffer_seed_offset),
                ));
                let compatible = cache.projection_epoch == projection_epoch
                    && cache.parameter_epoch == expected_parameter_epoch
                    && cache.transport.playing == policy.transport_playing
                    && cache.transport.tempo_bpm == policy.transport_tempo_bpm
                    && cache.transport.timeline_position_samples
                        == expected_timeline_position_samples
                    && cache.block_size == block_size
                    && cache.frame_count == block_size
                    && cache.channel_count == 2
                    && cache.input_signature == expected_input_signature
                    && cache.source_block_sequence > cache.admitted_from_block_sequence
                    && cache.source_block_sequence
                        <= cache
                            .admitted_from_block_sequence
                            .saturating_add(policy.target_window_blocks as u64);
                (!desired_sequences.contains(&cache.source_block_sequence) || !compatible)
                    .then_some(cache.source_block_sequence)
            })
            .collect::<Vec<_>>();
        self.engine.retire_prework_entries_matching(
            |cache| retire_sequences.contains(&cache.source_block_sequence),
            RuntimePreworkInvalidationReason::ForecastPlanChanged,
        );

        let targets = desired_sequences
            .into_iter()
            .map(|target_block_sequence| RuntimePreworkWindowTarget {
                target_block_sequence,
                admitted_from_block_sequence: current_block_sequence,
                buffer: synthetic_stereo_block(
                    sample_rate,
                    FrameCount(block_size),
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
            .collect::<Vec<_>>();
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
            block_size,
        );
        if self.control.running {
            let requested_cycles = self.multicore_prework_requested_cycles(1);
            let _ = self.service_prework_lane_with_policy(
                processing_epoch,
                requested_cycles,
                policy.prepare_budget_per_cycle,
            );
        } else {
            let _ = self.service_pending_prework_cycle(
                processing_epoch,
                policy.prepare_budget_per_cycle,
                RuntimePreworkBacklogClass::Deferred,
            );
        }
    }

    pub(crate) fn maybe_rebuild_prework_window_from_current_forecast_plan(
        &mut self,
    ) -> Result<usize, RuntimeError> {
        if !self.control.configured
            || self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled
        {
            return Ok(0);
        }
        if self.engine.graph.is_none() || !self.engine.snapshot.prework_cache_enabled {
            return Ok(0);
        }
        let Some(policy) = self.prework_forecast_policy.clone() else {
            return Ok(0);
        };
        let current_block_sequence = self
            .engine
            .snapshot
            .last_block_sequence
            .or_else(|| self.timeline.next_block_sequence.checked_sub(1))
            .unwrap_or(0);
        let processing_epoch = self
            .engine
            .snapshot
            .last_processing_epoch
            .unwrap_or(self.projection_epoch);
        let rebuilt = self.prime_engine_prework_window_with_forecast(
            processing_epoch,
            current_block_sequence,
            &policy,
        )?;
        self.reconcile_prework_service_state(Some(processing_epoch));
        Ok(rebuilt)
    }
}
