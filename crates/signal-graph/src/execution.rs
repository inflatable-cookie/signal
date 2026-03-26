// Graph execution engine for real-time and anticipative processing
use crate::{
    bus, parameter_application_report, planning_group_for_node, stage_parameter_events_for_node,
    stage_processor::apply_stage, AudioBuffer, ExecutableGraph, GraphBlockReport,
    GraphBusLevelReport, GraphBusState, GraphCapturedBusOutput, GraphContractSummary,
    GraphExecutionContext, GraphExecutionLane, GraphExecutionRequest, GraphNodeRenderOverride,
    GraphNodeSilencePolicy, GraphParameterBatch, GraphPlanningSummary, GraphPreparedBus,
    GraphPreparedDispatch, GraphRoutingSummary,
};

impl ExecutableGraph {
    /// Execute the graph with a full execution request.
    pub fn execute(&self, request: GraphExecutionRequest) -> (AudioBuffer, GraphBlockReport) {
        let GraphExecutionRequest {
            context,
            mut buffer,
            parameter_batch,
        } = request;
        let report =
            self.process_with_parameter_batch(&mut buffer, context, parameter_batch.as_ref());
        (buffer, report)
    }

    /// Process the graph with the given buffer.
    pub fn process(&self, buffer: &mut AudioBuffer) -> GraphBlockReport {
        self.process_with_context(buffer, GraphExecutionContext::default())
    }

    /// Process with a specific execution context.
    pub fn process_with_context(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
    ) -> GraphBlockReport {
        self.process_with_parameter_batch(buffer, context, None)
    }

    /// Process with parameter batch and optional node render overrides.
    pub fn process_with_parameter_batch_and_node_overrides(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        node_render_overrides: &[GraphNodeRenderOverride],
    ) -> GraphBlockReport {
        let input_peak = bus::peak_abs(buffer.samples());
        let planning = self.planning_summary(context.anticipative_enabled);
        let contract = self.contract_summary();
        let routing = self.routing_summary();
        let prepared = self.prepare_anticipative(buffer, &context, parameter_batch);
        let (working_buffer, report) = self.execute_realtime_from_prepared_with_node_overrides(
            buffer,
            input_peak,
            prepared,
            context,
            parameter_batch,
            &planning,
            &contract,
            &routing,
            node_render_overrides,
        );
        *buffer = working_buffer;
        report
    }

    /// Process with parameter batch (no node overrides).
    pub fn process_with_parameter_batch(
        &self,
        buffer: &mut AudioBuffer,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
    ) -> GraphBlockReport {
        self.process_with_parameter_batch_and_node_overrides(buffer, context, parameter_batch, &[])
    }

    /// Prepare anticipative dispatches (pre-work) for the graph.
    pub fn prepare_anticipative(
        &self,
        buffer: &AudioBuffer,
        context: &GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
    ) -> Option<GraphPreparedDispatch> {
        let planning = self.planning_summary(context.anticipative_enabled);
        let anticipative_dispatches = planning
            .dispatches
            .iter()
            .filter(|dispatch| dispatch.lane == GraphExecutionLane::Anticipative)
            .collect::<Vec<_>>();
        if anticipative_dispatches.is_empty() {
            return None;
        }

        let mut prepared = bus::seeded_bus_state(buffer);
        self.execute_dispatches(
            &mut prepared,
            &anticipative_dispatches,
            context.anticipative_enabled,
            parameter_batch,
            &[],
        );
        let latencies = prepared.latencies.clone();
        let tails = prepared.tails.clone();

        Some(GraphPreparedDispatch {
            output_peak: bus::peak_abs_across_buses(&prepared),
            buses: prepared
                .buses
                .into_iter()
                .map(|(bus_id, buffer)| GraphPreparedBus {
                    latency_samples: latencies.get(&bus_id).copied().unwrap_or(0),
                    tail_samples: tails.get(&bus_id).copied().unwrap_or(0),
                    bus_id,
                    buffer,
                })
                .collect(),
            dispatch_count: anticipative_dispatches.len(),
        })
    }

    /// Execute real-time dispatches from a prepared state.
    pub fn execute_realtime_from_prepared(
        &self,
        input: &AudioBuffer,
        input_peak: f32,
        prepared: Option<GraphPreparedDispatch>,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        planning: &GraphPlanningSummary,
        contract: &GraphContractSummary,
        routing: &GraphRoutingSummary,
    ) -> (AudioBuffer, GraphBlockReport) {
        self.execute_realtime_from_prepared_with_node_overrides(
            input,
            input_peak,
            prepared,
            context,
            parameter_batch,
            planning,
            contract,
            routing,
            &[],
        )
    }

    /// Execute with optional node render overrides.
    pub fn execute_realtime_from_prepared_with_node_overrides(
        &self,
        input: &AudioBuffer,
        input_peak: f32,
        prepared: Option<GraphPreparedDispatch>,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        planning: &GraphPlanningSummary,
        contract: &GraphContractSummary,
        routing: &GraphRoutingSummary,
        node_render_overrides: &[GraphNodeRenderOverride],
    ) -> (AudioBuffer, GraphBlockReport) {
        let (buffer, report, _) = self
            .execute_realtime_from_prepared_with_node_overrides_and_bus_captures(
                input,
                input_peak,
                prepared,
                context,
                parameter_batch,
                planning,
                contract,
                routing,
                node_render_overrides,
                &[],
            );
        (buffer, report)
    }

