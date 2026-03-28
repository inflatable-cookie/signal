use crate::{
    build_block_report, bus, planning_group_for_node, stage_parameter_events_for_node,
    stage_processor::apply_stage, AudioBuffer, ExecutableGraph, GraphBlockReport, GraphBusState,
    GraphCapturedBusOutput, GraphExecutionLane, GraphNodeRenderOverride,
    GraphNodeSilencePolicy, GraphParameterBatch, GraphRealtimeExecutionRequest,
};

impl ExecutableGraph {
    /// Execute real-time dispatches from a prepared state.
    pub fn execute_realtime_from_prepared(
        &self,
        request: GraphRealtimeExecutionRequest<'_>,
    ) -> (AudioBuffer, GraphBlockReport) {
        self.execute_realtime_from_prepared_with_node_overrides(GraphRealtimeExecutionRequest {
            node_render_overrides: &[],
            captured_bus_ids: &[],
            ..request
        })
    }

    /// Execute with optional node render overrides.
    pub fn execute_realtime_from_prepared_with_node_overrides(
        &self,
        request: GraphRealtimeExecutionRequest<'_>,
    ) -> (AudioBuffer, GraphBlockReport) {
        let (buffer, report, _) = self
            .execute_realtime_from_prepared_with_node_overrides_and_bus_captures(
                GraphRealtimeExecutionRequest {
                    captured_bus_ids: &[],
                    ..request
                },
            );
        (buffer, report)
    }

    /// Execute with node overrides and bus capture support.
    pub fn execute_realtime_from_prepared_with_node_overrides_and_bus_captures(
        &self,
        request: GraphRealtimeExecutionRequest<'_>,
    ) -> (AudioBuffer, GraphBlockReport, Vec<GraphCapturedBusOutput>) {
        let realtime_dispatches = request
            .planning
            .dispatches
            .iter()
            .filter(|dispatch| dispatch.lane == GraphExecutionLane::Realtime)
            .collect::<Vec<_>>();

        let prework_output_peak = request.prepared.as_ref().map(|prepared| prepared.output_peak);
        let mut realtime_input_peak = prework_output_peak;
        let mut working_state = request
            .prepared
            .as_ref()
            .map(bus::prepared_bus_state)
            .unwrap_or_else(|| bus::seeded_bus_state(request.input));

        if !realtime_dispatches.is_empty() {
            if prework_output_peak.is_none() {
                realtime_input_peak = Some(bus::peak_abs(
                    bus::graph_output_buffer(&working_state, request.input).samples(),
                ));
            }
            self.execute_dispatches(
                &mut working_state,
                &realtime_dispatches,
                request.context.anticipative_enabled,
                request.parameter_batch,
                request.node_render_overrides,
            );
        }

        let working_buffer = bus::graph_output_buffer(&working_state, request.input);
        let captured_buses = request
            .captured_bus_ids
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
            build_block_report(
                self,
                &request,
                request.parameter_batch,
                request.prepared.as_ref().map_or(0, |prepared| prepared.dispatch_count),
                realtime_dispatches.len(),
                prework_output_peak,
                realtime_input_peak,
                &working_buffer,
                &working_state,
            ),
            captured_buses,
        )
    }

    pub(super) fn execute_dispatches(
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
                for node in self
                    .plan
                    .nodes
                    .iter()
                    .filter(|node| planning_group_for_node(node, anticipative_enabled) == *phase)
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
