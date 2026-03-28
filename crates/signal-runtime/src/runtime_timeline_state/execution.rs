use super::*;

impl SignalRuntime {
    pub(crate) fn build_engine_execution_context(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
    ) -> GraphExecutionContext {
        self.build_engine_execution_context_with_overrides(
            processing_epoch,
            block_sequence,
            None,
            None,
        )
    }

    pub(crate) fn build_engine_execution_context_with_overrides(
        &self,
        processing_epoch: u64,
        block_sequence: u64,
        parameter_epoch_override: Option<u64>,
        transport_override: Option<TransportProjection>,
    ) -> GraphExecutionContext {
        let transport = self.resolve_transport(transport_override);
        GraphExecutionContext {
            processing_epoch,
            block_sequence,
            projection_epoch: self.projection_epoch,
            parameter_epoch: parameter_epoch_override.unwrap_or(self.latest_parameter_epoch),
            configured_block_size: self.config.graph.block_size,
            anticipative_enabled: self.anticipative_enabled,
            transport_playing: transport.map(|t| t.playing).unwrap_or(false),
            transport_tempo_bpm: transport.map(|t| t.tempo_bpm).unwrap_or(0.0),
            timeline_position_samples: transport.map(|t| t.timeline_position_samples).unwrap_or(0),
        }
    }

    pub(crate) fn resolve_transport(
        &self,
        transport_override: Option<TransportProjection>,
    ) -> Option<TransportProjection> {
        transport_override.or(self.applied_transport)
    }

    pub(crate) fn advance_engine_transport(
        &mut self,
        frame_count: i64,
    ) -> RuntimeEngineTransportAdvance {
        let Some(mut transport) = self.applied_transport else {
            return RuntimeEngineTransportAdvance::default();
        };
        let start_samples = Some(transport.timeline_position_samples);
        if !transport.playing || frame_count <= 0 {
            return RuntimeEngineTransportAdvance {
                start_samples,
                end_samples: start_samples,
                loop_wrapped: false,
            };
        }

        let advanced = transport
            .timeline_position_samples
            .saturating_add(frame_count);
        let mut loop_wrapped = false;
        transport.timeline_position_samples = if let Some(loop_region) = transport.loop_state {
            let loop_start = loop_region.start_samples;
            let loop_end = loop_region.end_samples;
            if loop_end > loop_start && advanced >= loop_end {
                loop_wrapped = true;
                let loop_len = loop_end.saturating_sub(loop_start);
                loop_start.saturating_add((advanced - loop_start).rem_euclid(loop_len))
            } else {
                advanced
            }
        } else {
            advanced
        };
        self.applied_transport = Some(transport);
        RuntimeEngineTransportAdvance {
            start_samples,
            end_samples: Some(transport.timeline_position_samples),
            loop_wrapped,
        }
    }

    pub(crate) fn apply_engine_transport_update(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        pending_transition: Option<RuntimePendingTransportTransition>,
        result: &mut RuntimeEngineBlockResult,
    ) {
        let transport_advance = self.advance_engine_transport(result.output.frames().0 as i64);
        self.timeline.record_engine_block_window(
            transport_advance.start_samples,
            transport_advance.end_samples,
        );
        if let Some(transport) = self.applied_transport {
            self.timeline.update_transport_state(transport);
            if transport_advance.loop_wrapped {
                self.timeline
                    .record_loop_wrap(processing_epoch, block_sequence, transport);
            }
        }
        result.snapshot.transport_epoch = self.timeline.transport_epoch;
        result.snapshot.transport_transition = pending_transition
            .map(|transition| transition.kind)
            .or(transport_advance
                .loop_wrapped
                .then_some(RuntimeTransportTransitionKind::LoopWrapped));
        result.snapshot.transport_block_start_samples = transport_advance.start_samples;
        result.snapshot.transport_block_end_samples = transport_advance.end_samples;
        result.snapshot.transport_loop_wrapped = transport_advance.loop_wrapped;
        self.engine.snapshot.transport_epoch = result.snapshot.transport_epoch;
        self.engine.snapshot.transport_transition = result.snapshot.transport_transition;
        self.engine.snapshot.transport_block_start_samples =
            result.snapshot.transport_block_start_samples;
        self.engine.snapshot.transport_block_end_samples =
            result.snapshot.transport_block_end_samples;
        self.engine.snapshot.transport_loop_wrapped = result.snapshot.transport_loop_wrapped;
    }

    pub(crate) fn finalize_engine_block_result(
        &mut self,
        processing_epoch: u64,
        block_sequence: u64,
        automation_metrics: RuntimeAutomationBatchMetrics,
        block_start: Instant,
        result: &mut RuntimeEngineBlockResult,
    ) -> Result<(), RuntimeError> {
        self.metering.capture(
            self.config.sample_rate.0,
            &result.output,
            result.meter_sources.clone(),
        );
        self.automation
            .record_execution(RuntimeAutomationExecutionRecord {
                block_sequence,
                timeline_position_samples: result.snapshot.transport_block_start_samples,
                transport_playing: result
                    .snapshot
                    .last_execution_context
                    .as_ref()
                    .map(|context| context.transport_playing),
                parameter_epoch: result.snapshot.parameter_epoch,
                parameter_event_count: result.snapshot.parameter_event_count,
                parameter_ignored_event_count: result.snapshot.parameter_ignored_event_count,
                parameter_sub_block_count: result.snapshot.parameter_sub_block_count,
                parameter_coalesced_event_count: result.snapshot.parameter_coalesced_event_count,
                metrics: automation_metrics,
            });
        self.recording_capture.record_output_block(&result.output);
        let _ = self.enforce_scheduler_after_engine_block(processing_epoch, block_sequence)?;
        self.refresh_scheduler_topology_summary();
        let execution_time_ns = block_start.elapsed().as_nanos().min(u64::MAX as u128) as u64;
        self.record_block_execution_timing_ns(result.output.frames().0, execution_time_ns);
        result.snapshot = self.engine.snapshot.clone();
        Ok(())
    }
}
