use super::*;

impl SignalRuntime {
    /// Services the prework lane for the given number of cycles using the current forecast policy.
    pub fn service_prework_lane(
        &mut self,
        processing_epoch: u64,
        cycles: usize,
    ) -> Result<usize, RuntimeError> {
        if self.prework_forecast_mode == RuntimePreworkForecastMode::Disabled {
            return Ok(0);
        }
        let policy = self.prework_forecast_policy.clone().ok_or_else(|| {
            RuntimeError::new(
                RuntimeErrorKind::InvalidState,
                "runtime prework forecast policy must be set before servicing the prework lane",
            )
        })?;
        self.service_prework_lane_with_policy(
            processing_epoch,
            cycles,
            policy.prepare_budget_per_cycle,
        )
    }

    /// Allocates and returns the next block sequence number from the timeline.
    pub fn allocate_block_sequence(&mut self) -> u64 {
        self.timeline.allocate_block_sequence()
    }

    /// Records a completed block sequence and emits a lease rollover event if a rollover occurred.
    pub fn record_block_sequence(
        &mut self,
        sandbox_id: impl Into<String>,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
    ) {
        let sandbox_id = sandbox_id.into();
        if let Some(rollover) = self.timeline.record_block_sequence(
            &sandbox_id,
            processing_epoch,
            lease_id,
            block_sequence,
        ) {
            self.emit(RuntimeEvent::LeaseRollover {
                sandbox_id: rollover.sandbox_id,
                previous_lease_id: rollover.previous_lease_id,
                lease_id: rollover.lease_id,
                processing_epoch: rollover.processing_epoch,
                first_block_sequence: rollover.first_block_sequence,
            });
        }
    }

    /// Records a parameter automation summary for the given epoch and lease.
    pub fn record_automation_summary(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        summary: ParameterAutomationSummary,
    ) {
        self.automation
            .record_summary(processing_epoch, lease_id, summary);
    }

    /// Records a plugin event packet summary for the given epoch, lease, and block sequence.
    pub fn record_plugin_event_summary(
        &mut self,
        processing_epoch: u64,
        lease_id: impl Into<String>,
        block_sequence: u64,
        generated_event_bytes: u32,
        summary: EventPacketSummary,
    ) {
        self.plugin_events.record_summary(
            processing_epoch,
            lease_id,
            block_sequence,
            generated_event_bytes,
            summary,
        );
    }

    pub(crate) fn current_graph_parameter_batch(
        &self,
        context: &GraphExecutionContext,
        frame_count: usize,
    ) -> (Option<GraphParameterBatch>, RuntimeAutomationBatchMetrics) {
        self.graph_parameter_batch_for_transport(context, frame_count, self.resolve_transport(None))
    }

    pub(crate) fn graph_parameter_batch_for_transport(
        &self,
        context: &GraphExecutionContext,
        frame_count: usize,
        transport: Option<TransportProjection>,
    ) -> (Option<GraphParameterBatch>, RuntimeAutomationBatchMetrics) {
        let Some(graph) = self.applied_graph.as_ref() else {
            return (None, RuntimeAutomationBatchMetrics::default());
        };

        let (automation_batch, automated_targets, mut automation_metrics) = self
            .automation
            .graph_parameter_batch(graph, transport, frame_count, context.parameter_epoch);

        let mut events = self
            .applied_parameter_batch
            .as_ref()
            .map(|batch| {
                batch
                    .events
                    .iter()
                    .filter(|event| !automated_targets.contains(&event.target))
                    .filter_map(|event| {
                        graph_parameter_target_from_runtime_target(graph, &event.target).map(
                            |target| GraphParameterEvent {
                                sample_offset: event
                                    .sample_offset
                                    .min(frame_count.saturating_sub(1)),
                                target,
                                value: event.normalized_value,
                            },
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if let Some(mut automation_events) = automation_batch.map(|batch| batch.events) {
            events.append(&mut automation_events);
        }
        if events.is_empty() {
            return (None, automation_metrics);
        }
        events.sort_by(|left, right| {
            left.target
                .node_id
                .cmp(&right.target.node_id)
                .then(left.target.stage_index.cmp(&right.target.stage_index))
                .then(
                    graph_stage_parameter_sort_key(left.target.parameter)
                        .cmp(&graph_stage_parameter_sort_key(right.target.parameter)),
                )
                .then(left.sample_offset.cmp(&right.sample_offset))
        });
        automation_metrics.max_sample_offset = events.iter().map(|event| event.sample_offset).max();
        (
            Some(GraphParameterBatch {
                epoch: context.parameter_epoch,
                strategy: GraphParameterApplicationStrategy::SplitAtEvents {
                    max_sub_blocks: automation_metrics.strategy_max_sub_blocks.max(8),
                },
                events,
            }),
            automation_metrics,
        )
    }

    pub(crate) fn emit(&mut self, event: RuntimeEvent) {
        for sink in &mut self.sinks {
            sink.push(event.clone());
        }
    }
}