    /// Execute with node overrides and bus capture support.
    pub fn execute_realtime_from_prepared_with_node_overrides_and_bus_captures(
        &self,
        input: &AudioBuffer,
        input_peak: f32,
        prepared: Option<GraphPreparedDispatch>,
        context: GraphExecutionContext,
        parameter_batch: Option<&GraphParameterBatch>,
        planning: &GraphPlanningSummary,
        contract: &GraphContractSummary,
        routing: &GraphRoutingSummary,
        node_render_overrides: &[GraphNodeRenderOverride],
        captured_bus_ids: &[String],
    ) -> (AudioBuffer, GraphBlockReport, Vec<GraphCapturedBusOutput>) {
        let realtime_dispatches = planning
            .dispatches
            .iter()
            .filter(|dispatch| dispatch.lane == GraphExecutionLane::Realtime)
            .collect::<Vec<_>>();

        let prework_output_peak = prepared.as_ref().map(|prepared| prepared.output_peak);
        let mut realtime_input_peak = prework_output_peak;
        let mut working_state = prepared
            .as_ref()
            .map(bus::prepared_bus_state)
            .unwrap_or_else(|| bus::seeded_bus_state(input));
        let parameter_report =
            parameter_application_report(&self.plan, input.frames().0, parameter_batch);

        if !realtime_dispatches.is_empty() {
            if prework_output_peak.is_none() {
                realtime_input_peak = Some(bus::peak_abs(
                    bus::graph_output_buffer(&working_state, input).samples(),
                ));
            }
            self.execute_dispatches(
                &mut working_state,
                &realtime_dispatches,
                context.anticipative_enabled,
                parameter_batch,
                node_render_overrides,
            );
        }

        let working_buffer = bus::graph_output_buffer(&working_state, input);
        let output_latency_samples = working_state
            .latencies
            .get("main:out")
            .copied()
            .unwrap_or_else(|| routing.output_latency_samples);
        let max_bus_latency_samples = working_state
            .latencies
            .values()
            .copied()
            .max()
            .unwrap_or_else(|| routing.max_bus_latency_samples);
        let output_tail_samples = working_state
            .tails
            .get("main:out")
            .copied()
            .unwrap_or_else(|| routing.output_tail_samples);
        let max_bus_tail_samples = working_state
            .tails
            .values()
            .copied()
            .max()
            .unwrap_or_else(|| routing.max_bus_tail_samples);
        let bus_levels = working_state
            .buses
            .iter()
            .map(|(bus_id, buffer)| GraphBusLevelReport {
                bus_id: bus_id.clone(),
                peak: bus::peak_abs(buffer.samples()),
                rms: bus::rms(buffer.samples()),
                latency_samples: working_state.latencies.get(bus_id).copied().unwrap_or(0),
                tail_samples: working_state.tails.get(bus_id).copied().unwrap_or(0),
            })
            .collect::<Vec<_>>();
        let captured_buses = captured_bus_ids
            .iter()
            .filter_map(|bus_id| {
                working_state
                    .buses
                    .get(bus_id)
                    .cloned()
                    .map(|buffer| GraphCapturedBusOutput {
                        bus_id: bus_id.clone(),
                        buffer,
                    })
            })
            .collect::<Vec<_>>();

        (
            working_buffer.clone(),
            GraphBlockReport {
                graph_id: self.plan.graph_id.clone(),
                context,
                node_count: self.node_count(),
                stateful_node_count: self.stateful_node_count(),
                latency_node_count: self.latency_node_count(),
                plugin_backed_node_count: self.plugin_backed_node_count(),
                contract_issue_count: contract.issue_count,
                silence_clear_node_count: contract.silence_clear_node_count,
                adaptive_channel_node_count: contract.adaptive_channel_node_count,
                resettable_node_count: contract.resettable_node_count,
                scratch_buffer_count: contract.scratch_buffer_count,
                track_lane_node_count: contract.track_lane_node_count,
                bus_node_count: contract.bus_node_count,
                send_return_node_count: contract.send_return_node_count,
                console_node_count: contract.console_node_count,
                routed_bus_count: routing.routed_bus_count,
                direct_edge_count: routing.direct_edge_count,
                fan_in_bus_count: routing.fan_in_bus_count,
                fan_out_bus_count: routing.fan_out_bus_count,
                mixed_bus_count: routing.mixed_bus_count,
                silent_source_bus_count: working_state.silent_source_bus_count,
                phase_count: planning.phase_count,
                anticipative_phase_count: planning.anticipative_phase_count,
                phase_order: planning.phase_order.clone(),
                lane_count: planning.lane_count,
                anticipative_lane_count: planning.anticipative_lane_count,
                lane_order: planning.lane_order.clone(),
                dispatch_count: planning.dispatch_count,
                dispatch_boundary_count: planning.dispatch_boundary_count,
                dispatch_order: planning
                    .dispatches
                    .iter()
                    .map(|dispatch| dispatch.lane)
                    .collect(),
                prepared_dispatch_count: prepared
                    .as_ref()
                    .map_or(0, |prepared| prepared.dispatch_count),
                realtime_dispatch_count: realtime_dispatches.len(),
                dispatch_handoff_count: usize::from(
                    prepared.is_some() && !realtime_dispatches.is_empty(),
                ),
                stage_count: self.stage_count(),
                dynamic_kernel_stage_count: self.dynamic_kernel_stage_count(),
                dynamic_stage_state_model: self.dynamic_stage_state_model(),
                total_latency_samples: self.total_latency_samples(),
                max_node_latency_samples: self.max_node_latency_samples(),
                total_tail_samples: self.total_tail_samples(),
                max_node_tail_samples: self.max_node_tail_samples(),
                output_latency_samples,
                max_bus_latency_samples,
                output_tail_samples,
                max_bus_tail_samples,
                parameter_epoch: parameter_batch.map(|batch| batch.epoch),
                parameter_event_count: parameter_report.event_count,
                parameter_targeted_node_count: parameter_report.targeted_node_count,
                parameter_ignored_event_count: parameter_report.ignored_event_count,
                parameter_sub_block_count: parameter_report.sub_block_count,
                parameter_coalesced_event_count: parameter_report.coalesced_event_count,
                frame_count: working_buffer.frames().0,
                channel_count: working_buffer.channel_count().0,
                input_peak,
                prework_output_peak,
                realtime_input_peak,
                output_peak: bus::peak_abs(working_buffer.samples()),
                output_rms: bus::rms(working_buffer.samples()),
                bus_level_count: bus_levels.len(),
                bus_levels,
                first_output_sample: working_buffer.samples().first().copied(),
            },
            captured_buses,
        )
    }

