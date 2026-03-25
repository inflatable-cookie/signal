use super::*;

impl SignalRuntime {
    pub(crate) fn require_handshake(&self) -> Result<(), RuntimeError> {
        if self.control.handshaken {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be handshaken before control requests",
            ))
        }
    }

    pub(crate) fn require_configured(&self) -> Result<(), RuntimeError> {
        if self.control.configured {
            Ok(())
        } else {
            Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime must be configured before this request",
            ))
        }
    }

    pub(crate) fn validate_automation_projection_request(
        projection: &RuntimeAutomationProjection,
    ) -> Result<(), RuntimeError> {
        for lane in &projection.lanes {
            if lane.automation_lane_id.is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "automation_lane_id must not be empty",
                ));
            }
            if lane.target.node_id.is_empty() || lane.target.parameter_id.is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "automation target node_id and parameter_id must not be empty",
                ));
            }
            if lane.resolution.max_sub_blocks == 0 {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "automation max_sub_blocks must be greater than zero",
                ));
            }
            if lane.interpolation == RuntimeAutomationInterpolation::Linear
                && lane.resolution.ramp_step_samples == 0
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "linear automation ramp_step_samples must be greater than zero",
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn validate_tempo_map_projection_request(
        projection: &RuntimeTempoMapProjection,
    ) -> Result<(), RuntimeError> {
        let mut segments = projection.segments.clone();
        segments.sort_by_key(|segment| (segment.start_samples, segment.segment_id.clone()));
        let mut previous_end = None;
        let mut previous_open_ended = false;
        for segment in &segments {
            if segment.segment_id.is_empty() {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "tempo map segment_id must not be empty",
                ));
            }
            if !segment.start_tempo_bpm.is_finite() || segment.start_tempo_bpm <= 0.0 {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "tempo map start_tempo_bpm must be positive",
                ));
            }
            if let Some(end_tempo_bpm) = segment.end_tempo_bpm {
                if !end_tempo_bpm.is_finite() || end_tempo_bpm <= 0.0 {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        "tempo map end_tempo_bpm must be positive",
                    ));
                }
            }
            if let Some(end_samples) = segment.end_samples {
                if end_samples <= segment.start_samples {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        "tempo map end_samples must be greater than start_samples",
                    ));
                }
            }
            if segment.interpolation == RuntimeTempoMapInterpolation::Linear
                && (segment.end_samples.is_none() || segment.end_tempo_bpm.is_none())
            {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "linear tempo map segments require end_samples and end_tempo_bpm",
                ));
            }
            if previous_open_ended {
                return Err(RuntimeError::new(
                    RuntimeErrorKind::InvalidRequest,
                    "open-ended tempo map segments must be the final segment",
                ));
            }
            if let Some(previous_end) = previous_end {
                if segment.start_samples < previous_end {
                    return Err(RuntimeError::new(
                        RuntimeErrorKind::InvalidRequest,
                        "tempo map segments must not overlap",
                    ));
                }
            }
            previous_open_ended = segment.end_samples.is_none();
            previous_end = segment.end_samples;
        }
        Ok(())
    }

    pub(crate) fn apply_transport_projection_state(
        &mut self,
        projection: TransportProjection,
    ) -> Result<(), RuntimeError> {
        if projection.tempo_bpm <= 0.0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "tempo_bpm must be positive",
            ));
        }

        let transition = classify_transport_transition(self.applied_transport, projection);
        let Some(transition) = transition else {
            self.applied_transport = Some(projection);
            self.timeline.update_transport_state(projection);
            return Ok(());
        };

        let current_ready_block = self.engine.snapshot.last_block_sequence.unwrap_or(0);
        let reason = classify_transport_invalidation_reason(self.applied_transport, projection);
        self.engine.retire_prework_entries_matching(
            |cache| {
                cache.source_block_sequence <= current_ready_block
                    && (cache.transport.playing != projection.playing
                        || cache.transport.tempo_bpm != projection.tempo_bpm
                        || cache.transport.timeline_position_samples
                            != projection.timeline_position_samples
                        || cache.transport.loop_state != projection.loop_state)
            },
            reason,
        );
        self.applied_transport = Some(projection);
        self.timeline.record_transport_projection(
            transition,
            self.engine
                .snapshot
                .last_block_sequence
                .map(|block_sequence| block_sequence.saturating_add(1)),
            self.engine.snapshot.last_processing_epoch,
            projection,
        );
        self.engine.snapshot.transport_epoch = self.timeline.transport_epoch;
        self.engine.snapshot.transport_transition = Some(transition);
        self.engine.snapshot.transport_loop_wrapped = false;
        Ok(())
    }

    pub(crate) fn apply_parameter_batch_state(
        &mut self,
        batch: ParameterBatch,
    ) -> Result<(), RuntimeError> {
        if batch.epoch < self.projection_epoch {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "parameter batch epoch is stale",
            ));
        }
        if !batch.events.is_empty() {
            let current_ready_block = self.timeline.next_block_sequence.saturating_sub(1);
            self.engine.retire_prework_entries_matching(
                |cache| {
                    cache.source_block_sequence <= current_ready_block
                        && cache.parameter_epoch != batch.epoch
                },
                RuntimePreworkInvalidationReason::ParameterBatchApplied,
            );
        }
        self.latest_parameter_epoch = batch.epoch;
        self.applied_parameter_batch = Some(batch);
        Ok(())
    }

    pub(crate) fn apply_hardware_config_state(
        &mut self,
        request: HardwareConfigRequest,
    ) -> Result<(), RuntimeError> {
        self.require_handshake()?;
        self.require_configured()?;
        if request.buffer_size == 0 || request.sample_rate.0 == 0 {
            return Err(RuntimeError::new(
                RuntimeErrorKind::InvalidRequest,
                "hardware config sample_rate and buffer_size must be non-zero",
            ));
        }

        self.config.sample_rate = request.sample_rate;
        self.config.graph.block_size = request.buffer_size;
        self.diagnostics.backend_policy_tier = request.backend_policy;
        self.emit(RuntimeEvent::EffectiveConfigChanged(
            self.get_effective_config(),
        ));
        Ok(())
    }
}
