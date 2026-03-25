use super::super::*;

impl SignalRuntime {
    pub fn forecast_transport_projection_for_block(
        &self,
        block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> TransportProjection {
        let loop_length_blocks = policy.transport_loop_length_blocks.max(1);
        let loop_end_samples = (self
            .config
            .graph
            .block_size
            .saturating_mul(loop_length_blocks)) as i64;
        let timeline_position_samples = ((block_sequence as i64)
            .saturating_mul(self.config.graph.block_size as i64))
        .rem_euclid(loop_end_samples);
        TransportProjection {
            playing: policy.transport_playing,
            timeline_position_samples,
            tempo_bpm: policy.transport_tempo_bpm,
            loop_state: Some(crate::interfaces::LoopRegion {
                start_samples: 0,
                end_samples: loop_end_samples,
            }),
        }
    }

    pub fn forecast_parameter_batch_for_block(
        &self,
        block_sequence: u64,
        policy: &RuntimePreworkForecastPolicy,
    ) -> ParameterBatch {
        let cycle_length = policy.parameter_cycle_length.max(1);
        let denominator = cycle_length.saturating_sub(1).max(1) as f32;
        ParameterBatch {
            epoch: self
                .projection_epoch
                .saturating_add(block_sequence)
                .saturating_add(1),
            events: vec![crate::interfaces::ParameterEvent {
                target: policy.parameter_target.clone(),
                sample_offset: 0,
                normalized_value: ((block_sequence % cycle_length) as f32) / denominator,
            }],
        }
    }

    pub fn apply_forecast_state_for_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<usize, RuntimeError> {
        let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime prework forecast policy must be set before applying forecast state",
            )
        })?;
        self.apply_forecast_transport_projection(
            self.forecast_transport_projection_for_block(block_sequence, &policy),
        )?;
        self.apply_parameter_batch(
            self.forecast_parameter_batch_for_block(block_sequence, &policy),
        )?;
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(0);
        }
        let _ = processing_epoch;
        Ok(self.reconcile_prework_window_with_forecast(block_sequence, &policy))
    }

    pub fn prepare_plugin_dispatch_state_for_block(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> Result<RuntimePluginDispatchState, RuntimeError> {
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(RuntimePluginDispatchState {
                transport: self.applied_transport,
                parameter_batch: None,
            });
        }

        let Some(policy) = self.prework_forecast_policy.clone() else {
            return Ok(RuntimePluginDispatchState {
                transport: self.applied_transport,
                parameter_batch: None,
            });
        };

        let transport = self.forecast_transport_projection_for_block(block_sequence, &policy);
        let parameter_batch = self.forecast_parameter_batch_for_block(block_sequence, &policy);
        self.apply_forecast_transport_projection(transport)?;
        self.apply_parameter_batch(parameter_batch.clone())?;
        let _ = processing_epoch;
        let _ = self.reconcile_prework_window_with_forecast(block_sequence, &policy);

        Ok(RuntimePluginDispatchState {
            transport: Some(transport),
            parameter_batch: Some(parameter_batch),
        })
    }

    pub(crate) fn apply_forecast_transport_projection(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError> {
        if projection.tempo_bpm <= 0.0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "tempo_bpm must be positive",
            ));
        }
        self.applied_transport = Some(projection);
        self.timeline.update_transport_state(projection);
        self.engine.snapshot.transport_epoch = self.timeline.transport_epoch;
        self.engine.snapshot.transport_transition = None;
        self.engine.snapshot.transport_loop_wrapped = false;
        Ok(())
    }
}