    fn execute_dispatches(
        &self,
        state: &mut GraphBusState,
        dispatches: &[&crate::GraphLaneDispatch],
        anticipative_enabled: bool,
        parameter_batch: Option<&GraphParameterBatch>,
        node_render_overrides: &[GraphNodeRenderOverride],
    ) {
        let node_render_overrides = bus::node_render_override_map(node_render_overrides);
        for dispatch in dispatches {
            for phase in &dispatch.phase_order {
                for node in
                    self.plan.nodes.iter().filter(|node| {
                        planning_group_for_node(node, anticipative_enabled) == *phase
                    })
                {
                    let mut working = bus::source_buffer_for_node(state, node);
                    let input_latency = state
                        .latencies
                        .get(&node.buffer_contract.input.bus_id)
                        .copied()
                        .unwrap_or(0);
                    let input_tail = state
                        .tails
                        .get(&node.buffer_contract.input.bus_id)
                        .copied()
                        .unwrap_or(0);
                    let input_was_silent = bus::peak_abs(working.samples()) == 0.0;
                    if input_was_silent {
                        state.silent_source_bus_count += 1;
                    }
                    if let Some(node_render_override) =
                        node_render_overrides.get(node.node_id.as_str())
                    {
                        let output = if node_render_override.bypassed {
                            bus::adapt_buffer_to_layout(
                                &working,
                                node.buffer_contract.output.channels,
                                node.buffer_contract.channel_adaptation,
                            )
                        } else {
                            bus::adapt_buffer_to_layout(
                                &node_render_override.buffer,
                                node.buffer_contract.output.channels,
                                node.buffer_contract.channel_adaptation,
                            )
                        };
                        bus::mix_buffer_into_bus(
                            state,
                            node.buffer_contract.output.bus_id.as_str(),
                            output,
                            input_latency.saturating_add(node_render_override.latency_samples),
                            input_tail.saturating_add(node_render_override.tail_samples),
                        );
                        continue;
                    }
                    if !bus::apply_node_contract(&mut working, node) {
                        if node.buffer_contract.silence_policy
                            == GraphNodeSilencePolicy::ClearOutput
                        {
                            bus::mix_buffer_into_bus(
                                state,
                                node.buffer_contract.output.bus_id.as_str(),
                                working,
                                input_latency.saturating_add(node.latency_samples),
                                input_tail.saturating_add(node.tail_samples),
                            );
                        }
                        continue;
                    }

                    for (stage_index, stage) in node.stages.iter().enumerate() {
                        let events = stage_parameter_events_for_node(
                            parameter_batch,
                            node,
                            stage_index,
                            stage,
                            working.frames().0,
                        );
                        apply_stage(
                            &mut working,
                            stage,
                            &events,
                            parameter_batch.map(|batch| batch.strategy),
                        );
                    }
                    let output = bus::adapt_buffer_to_layout(
                        &working,
                        node.buffer_contract.output.channels,
                        node.buffer_contract.channel_adaptation,
                    );
                    bus::mix_buffer_into_bus(
                        state,
                        node.buffer_contract.output.bus_id.as_str(),
                        output,
                        input_latency.saturating_add(node.latency_samples),
                        input_tail.saturating_add(node.tail_samples),
                    );
                }
            }
        }
    }
}
